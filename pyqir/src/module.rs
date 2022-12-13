// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    core::Context,
    core::{MemoryBuffer, Message},
    values::Value,
};
use core::slice;
#[allow(clippy::wildcard_imports, deprecated)]
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    bit_reader::LLVMParseBitcodeInContext,
    bit_writer::LLVMWriteBitcodeToMemoryBuffer,
    core::*,
    ir_reader::LLVMParseIRInContext,
    LLVMLinkage, LLVMModule,
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use std::{
    ffi::CString,
    ops::Deref,
    ptr::{self, NonNull},
    str,
};

/// A module is a collection of global values.
///
/// :param Context context: The LLVM context.
/// :param str name: The module name.
#[pyclass(unsendable)]
#[pyo3(text_signature = "(context, str)")]
pub(crate) struct Module {
    module: NonNull<LLVMModule>,
    context: Py<Context>,
}

#[pymethods]
impl Module {
    #[new]
    pub(crate) fn new(py: Python, context: Py<Context>, name: &str) -> Self {
        let name = CString::new(name).unwrap();
        let module = unsafe {
            LLVMModuleCreateWithNameInContext(name.as_ptr(), context.borrow(py).as_ptr())
        };
        Self {
            module: NonNull::new(module).unwrap(),
            context,
        }
    }

    /// Creates a module from LLVM IR.
    ///
    /// :param str ir: The LLVM IR for a module.
    /// :param typing.Optional[str] name: The name of the module.
    /// :returns: The module.
    /// :rtype: Module
    #[staticmethod]
    #[pyo3(text_signature = "(context, ir, name=\"\")")]
    fn from_ir(py: Python, context: Py<Context>, ir: &str, name: Option<&str>) -> PyResult<Self> {
        let name = CString::new(name.unwrap_or_default()).unwrap();

        // Don't dispose this buffer. LLVMParseIRInContext takes ownership.
        let buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRange(ir.as_ptr().cast(), ir.len(), name.as_ptr(), 0)
        };

        let mut module = ptr::null_mut();
        let mut error = ptr::null_mut();
        unsafe {
            let context_ref = context.borrow(py).as_ptr();
            if LLVMParseIRInContext(context_ref, buffer, &mut module, &mut error) != 0 {
                let error = Message::from_raw(error);
                return Err(PyValueError::new_err(error.to_str().unwrap().to_string()));
            }
        }

        Ok(Self {
            module: NonNull::new(module).unwrap(),
            context,
        })
    }

    /// Creates a module from LLVM bitcode.
    ///
    /// :param bytes bitcode: The LLVM bitcode for a module.
    /// :param typing.Optional[str] name: The name of the module.
    /// :returns: The module.
    /// :rtype: Module
    #[staticmethod]
    #[pyo3(text_signature = "(context, bitcode, name=\"\")")]
    fn from_bitcode(
        py: Python,
        context: Py<Context>,
        bitcode: &[u8],
        name: Option<&str>,
    ) -> PyResult<Self> {
        let name = CString::new(name.unwrap_or_default()).unwrap();
        let buffer = unsafe {
            MemoryBuffer::from_raw(LLVMCreateMemoryBufferWithMemoryRange(
                bitcode.as_ptr().cast(),
                bitcode.len(),
                name.as_ptr(),
                0,
            ))
        };

        let mut module = ptr::null_mut();
        let mut error = ptr::null_mut();
        let context_ref = context.borrow(py).as_ptr();

        unsafe {
            #[allow(deprecated)]
            if LLVMParseBitcodeInContext(context_ref, buffer.as_ptr(), &mut module, &mut error) == 0
            {
                Ok(Self {
                    module: NonNull::new(module).unwrap(),
                    context,
                })
            } else {
                let error = Message::from_raw(error);
                Err(PyValueError::new_err(error.to_str().unwrap().to_string()))
            }
        }
    }

    /// The name of the original source file that this module was compiled from.
    ///
    /// :type: str
    #[getter]
    fn source_filename(&self) -> &str {
        unsafe {
            let mut len = 0;
            let name = LLVMGetSourceFileName(self.as_ptr(), &mut len);
            str::from_utf8(slice::from_raw_parts(name.cast(), len)).unwrap()
        }
    }

    #[setter]
    fn set_source_filename(&self, value: &str) {
        unsafe {
            LLVMSetSourceFileName(self.as_ptr(), value.as_ptr().cast(), value.len());
        }
    }

    /// The functions declared in this module.
    ///
    /// :type: typing.List[Function]
    #[getter]
    fn functions(slf: Py<Module>, py: Python) -> PyResult<Vec<PyObject>> {
        let module = slf.borrow(py).as_ptr();
        let mut functions = Vec::new();
        unsafe {
            let mut function = LLVMGetFirstFunction(module);
            while !function.is_null() {
                functions.push(Value::from_raw(py, slf.clone_ref(py).into(), function)?);
                function = LLVMGetNextFunction(function);
            }
        }
        Ok(functions)
    }

    /// The LLVM bitcode for this module.
    ///
    /// :type: bytes
    #[getter]
    fn bitcode<'py>(&self, py: Python<'py>) -> &'py PyBytes {
        unsafe {
            let buffer = MemoryBuffer::from_raw(LLVMWriteBitcodeToMemoryBuffer(self.as_ptr()));
            let start = LLVMGetBufferStart(buffer.as_ptr());
            let len = LLVMGetBufferSize(buffer.as_ptr());
            PyBytes::new(py, slice::from_raw_parts(start.cast(), len))
        }
    }

    /// The LLVM context.
    ///
    /// :type: Context
    #[getter]
    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }

    /// Verifies that this module is valid.
    ///
    /// :returns: An error description if this module is invalid or `None` if this module is valid.
    /// :rtype: typing.Optional[str]
    fn verify(&self) -> Option<String> {
        unsafe {
            let action = LLVMVerifierFailureAction::LLVMReturnStatusAction;
            let mut error = ptr::null_mut();
            if LLVMVerifyModule(self.as_ptr(), action, &mut error) == 0 {
                None
            } else {
                let error = Message::from_raw(error);
                Some(error.to_str().unwrap().to_string())
            }
        }
    }

    /// Converts this module into an LLVM IR string.
    ///
    /// :rtype: str
    fn __str__(&self) -> String {
        unsafe {
            Message::from_raw(LLVMPrintModuleToString(self.as_ptr()))
                .to_str()
                .unwrap()
                .to_string()
        }
    }
}

impl Deref for Module {
    type Target = NonNull<LLVMModule>;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module.as_ptr());
        }
    }
}

impl Eq for Module {}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.module == other.module
    }
}

/// The linkage kind for a global value in a module.
#[pyclass]
#[derive(Clone, Copy)]
pub(crate) enum Linkage {
    #[pyo3(name = "APPENDING")]
    Appending,
    #[pyo3(name = "AVAILABLE_EXTERNALLY")]
    AvailableExternally,
    #[pyo3(name = "COMMON")]
    Common,
    #[pyo3(name = "EXTERNAL")]
    External,
    #[pyo3(name = "EXTERNAL_WEAK")]
    ExternalWeak,
    #[pyo3(name = "INTERNAL")]
    Internal,
    #[pyo3(name = "LINK_ONCE_ANY")]
    LinkOnceAny,
    #[pyo3(name = "LINK_ONCE_ODR")]
    LinkOnceOdr,
    #[pyo3(name = "PRIVATE")]
    Private,
    #[pyo3(name = "WEAK_ANY")]
    WeakAny,
    #[pyo3(name = "WEAK_ODR")]
    WeakOdr,
}

impl From<Linkage> for LLVMLinkage {
    fn from(linkage: Linkage) -> Self {
        match linkage {
            Linkage::Appending => Self::LLVMAppendingLinkage,
            Linkage::AvailableExternally => Self::LLVMAvailableExternallyLinkage,
            Linkage::Common => Self::LLVMCommonLinkage,
            Linkage::External => Self::LLVMExternalLinkage,
            Linkage::ExternalWeak => Self::LLVMExternalWeakLinkage,
            Linkage::Internal => Self::LLVMInternalLinkage,
            Linkage::LinkOnceAny => Self::LLVMLinkOnceAnyLinkage,
            Linkage::LinkOnceOdr => Self::LLVMLinkOnceODRLinkage,
            Linkage::Private => Self::LLVMPrivateLinkage,
            Linkage::WeakAny => Self::LLVMWeakAnyLinkage,
            Linkage::WeakOdr => Self::LLVMWeakODRLinkage,
        }
    }
}

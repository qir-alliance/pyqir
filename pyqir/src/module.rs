// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    core::Context,
    core::{MemoryBuffer, Message},
    metadata::Metadata,
    values::{Constant, Owner, Value},
};
use core::mem::forget;
use core::slice;
#[allow(clippy::wildcard_imports, deprecated)]
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    bit_reader::LLVMParseBitcodeInContext,
    bit_writer::LLVMWriteBitcodeToMemoryBuffer,
    core::*,
    ir_reader::LLVMParseIRInContext,
    linker::LLVMLinkModules2,
    LLVMLinkage, LLVMModule,
};
use pyo3::{exceptions::PyValueError, prelude::*, pyclass::CompareOp, types::PyBytes};
use qirlib::{context::set_diagnostic_handler, module::FlagBehavior};
use std::{
    collections::hash_map::DefaultHasher,
    ffi::CString,
    hash::{Hash, Hasher},
    ops::Deref,
    ptr::{self, NonNull},
    str,
};

/// A module is a collection of global values.
///
/// :param Context context: The LLVM context.
/// :param str name: The module name.
#[pyclass(unsendable)]
pub(crate) struct Module {
    module: NonNull<LLVMModule>,
    context: Py<Context>,
}

#[pymethods]
impl Module {
    #[new]
    #[pyo3(text_signature = "(context, name)")]
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

    /// Adds a flag to the llvm.module.flags metadata
    ///
    /// See https://llvm.org/docs/LangRef.html#module-flags-metadata
    ///
    /// :param ModuleFlagBehavior behavior: flag specifying the behavior when two (or more) modules are merged together
    /// :param str id: string that is a unique ID for the metadata.
    /// :param Union[Metadata, Value] flag: value of the flag
    #[pyo3(text_signature = "(behavior, id, flag)")]
    pub(crate) fn add_flag(
        &self,
        py: Python,
        behavior: ModuleFlagBehavior,
        id: &str,
        flag: Flag,
    ) -> PyResult<()> {
        let context = self.context().clone_ref(py);
        let _owner = Owner::merge(py, [Owner::Context(context), flag.owner().clone_ref(py)])?;
        let md = match flag {
            Flag::Constant(v) => unsafe { LLVMValueAsMetadata(v.into_super().as_ptr()) },
            Flag::Metadata(m) => m.as_ptr(),
        };
        unsafe {
            qirlib::module::add_flag(self.module.as_ptr(), behavior.into(), id, md);
        }
        Ok(())
    }

    /// Gets the flag value from the llvm.module.flags metadata for a given id
    ///
    /// See https://llvm.org/docs/LangRef.html#module-flags-metadata
    ///
    /// :param str id: metadata string that is a unique ID for the metadata.
    /// :returns: value of the flag if found, otherwise None
    /// :rtype: typing.Optional[Metadata]
    #[pyo3(text_signature = "(id)")]
    pub(crate) fn get_flag(slf: Py<Module>, py: Python, id: &str) -> Option<PyObject> {
        let module = slf.borrow(py).module.as_ptr();
        let flag = unsafe { LLVMGetModuleFlag(module, id.as_ptr().cast(), id.len()) };

        if flag.is_null() {
            return None;
        }

        let owner = slf.into();
        let value = unsafe { Metadata::from_raw(py, owner, flag) };
        value.ok()
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

    /// Link the supplied module into the current module.
    /// Destroys the supplied module.
    ///
    /// :raises: An error if linking failed.
    pub fn link(&self, other: Py<Module>, py: Python) -> PyResult<()> {
        let context = self.context.borrow(py).as_ptr();
        if context != other.borrow(py).context.borrow(py).as_ptr() {
            return Err(PyValueError::new_err(
                "Cannot link modules from different contexts. Modules are untouched.".to_string(),
            ));
        }
        unsafe {
            let mut c_char_output: *mut ::core::ffi::c_char = ptr::null_mut();
            let output = std::ptr::from_mut::<*mut ::core::ffi::c_char>(&mut c_char_output)
                .cast::<*mut ::core::ffi::c_void>()
                .cast::<::core::ffi::c_void>();

            set_diagnostic_handler(context, output);
            let result = LLVMLinkModules2(self.module.as_ptr(), other.borrow(py).module.as_ptr());
            // `forget` the other module. LLVM has destroyed it
            // and we'll get a segfault if we drop it.
            forget(other);
            if result == 0 {
                Ok(())
            } else {
                let error = Message::from_raw(c_char_output);
                return Err(PyValueError::new_err(error.to_str().unwrap().to_string()));
            }
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
#[derive(Clone, Copy, PartialEq, Hash)]
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

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl Linkage {
    // In order to implement the comparison operators, we have to do
    // it all in one impl of __richcmp__ for pyo3 to work.
    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
        match op {
            CompareOp::Eq => self.eq(other).into_py(py),
            CompareOp::Ne => (!self.eq(other)).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
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

/// Module flag behavior choices
#[pyclass]
#[derive(Clone)]
pub(crate) enum ModuleFlagBehavior {
    #[pyo3(name = "ERROR")]
    Error,
    #[pyo3(name = "WARNING")]
    Warning,
    #[pyo3(name = "REQUIRE")]
    Require,
    #[pyo3(name = "OVERRIDE")]
    Override,
    #[pyo3(name = "APPEND")]
    Append,
    #[pyo3(name = "APPEND_UNIQUE")]
    AppendUnique,
    #[pyo3(name = "MAX")]
    Max,
}

impl From<FlagBehavior> for ModuleFlagBehavior {
    fn from(flag: FlagBehavior) -> Self {
        match flag {
            FlagBehavior::Error => ModuleFlagBehavior::Error,
            FlagBehavior::Warning => ModuleFlagBehavior::Warning,
            FlagBehavior::Require => ModuleFlagBehavior::Require,
            FlagBehavior::Override => ModuleFlagBehavior::Override,
            FlagBehavior::Append => ModuleFlagBehavior::Append,
            FlagBehavior::AppendUnique => ModuleFlagBehavior::AppendUnique,
            FlagBehavior::Max => ModuleFlagBehavior::Max,
        }
    }
}

impl From<ModuleFlagBehavior> for FlagBehavior {
    fn from(flag: ModuleFlagBehavior) -> Self {
        match flag {
            ModuleFlagBehavior::Error => FlagBehavior::Error,
            ModuleFlagBehavior::Warning => FlagBehavior::Warning,
            ModuleFlagBehavior::Require => FlagBehavior::Require,
            ModuleFlagBehavior::Override => FlagBehavior::Override,
            ModuleFlagBehavior::Append => FlagBehavior::Append,
            ModuleFlagBehavior::AppendUnique => FlagBehavior::AppendUnique,
            ModuleFlagBehavior::Max => FlagBehavior::Max,
        }
    }
}

#[derive(FromPyObject)]
pub(crate) enum Flag<'py> {
    Constant(PyRef<'py, Constant>),
    Metadata(PyRef<'py, Metadata>),
}

impl Flag<'_> {
    fn owner(&self) -> &Owner {
        match self {
            Flag::Constant(v) => v.as_ref().owner(),
            Flag::Metadata(m) => m.owner(),
        }
    }
}

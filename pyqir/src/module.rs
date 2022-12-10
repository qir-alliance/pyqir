// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{context::Context, values::Value};
use inkwell::{
    memory_buffer::MemoryBuffer,
    values::{AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum},
    LLVMReference,
};
use llvm_sys::core::LLVMValueAsMetadata;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use qirlib::llvm_wrapper::{LLVMRustAddModuleFlag, LLVMRustModFlagBehavior};
use std::mem::transmute;

/// A module is a collection of global values.
///
/// :param Context context: The LLVM context.
/// :param str name: The module name.
#[pyclass(unsendable)]
#[pyo3(text_signature = "(context, str)")]
pub(crate) struct Module {
    module: inkwell::module::Module<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Module {
    #[new]
    pub(crate) fn new(py: Python, context: Py<Context>, name: &str) -> Self {
        let module = {
            let context = context.borrow(py);
            let module = context.create_module(name);
            unsafe {
                transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
            }
        };
        Self { module, context }
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
        let buffer =
            MemoryBuffer::create_from_memory_range(ir.as_bytes(), name.unwrap_or_default());
        let module = {
            let context = context.borrow(py);
            let module = context
                .create_module_from_ir(buffer)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            unsafe {
                transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
            }
        };
        Ok(Self { module, context })
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
        let buffer = MemoryBuffer::create_from_memory_range(bitcode, name.unwrap_or_default());
        let module = {
            let context = context.borrow(py);
            let module = inkwell::module::Module::parse_bitcode_from_buffer(&buffer, &**context)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            unsafe {
                transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
            }
        };
        Ok(Self { module, context })
    }

    /// The name of the original source file that this module was compiled from.
    ///
    /// :type: str
    #[getter]
    fn source_filename(&self) -> &str {
        self.module
            .get_source_file_name()
            .to_str()
            .expect("Name is not valid UTF-8.")
    }

    #[setter]
    fn set_source_filename(&self, value: &str) {
        self.module.set_source_file_name(value);
    }

    /// The functions declared in this module.
    ///
    /// :type: typing.List[Function]
    #[getter]
    fn functions(slf: Py<Module>, py: Python) -> PyResult<Vec<PyObject>> {
        slf.borrow(py)
            .module
            .get_functions()
            .map(|f| unsafe { Value::from_any(py, slf.clone_ref(py).into(), f) })
            .collect()
    }

    /// The LLVM bitcode for this module.
    ///
    /// :type: bytes
    #[getter]
    fn bitcode<'py>(&self, py: Python<'py>) -> &'py PyBytes {
        PyBytes::new(py, self.module.write_bitcode_to_memory().as_slice())
    }

    /// The LLVM context.
    ///
    /// :type: Context
    #[getter]
    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }

    /// Adds a metadata flag to the llvm.module.flags metadata
    ///
    /// See https://llvm.org/docs/LangRef.html#module-flags-metadata
    ///
    /// :param behavior: flag specifying the behavior when two (or more) modules are merged together
    /// :param id: metadata string that is a unique ID for the metadata.
    /// :param metadata: metadata value of the flag
    #[pyo3(text_signature = "(behavior, id, metadata)")]
    pub(crate) fn add_metadata_flag(
        &self,
        behavior: ModuleFlagBehavior,
        id: &str,
        metadata: &Value,
    ) -> PyResult<()> {
        let value = BasicMetadataValueEnum::try_from(unsafe { metadata.get() })?;
        let md = unsafe { LLVMValueAsMetadata(value.into_metadata_value().get_ref()) };

        unsafe {
            LLVMRustAddModuleFlag(
                self.module.get_ref(),
                behavior
                    .try_into()
                    .expect("Could not convert behavior for the current version of LLVM"),
                id.as_ptr() as *mut ::libc::c_char,
                id.len(),
                md,
            );
        }
        Ok(())
    }

    /// Adds a value flag to the llvm.module.flags metadata
    ///
    /// See https://llvm.org/docs/LangRef.html#module-flags-metadata
    ///
    /// :param behavior: flag specifying the behavior when two (or more) modules are merged together
    /// :param id: metadata string that is a unique ID for the metadata.
    /// :param value: value of the flag
    #[pyo3(text_signature = "(behavior, id, flag)")]
    pub(crate) fn add_value_flag(
        &self,
        behavior: ModuleFlagBehavior,
        id: &str,
        flag: &Value,
    ) -> PyResult<()> {
        let value = BasicValueEnum::try_from(unsafe { flag.get() })?;
        let md = unsafe { LLVMValueAsMetadata(value.get_ref()) };

        unsafe {
            LLVMRustAddModuleFlag(
                self.module.get_ref(),
                behavior
                    .try_into()
                    .expect("Could not convert behavior for the current version of LLVM"),
                id.as_ptr() as *mut ::libc::c_char,
                id.len(),
                md,
            );
        }
        Ok(())
    }

    /// Gets the flag value from the llvm.module.flags metadata for a given id
    ///
    /// See https://llvm.org/docs/LangRef.html#module-flags-metadata
    ///
    /// :param id: metadata string that is a unique ID for the metadata.
    /// :returns: value of the flag if found, otherwise None
    #[pyo3(text_signature = "(id)")]
    pub(crate) fn get_flag(slf: Py<Module>, py: Python, id: &str) -> Option<PyObject> {
        let flag = slf.borrow(py).module.get_flag(id);
        if let Some(flag) = flag {
            let ave = AnyValueEnum::MetadataValue(flag);
            let owner = slf.into();
            let value = unsafe { Value::from_any(py, owner, ave) };
            value.ok()
        } else {
            None
        }
    }

    /// Verifies that this module is valid.
    ///
    /// :returns: An error description if this module is invalid or `None` if this module is valid.
    /// :rtype: typing.Optional[str]
    fn verify(&self) -> Option<String> {
        self.module.verify().map_err(|e| e.to_string()).err()
    }

    /// Converts this module into an LLVM IR string.
    ///
    /// :rtype: str
    fn __str__(&self) -> String {
        self.module.to_string()
    }
}

impl Module {
    pub(crate) unsafe fn get(&self) -> &inkwell::module::Module<'static> {
        &self.module
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

impl From<Linkage> for inkwell::module::Linkage {
    fn from(linkage: Linkage) -> Self {
        match linkage {
            Linkage::Appending => Self::Appending,
            Linkage::AvailableExternally => Self::AvailableExternally,
            Linkage::Common => Self::Common,
            Linkage::External => Self::External,
            Linkage::ExternalWeak => Self::ExternalWeak,
            Linkage::Internal => Self::Internal,
            Linkage::LinkOnceAny => Self::LinkOnceAny,
            Linkage::LinkOnceOdr => Self::LinkOnceODR,
            Linkage::Private => Self::Private,
            Linkage::WeakAny => Self::WeakAny,
            Linkage::WeakOdr => Self::WeakODR,
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
    #[pyo3(name = "MIN")]
    Min,
}

impl From<LLVMRustModFlagBehavior> for ModuleFlagBehavior {
    fn from(flag: LLVMRustModFlagBehavior) -> Self {
        match flag {
            LLVMRustModFlagBehavior::Error => ModuleFlagBehavior::Error,
            LLVMRustModFlagBehavior::Warning => ModuleFlagBehavior::Warning,
            LLVMRustModFlagBehavior::Require => ModuleFlagBehavior::Require,
            LLVMRustModFlagBehavior::Override => ModuleFlagBehavior::Override,
            LLVMRustModFlagBehavior::Append => ModuleFlagBehavior::Append,
            LLVMRustModFlagBehavior::AppendUnique => ModuleFlagBehavior::AppendUnique,
            LLVMRustModFlagBehavior::Max => ModuleFlagBehavior::Max,
            #[cfg(any(feature = "llvm14-0"))]
            LLVMRustModFlagBehavior::Min => ModuleFlagBehavior::Min,
        }
    }
}

impl From<ModuleFlagBehavior> for LLVMRustModFlagBehavior {
    fn from(flag: ModuleFlagBehavior) -> Self {
        match flag {
            ModuleFlagBehavior::Error => LLVMRustModFlagBehavior::Error,
            ModuleFlagBehavior::Warning => LLVMRustModFlagBehavior::Warning,
            ModuleFlagBehavior::Require => LLVMRustModFlagBehavior::Require,
            ModuleFlagBehavior::Override => LLVMRustModFlagBehavior::Override,
            ModuleFlagBehavior::Append => LLVMRustModFlagBehavior::Append,
            ModuleFlagBehavior::AppendUnique => LLVMRustModFlagBehavior::AppendUnique,
            ModuleFlagBehavior::Max => LLVMRustModFlagBehavior::Max,
            ModuleFlagBehavior::Min => todo!("Min is not supported on LLVM < 14"),
        }
    }
}

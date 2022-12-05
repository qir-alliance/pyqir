// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{context::Context, values::Value};
use inkwell::{
    memory_buffer::MemoryBuffer,
    module::FlagBehavior,
    values::{AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum},
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
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

    #[pyo3(text_signature = "(key)")]
    pub(crate) fn get_flag(slf: Py<Module>, py: Python, key: &str) -> Option<PyObject> {
        let flag = slf.borrow(py).module.get_flag(key);
        let owner = slf.into();
        if let Some(flag) = flag {
            let ave = AnyValueEnum::MetadataValue(flag);
            let value = unsafe { Value::from_any(py, owner, ave) };
            value.ok()
        } else {
            None
        }
    }

    #[pyo3(text_signature = "(key, behavior, flag)")]
    pub(crate) fn add_metadata_flag(
        &self,
        key: &str,
        behavior: ModuleFlagBehavior,
        flag: &Value,
    ) -> PyResult<()> {
        let value = BasicMetadataValueEnum::try_from(unsafe { flag.get() })?;
        self.module
            .add_metadata_flag(key, behavior.into(), value.into_metadata_value());
        Ok(())
    }

    #[pyo3(text_signature = "(key, behavior, flag)")]
    pub(crate) fn add_value_flag(
        &self,
        key: &str,
        behavior: ModuleFlagBehavior,
        flag: &Value,
    ) -> PyResult<()> {
        let value = BasicValueEnum::try_from(unsafe { flag.get() })?;
        self.module
            .add_basic_value_flag(key, behavior.into(), value);
        Ok(())
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
        }
    }
}

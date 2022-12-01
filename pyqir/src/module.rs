// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{context::Context, values::Value};
use inkwell::{memory_buffer::MemoryBuffer, module::FlagBehavior, values::{BasicValueEnum, BasicMetadataValueEnum, MetadataValue, AnyValue}};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use std::mem::transmute;

/// A module is a collection of functions.
///
/// :param Context context: The global context.
/// :param str name: The module name.
#[pyclass(unsendable)]
pub struct Module {
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
    /// :param Optional[str] name: The name of the module.
    /// :rtype: Module
    /// :returns: The module.
    #[staticmethod]
    #[pyo3(text_signature = "(ir, name=\"\")")]
    fn from_ir(py: Python, ir: &str, name: Option<&str>) -> PyResult<Self> {
        let context = Context::new();
        let buffer =
            MemoryBuffer::create_from_memory_range(ir.as_bytes(), name.unwrap_or_default());
        let module = context
            .create_module_from_ir(buffer)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            module: unsafe {
                transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
            },
            context: Py::new(py, context)?,
        })
    }

    /// Creates a module from LLVM bitcode.
    ///
    /// :param bytes bitcode: The LLVM bitcode for a module.
    /// :param Optional[str] name: The name of the module.
    /// :rtype: Module
    /// :returns: The module.
    #[staticmethod]
    #[pyo3(text_signature = "(bitcode, name=\"\")")]
    fn from_bitcode(py: Python, bitcode: &[u8], name: Option<&str>) -> PyResult<Self> {
        let context = Context::new();
        let buffer = MemoryBuffer::create_from_memory_range(bitcode, name.unwrap_or_default());
        let module = inkwell::module::Module::parse_bitcode_from_buffer(&buffer, &*context)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            module: unsafe {
                transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
            },
            context: Py::new(py, context)?,
        })
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
    /// :type: List[Function]
    #[getter]
    fn functions(&self, py: Python) -> PyResult<Vec<PyObject>> {
        self.module
            .get_functions()
            .map(|f| unsafe { Value::from_any(py, self.context.clone(), f) })
            .collect()
    }

    /// The LLVM bitcode for this module.
    ///
    /// :type: bytes
    #[getter]
    fn bitcode<'py>(&self, py: Python<'py>) -> &'py PyBytes {
        PyBytes::new(py, self.module.write_bitcode_to_memory().as_slice())
    }

    /// The global context.
    ///
    /// :type: Context
    #[getter]
    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }

    fn __str__(&self) -> String {
        self.module.to_string()
    }
}

impl Module {
    pub(crate) unsafe fn get(&self) -> &inkwell::module::Module<'static> {
        &self.module
    }
}

/// An attribute.
#[pyclass(unsendable)]
pub(crate) struct Attribute(pub(crate) inkwell::attributes::Attribute);

#[pymethods]
impl Attribute {
    /// The value of the attribute as a string.
    #[getter]
    fn value(&self) -> &str {
        self.0
            .get_string_value()
            .to_str()
            .expect("Value is not valid UTF-8.")
    }
}

/// Verifies that a module is valid.
///
/// :returns: An error description if the module is invalid or `None` if the module is valid.
/// :rtype: Optional[str]
#[pyfunction]
pub(crate) fn verify_module(module: &Module) -> Option<String> {
    module.module.verify().map_err(|e| e.to_string()).err()
}

/// An instruction opcode.
#[pyclass]
#[derive(Clone)]
pub(crate) enum ModuleFlagBehavior {
    /// Emits an error if two values disagree, otherwise the resulting value is that of the operands.
    #[pyo3(name = "ERROR")]
    Error,
    /// Emits a warning if two values disagree. The result value will be the operand for the flag from the first module being linked.
    #[pyo3(name = "WARNING")]
    Warning,
    /// Adds a requirement that another module flag be present and have a specified value after linking is performed. The value must be a metadata pair, where the first element of the pair is the ID of the module flag to be restricted, and the second element of the pair is the value the module flag should be restricted to. This behavior can be used to restrict the allowable results (via triggering of an error) of linking IDs with the **Override** behavior.
    #[pyo3(name = "REQUIRE")]
    Require,
    /// Uses the specified value, regardless of the behavior or value of the other module. If both modules specify **Override**, but the values differ, an error will be emitted.
    #[pyo3(name = "OVERRIDE")]
    Override,
    /// Appends the two values, which are required to be metadata nodes.
    #[pyo3(name = "APPEND")]
    Append,
    /// Appends the two values, which are required to be metadata nodes. However, duplicate entries in the second list are dropped during the append operation.
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

#[pyfunction]
pub(crate) fn get_flag(py: Python, module: Py<Module>, key: &str) -> Option<PyObject> {
    let module  = module.borrow(py);
    if let Some(flag) = module.module.get_flag(key) {
        let ave = flag.as_any_value_enum();
        let value = unsafe { Value::from_any(py, module.context.clone(), ave)};
        value.ok()
    } else {
        None
    }
}

#[pyfunction]
pub(crate) fn add_metadata_flag(py: Python, module: Py<Module>, key: &str, behavior: ModuleFlagBehavior, flag: &Value)-> PyResult<()> {
    let module  = module.borrow(py);
    let value = BasicMetadataValueEnum::try_from(unsafe { flag.get() })?;
    module.module.add_metadata_flag(key, behavior.into(), value.into_metadata_value());
    Ok(())
}

#[pyfunction]
pub(crate) fn add_value_flag(module: &Module, key: &str, behavior: ModuleFlagBehavior, flag: &Value) -> PyResult<()> {
    let value = BasicValueEnum::try_from(unsafe { flag.get() })?;
    module.module.add_basic_value_flag(key, behavior.into(), value);
    Ok(())
}

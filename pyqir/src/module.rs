// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{context::Context, values::Value};
use inkwell::memory_buffer::MemoryBuffer;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use std::mem::transmute;

/// A module.
#[pyclass(unsendable)]
pub(crate) struct Module {
    module: inkwell::module::Module<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Module {
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

    fn __str__(&self) -> String {
        self.module.to_string()
    }
}

impl Module {
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

    pub(crate) unsafe fn get(&self) -> &inkwell::module::Module<'static> {
        &self.module
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
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

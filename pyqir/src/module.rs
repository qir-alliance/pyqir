// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    context::Context,
    values::{Owner, Value},
};
use inkwell::memory_buffer::MemoryBuffer;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use std::mem::transmute;

/// A module is a collection of functions.
///
/// :param Context context: The global context.
/// :param str name: The module name.
#[pyclass(unsendable)]
pub(crate) struct Module {
    module: inkwell::module::Module<'static>,
    context: Py<Context>,
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.module == other.module
    }
}

impl Eq for Module {}

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
    /// :param Optional[str] name: The name of the module.
    /// :rtype: Module
    /// :returns: The module.
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
    /// :type: List[Function]
    #[getter]
    fn functions(slf: Py<Module>, py: Python) -> PyResult<Vec<PyObject>> {
        slf.borrow(py)
            .module
            .get_functions()
            .map(|f| unsafe { Value::from_any(py, Owner::Module(slf.clone()), f) })
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

    /// Verifies that this module is valid.
    ///
    /// :returns: An error description if this module is invalid or `None` if this module is valid.
    /// :rtype: Optional[str]
    fn verify(&self) -> Option<String> {
        self.module.verify().map_err(|e| e.to_string()).err()
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

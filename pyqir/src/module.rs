// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{context::Context, values::Value};
use inkwell::memory_buffer::MemoryBuffer;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use std::mem::transmute;

#[pyclass(unsendable)]
pub(crate) struct Module {
    module: inkwell::module::Module<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Module {
    #[staticmethod]
    #[pyo3(text_signature = "(ir)")]
    fn from_ir(py: Python, ir: &str) -> PyResult<Self> {
        let context = inkwell::context::Context::create();
        let buffer = MemoryBuffer::create_from_memory_range(ir.as_bytes(), "");
        let module = context
            .create_module_from_ir(buffer)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let module = unsafe {
            transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
        };
        let context = Py::new(py, Context::new(context))?;
        Ok(Self { module, context })
    }

    #[staticmethod]
    #[pyo3(text_signature = "(bitcode)")]
    fn from_bitcode(py: Python, bitcode: &[u8]) -> PyResult<Self> {
        let context = inkwell::context::Context::create();
        let buffer = MemoryBuffer::create_from_memory_range(bitcode, "");
        let module = inkwell::module::Module::parse_bitcode_from_buffer(&buffer, &context)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let module = unsafe {
            transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
        };
        let context = Py::new(py, Context::new(context))?;
        Ok(Self { module, context })
    }

    #[getter]
    fn functions(&self, py: Python) -> PyResult<Vec<PyObject>> {
        self.module
            .get_functions()
            .map(|f| unsafe { Value::from_any(py, self.context.clone(), f) })
            .collect()
    }

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

    pub(crate) fn get(&self) -> &inkwell::module::Module<'static> {
        &self.module
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }
}

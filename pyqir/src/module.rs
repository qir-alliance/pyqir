// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{context::Context, values::Value};
use inkwell::memory_buffer::MemoryBuffer;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyList},
};
use std::mem::transmute;

#[pyclass(unsendable)]
pub(crate) struct Module {
    module: inkwell::module::Module<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Module {
    #[staticmethod]
    #[pyo3(text_signature = "(ir, name=\"\")")]
    fn from_ir(py: Python, ir: &str, name: Option<&str>) -> PyResult<Self> {
        let context = inkwell::context::Context::create();
        let buffer =
            MemoryBuffer::create_from_memory_range(ir.as_bytes(), name.unwrap_or_default());
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
    #[pyo3(text_signature = "(bitcode, name=\"\")")]
    fn from_bitcode(py: Python, bitcode: &[u8], name: Option<&str>) -> PyResult<Self> {
        let context = inkwell::context::Context::create();
        let buffer = MemoryBuffer::create_from_memory_range(bitcode, name.unwrap_or_default());
        let module = inkwell::module::Module::parse_bitcode_from_buffer(&buffer, &context)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let module = unsafe {
            transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
        };
        let context = Py::new(py, Context::new(context))?;
        Ok(Self { module, context })
    }

    #[staticmethod]
    #[pyo3(text_signature = "(modules, name=\"\")")]
    fn link<'py>(
        py: Python<'py>,
        modules: Vec<Py<Module>>,
        name: Option<&str>,
    ) -> PyResult<(&'py PyList, Self)> {
        let context = inkwell::context::Context::create();
        let modules: Vec<(&[u8], String)> = modules
            .iter()
            .map(|m| {
                let v = m.borrow(py);
                let bytes = v.bitcode(py).as_bytes();
                let name = v.module.get_name().to_str().unwrap().to_owned();
                (bytes, name)
            })
            .collect();
        let (module, names) = qirlib::module::link(&context, modules, name)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let module = unsafe {
            transmute::<inkwell::module::Module<'_>, inkwell::module::Module<'static>>(module)
        };
        let pynames = PyList::new(py, names);
        let context = Py::new(py, Context::new(context))?;
        Ok((pynames, Self { module, context }))
    }

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

#[pyclass(unsendable)]
pub(crate) struct Attribute(pub(crate) inkwell::attributes::Attribute);

#[pymethods]
impl Attribute {
    #[getter]
    fn value(&self) -> &str {
        self.0
            .get_string_value()
            .to_str()
            .expect("Value is not valid UTF-8.")
    }
}

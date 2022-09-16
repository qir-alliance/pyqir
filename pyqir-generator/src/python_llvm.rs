use pyo3::{exceptions::PyValueError, prelude::*};
use qirlib::inkwell::{context::Context, module::Module};
use std::{mem::transmute, path::PathBuf};

#[pyclass(unsendable)]
pub(crate) struct LlvmContext(Context);

#[pymethods]
impl LlvmContext {
    #[new]
    fn new() -> LlvmContext {
        LlvmContext(Context::create())
    }
}

#[pyclass(unsendable)]
pub(crate) struct LlvmModule {
    module: Module<'static>,
    _context: Py<LlvmContext>,
}

#[pymethods]
impl LlvmModule {
    #[new]
    fn new(py: Python, context: Py<LlvmContext>, name: &str) -> Self {
        let module = {
            let context = context.borrow(py);
            let module = context.0.create_module(name);
            unsafe { transmute::<Module, Module<'static>>(module) }
        };

        Self {
            module,
            _context: context,
        }
    }

    #[staticmethod]
    fn parse_bitcode_from_path(py: Python, context: Py<LlvmContext>, path: &str) -> PyResult<Self> {
        let module = {
            let context = context.borrow(py);
            let module = Module::parse_bitcode_from_path(path, &context.0)
                .map_err(|s| PyValueError::new_err(s.to_string()))?;
            unsafe { transmute::<Module, Module<'static>>(module) }
        };

        Ok(Self {
            module,
            _context: context,
        })
    }

    fn write_bitcode_to_path(&self, path: PathBuf) -> bool {
        self.module.write_bitcode_to_path(&path)
    }
}

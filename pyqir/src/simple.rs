// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    context::{self, Context},
    module::Module,
    types::Type,
    values::Value,
};
use inkwell::{module::Linkage, types::AnyTypeEnum};
use pyo3::{
    exceptions::{PyOSError, PyUnicodeDecodeError, PyValueError},
    prelude::*,
    types::PyBytes,
};
use qirlib::{passes, values};
use std::mem::transmute;

/// A simple module represents an executable program with these restrictions:
///
/// - There is one global qubit register and one global result register. Both are statically
///   allocated with a fixed size.
/// - There is only a single function that runs as the entry point.
///
/// :param str name: The name of the module.
/// :param int num_qubits: The number of statically allocated qubits.
/// :param int num_results: The number of statically allocated results.
#[pyclass(unsendable)]
#[pyo3(text_signature = "(name, num_qubits, num_results)")]
pub(crate) struct SimpleModule {
    module: Py<Module>,
    builder: Py<Builder>,
    num_qubits: u64,
    num_results: u64,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(py: Python, name: &str, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let context = Py::new(py, Context::new())?;
        let module = Py::new(py, Module::new(py, context.clone(), name))?;
        let builder = Py::new(py, Builder::new(py, context.clone()))?;

        {
            let context = context.borrow(py);
            let module = module.borrow(py);
            let builder = builder.borrow(py);
            let entry_point =
                values::entry_point(unsafe { module.get() }, "main", num_qubits, num_results);
            unsafe { builder.get() }
                .position_at_end(context.append_basic_block(entry_point, "entry"));
        }

        Ok(SimpleModule {
            module,
            builder,
            num_qubits,
            num_results,
        })
    }

    #[getter]
    fn context(&self, py: Python) -> Py<Context> {
        self.module.borrow(py).context().clone()
    }

    /// The global qubit register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn qubits(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let module = self.module.borrow(py);
        let context = module.context();
        let context_ref = unsafe { module.get() }.get_context();
        (0..self.num_qubits)
            .map(|id| unsafe {
                Value::from_any(py, context.clone(), values::qubit(&context_ref, id))
            })
            .collect()
    }

    /// The global result register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn results(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let module = self.module.borrow(py);
        let context = module.context();
        let context_ref = unsafe { module.get() }.get_context();
        (0..self.num_results)
            .map(|id| unsafe {
                Value::from_any(py, context.clone(), values::result(&context_ref, id))
            })
            .collect()
    }

    /// The instruction builder.
    ///
    /// :type: Builder
    #[getter]
    fn builder(&self) -> Py<Builder> {
        self.builder.clone()
    }

    /// Emits the LLVM IR for the module as plain text.
    ///
    /// :rtype: str
    fn ir(&self, py: Python) -> PyResult<String> {
        self.emit(py, |m| m.print_to_string().to_string())
    }

    /// Emits the LLVM bitcode for the module as a sequence of bytes.
    ///
    /// :rtype: bytes
    fn bitcode<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        self.emit(py, |m| {
            PyBytes::new(py, m.write_bitcode_to_memory().as_slice())
        })
    }

    /// Adds a declaration for an externally linked function to the module.
    ///
    /// :param str name: The name of the function.
    /// :param Type ty: The type of the function.
    /// :return: The function value.
    /// :rtype: Function
    #[pyo3(text_signature = "(self, name, ty)")]
    fn add_external_function(&mut self, py: Python, name: &str, ty: &Type) -> PyResult<PyObject> {
        let module = self.module.borrow(py);
        context::require_same(py, [module.context(), ty.context()])?;

        let context = ty.context().clone();
        let ty = unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty.get()) }
            .into_function_type();
        let function = unsafe { module.get() }.add_function(name, ty, None);
        unsafe { Value::from_any(py, context, function) }
    }

    /// Adds a global null-terminated string constant to the module.
    ///
    /// :param bytes Value: The string value without the null terminator.
    /// :returns: The global value.
    /// :rtype: Value
    #[pyo3(text_signature = "(value)")]
    fn add_global_string(&self, py: Python, value: &[u8]) -> PyResult<PyObject> {
        let module = self.module.borrow(py);
        let context = unsafe { module.get().get_context() };
        let value = context.const_string(value, true);
        let global = unsafe { module.get() }.add_global(
            context.i8_type().array_type(value.get_type().get_size()),
            None,
            "",
        );
        global.set_linkage(Linkage::Internal);
        global.set_constant(true);
        global.set_initializer(&value);
        unsafe { Value::from_any(py, module.context().clone(), global) }
    }
}

impl SimpleModule {
    fn emit<T>(&self, py: Python, f: impl Fn(&inkwell::module::Module) -> T) -> PyResult<T> {
        let module = self.module.borrow(py);
        let builder = self.builder.borrow(py);
        let context = inkwell::context::Context::create();

        let ret = unsafe { builder.get() }.build_return(None);
        let module = clone_module(unsafe { module.get() }, &context)?;
        ret.erase_from_basic_block();

        passes::run_basic(&module);
        module
            .verify()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(f(&module))
    }
}

fn clone_module<'ctx>(
    module: &inkwell::module::Module,
    context: &'ctx inkwell::context::Context,
) -> PyResult<inkwell::module::Module<'ctx>> {
    let name = module
        .get_name()
        .to_str()
        .map_err(PyUnicodeDecodeError::new_err)?;
    let bitcode = module.write_bitcode_to_memory();
    let new_module = inkwell::module::Module::parse_bitcode_from_buffer(&bitcode, context)
        .map_err(|e| {
            module.verify().err().map_or_else(
                || PyOSError::new_err(e.to_string()),
                |e| PyOSError::new_err(e.to_string()),
            )
        })?;
    new_module.set_name(name);
    Ok(new_module)
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    context::{self, Context},
    module::Module,
    types::Type,
    utils::{clone_module, function_type},
    values::Value,
};
use inkwell::types::AnyTypeEnum;
use pyo3::{
    exceptions::{PyOSError, PyValueError},
    prelude::*,
    types::PyBytes,
};
use qirlib::{module, types};
use std::{convert::Into, mem::transmute};

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
    types: Py<TypeFactory>,
    num_qubits: u64,
    num_results: u64,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(py: Python, name: &str, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let context = Py::new(py, Context::new())?;
        let module = Py::new(py, Module::new(py, context, name))?;
        let builder = Py::new(py, Builder::new(py, module.clone()))?;

        {
            let builder = builder.borrow(py);
            let module = module.borrow(py);
            unsafe { module::simple_init(module.get(), builder.get(), num_qubits, num_results) };
        }

        let types = Py::new(
            py,
            TypeFactory {
                module: module.clone(),
            },
        )?;

        Ok(SimpleModule {
            module,
            builder,
            types,
            num_qubits,
            num_results,
        })
    }

    #[getter]
    fn types(&self) -> Py<TypeFactory> {
        self.types.clone()
    }

    /// The global qubit register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn qubits(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let builder = self.builder.borrow(py);
        let module = self.module.borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        (0..self.num_qubits)
            .map(|id| unsafe {
                Value::from_any(py, module.context().clone(), builder.build_qubit(id))
            })
            .collect()
    }

    /// The global result register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn results(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let builder = self.builder.borrow(py);
        let module = self.module.borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        (0..self.num_results)
            .map(|id| unsafe {
                Value::from_any(py, module.context().clone(), builder.build_result(id))
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
}

impl SimpleModule {
    fn emit<T>(&self, py: Python, f: impl Fn(&inkwell::module::Module) -> T) -> PyResult<T> {
        let module = self.module.borrow(py);
        let builder = self.builder.borrow(py);
        let ret = unsafe { builder.get() }.build_return(None);
        let new_context = inkwell::context::Context::create();
        let new_module = clone_module(unsafe { module.get() }, &new_context)?;
        ret.erase_from_basic_block();
        module::simple_finalize(&new_module).map_err(PyOSError::new_err)?;
        Ok(f(&new_module))
    }
}

/// Provides access to all supported types.
#[pyclass]
pub(crate) struct TypeFactory {
    module: Py<Module>,
}

#[pymethods]
impl TypeFactory {
    /// The void type.
    ///
    /// :type: Type
    #[getter]
    fn void(&self, py: Python) -> PyResult<PyObject> {
        self.new_type(py, |m| m.get_context().void_type().into())
    }

    /// The boolean type.
    ///
    /// :type: Type
    #[getter]
    fn bool(&self, py: Python) -> PyResult<PyObject> {
        self.new_type(py, |m| m.get_context().bool_type().into())
    }

    /// An integer type.
    ///
    /// :param int width: The number of bits in the integers.
    /// :returns: The integer type.
    /// :rtype: Type
    #[pyo3(text_signature = "(width)")]
    fn int(&self, py: Python, width: u32) -> PyResult<PyObject> {
        self.new_type(py, |m| m.get_context().custom_width_int_type(width).into())
    }

    /// The double type.
    ///
    /// :type: Type
    #[getter]
    fn double(&self, py: Python) -> PyResult<PyObject> {
        self.new_type(py, |m| m.get_context().f64_type().into())
    }

    /// The qubit type.
    ///
    /// :type: Type
    #[getter]
    fn qubit(&self, py: Python) -> PyResult<PyObject> {
        self.new_type(py, |m| types::qubit(m).into())
    }

    /// The measurement result type.
    ///
    /// :type: Type
    #[getter]
    fn result(&self, py: Python) -> PyResult<PyObject> {
        self.new_type(py, |m| types::result(m).into())
    }

    /// A function type.
    ///
    /// :param Type ret: The return type.
    /// :param List[Type] params: The parameter types.
    /// :returns: The function type.
    /// :rtype: Type
    #[staticmethod]
    #[pyo3(text_signature = "(ret, params)")]
    #[allow(clippy::needless_pass_by_value)]
    fn function(py: Python, ret: &Type, params: Vec<Py<Type>>) -> PyResult<PyObject> {
        context::require_same(
            py,
            params
                .iter()
                .map(|t| t.borrow(py).context().clone())
                .chain([ret.context().clone()]),
        )?;

        let ty = function_type(
            unsafe { &ret.get() },
            params.iter().map(|t| unsafe {
                transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(t.borrow(py).get())
            }),
        )
        .ok_or_else(|| PyValueError::new_err("Invalid return or parameter type."))?;

        unsafe { Type::from_any(py, ret.context().clone(), ty.into()) }
    }
}

impl TypeFactory {
    fn new_type(
        &self,
        py: Python,
        f: impl for<'ctx> Fn(&inkwell::module::Module<'ctx>) -> AnyTypeEnum<'ctx>,
    ) -> PyResult<PyObject> {
        let module = self.module.borrow(py);
        let context = module.context().clone();
        let ty = f(unsafe { module.get() });
        unsafe { Type::from_any(py, context, ty) }
    }
}

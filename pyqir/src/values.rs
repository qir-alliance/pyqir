// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    context::Context,
    instructions::Instruction,
    module::Attribute,
    types::Type,
    utils::{extract_constant, AnyValue},
};
use inkwell::{
    attributes::AttributeLoc,
    types::AnyType,
    values::{AnyValueEnum, FloatValue, FunctionValue, IntValue},
};
use pyo3::{conversion::ToPyObject, exceptions::PyValueError, prelude::*, types::PyBytes};
use qirlib::values;
use std::{
    convert::{Into, TryInto},
    mem::transmute,
};

/// A value.
#[pyclass(subclass, unsendable)]
#[derive(Clone)]
pub(crate) struct Value {
    value: AnyValue<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Value {
    #[getter]
    fn r#type(&self, py: Python) -> PyResult<PyObject> {
        unsafe { Type::from_any(py, self.context.clone(), self.value.ty()) }
    }

    #[getter]
    fn name(&self) -> &str {
        self.value
            .name()
            .to_str()
            .expect("Name is not valid UTF-8.")
    }

    fn __str__(&self) -> String {
        self.value.to_string()
    }

    fn __repr__(&self) -> String {
        format!("<{:?}>", self.value)
    }
}

impl Value {
    pub(crate) unsafe fn from_any<'ctx>(
        py: Python,
        context: Py<Context>,
        value: impl Into<AnyValue<'ctx>>,
    ) -> PyResult<PyObject> {
        let value = transmute::<AnyValue<'_>, AnyValue<'static>>(value.into());
        #[allow(clippy::same_functions_in_if_condition)]
        if let Ok(inst) = value.try_into() {
            Instruction::from_inst(py, context, inst)
        } else if let Ok(block) = value.try_into() {
            let base = PyClassInitializer::from(Self { value, context });
            let block = base.add_subclass(BasicBlock(block));
            Ok(Py::new(py, block)?.to_object(py))
        } else if value.is_const() {
            Constant::from_any(py, context, value)
        } else {
            Ok(Py::new(py, Self { value, context })?.to_object(py))
        }
    }

    pub(crate) unsafe fn init(context: Py<Context>, value: AnyValue) -> PyClassInitializer<Self> {
        let value = transmute::<AnyValue<'_>, AnyValue<'static>>(value);
        PyClassInitializer::from(Self { value, context })
    }

    pub(crate) unsafe fn get(&self) -> AnyValue<'static> {
        self.value
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }
}

#[pyclass(extends = Value, unsendable)]
pub(crate) struct BasicBlock(inkwell::basic_block::BasicBlock<'static>);

#[pymethods]
impl BasicBlock {
    #[getter]
    fn instructions(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let block = slf.0;
        let context = &slf.into_super().context;
        let mut insts = Vec::new();
        let mut inst = block.get_first_instruction();

        while let Some(i) = inst {
            insts.push(unsafe { Instruction::from_inst(py, context.clone(), i) }?);
            inst = i.get_next_instruction();
        }

        Ok(insts)
    }

    #[getter]
    fn terminator(slf: PyRef<Self>, py: Python) -> PyResult<Option<PyObject>> {
        match slf.0.get_terminator() {
            Some(terminator) => {
                let context = slf.into_super().context.clone();
                unsafe { Instruction::from_inst(py, context, terminator) }.map(Some)
            }
            None => Ok(None),
        }
    }
}

#[pyclass(extends = Value, subclass)]
pub(crate) struct Constant;

#[pymethods]
impl Constant {
    #[getter]
    fn is_null(slf: PyRef<Self>) -> bool {
        slf.into_super().value.is_null()
    }
}

impl Constant {
    unsafe fn from_any(py: Python, context: Py<Context>, value: AnyValue) -> PyResult<PyObject> {
        if value.is_const() {
            let value = transmute::<AnyValue<'_>, AnyValue<'static>>(value);
            let base = PyClassInitializer::from(Value { value, context }).add_subclass(Constant);
            match value.try_into() {
                Ok(AnyValueEnum::IntValue(_)) => {
                    Ok(Py::new(py, base.add_subclass(IntConstant))?.to_object(py))
                }
                Ok(AnyValueEnum::FloatValue(_)) => {
                    Ok(Py::new(py, base.add_subclass(FloatConstant))?.to_object(py))
                }
                Ok(AnyValueEnum::FunctionValue(f)) => {
                    Ok(Py::new(py, base.add_subclass(Function(f)))?.to_object(py))
                }
                _ => Ok(Py::new(py, base)?.to_object(py)),
            }
        } else {
            Err(PyValueError::new_err("Value is not constant."))
        }
    }
}

#[pyclass(extends = Constant)]
pub(crate) struct IntConstant;

#[pymethods]
impl IntConstant {
    #[getter]
    fn value(slf: PyRef<Self>) -> u64 {
        let int: IntValue = slf.into_super().into_super().value.try_into().unwrap();
        int.get_zero_extended_constant().unwrap()
    }
}

#[pyclass(extends = Constant)]
pub(crate) struct FloatConstant;

#[pymethods]
impl FloatConstant {
    #[getter]
    fn value(slf: PyRef<Self>) -> f64 {
        let float: FloatValue = slf.into_super().into_super().value.try_into().unwrap();
        float.get_constant().unwrap().0
    }
}

#[pyclass(extends = Constant, unsendable)]
pub(crate) struct Function(FunctionValue<'static>);

#[pymethods]
impl Function {
    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let params = slf.0.get_params();
        let context = &slf.into_super().into_super().context;
        params
            .into_iter()
            .map(|p| unsafe { Value::from_any(py, context.clone(), p) })
            .collect()
    }

    #[getter]
    fn basic_blocks(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let function = slf.0;
        let context = &slf.into_super().into_super().context;
        function
            .get_basic_blocks()
            .into_iter()
            .map(|b| unsafe { Value::from_any(py, context.clone(), b) })
            .collect()
    }

    fn attribute(&self, name: &str) -> Option<Attribute> {
        Some(Attribute(
            self.0.get_string_attribute(AttributeLoc::Function, name)?,
        ))
    }
}

impl Function {
    pub(crate) unsafe fn get(&self) -> FunctionValue<'static> {
        self.0
    }
}

/// Creates a constant value.
///
/// :param Type ty: The type of the value.
/// :param Union[int, float] value: The value of the constant.
/// :returns: The constant value.
/// :rtype: Value
#[pyfunction]
#[pyo3(text_signature = "(ty, value)")]
pub(crate) fn r#const(py: Python, ty: &Type, value: &PyAny) -> PyResult<PyObject> {
    let context = ty.context().clone();
    let value = extract_constant(unsafe { &ty.get() }, value)?;
    unsafe { Value::from_any(py, context, value) }
}

#[pyfunction]
pub(crate) fn qubit_id(value: &Value) -> Option<u64> {
    values::qubit_id(unsafe { value.get() }.try_into().ok()?)
}

#[pyfunction]
pub(crate) fn result_id(value: &Value) -> Option<u64> {
    values::result_id(unsafe { value.get() }.try_into().ok()?)
}

#[pyfunction]
pub(crate) fn is_entry_point(function: &Function) -> bool {
    values::is_entry_point(unsafe { function.get() })
}

#[pyfunction]
pub(crate) fn is_interop_friendly(function: &Function) -> bool {
    values::is_interop_friendly(unsafe { function.get() })
}

#[pyfunction]
pub(crate) fn required_num_qubits(function: &Function) -> Option<u64> {
    values::required_num_qubits(unsafe { function.get() })
}

#[pyfunction]
pub(crate) fn required_num_results(function: &Function) -> Option<u64> {
    values::required_num_results(unsafe { function.get() })
}

#[pyfunction]
pub(crate) fn constant_bytes<'p>(py: Python<'p>, value: &Value) -> Option<&'p PyBytes> {
    let bytes = values::constant_bytes(unsafe { value.get() }.try_into().ok()?)?;
    Some(PyBytes::new(py, bytes))
}

pub(crate) unsafe fn extract_any<'ctx>(
    ty: &impl AnyType<'ctx>,
    ob: &PyAny,
) -> PyResult<AnyValue<'ctx>> {
    ob.extract()
        .map(|v: Value| v.value)
        .or_else(|_| extract_constant(ty, ob))
}

pub(crate) fn extract_contexts<'a>(
    values: impl IntoIterator<Item = &'a PyAny> + 'a,
) -> impl Iterator<Item = Py<Context>> + 'a {
    values
        .into_iter()
        .filter_map(|v| Some(v.extract::<Value>().ok()?.context))
}

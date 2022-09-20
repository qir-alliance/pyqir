#![allow(clippy::borrow_deref_ref)]
#![allow(clippy::used_underscore_binding)]

use crate::python::Context;
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
};
use qirlib::inkwell::types::{
    AnyTypeEnum, ArrayType, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType,
    PointerType, StructType, VoidType,
};
use std::{
    convert::{TryFrom, TryInto},
    mem::transmute,
};

#[pyclass(subclass)]
#[derive(Clone)]
pub(crate) struct Type {
    pub(crate) context: Py<Context>,
}

#[pyclass(extends = Type, unsendable)]
#[derive(Clone)]
pub(crate) struct Void(VoidType<'static>);

#[pymethods]
impl Void {
    #[new]
    fn new(py: Python, context: Py<Context>) -> (Self, Type) {
        let ty = {
            let context = context.borrow(py);
            let ty = context.void_type();
            unsafe { transmute::<VoidType, VoidType<'static>>(ty) }
        };

        (Self(ty), Type { context })
    }
}

#[pyclass(extends = Type, unsendable)]
#[derive(Clone)]
pub(crate) struct Integer(IntType<'static>);

#[pymethods]
impl Integer {
    #[new]
    fn new(py: Python, context: Py<Context>, width: u32) -> (Self, Type) {
        let ty = {
            let context = context.borrow(py);
            let ty = context.custom_width_int_type(width);
            unsafe { transmute::<IntType, IntType<'static>>(ty) }
        };

        (Self(ty), Type { context })
    }
}

#[pyclass(extends = Type, unsendable)]
#[derive(Clone)]
pub(crate) struct Double(FloatType<'static>);

#[pymethods]
impl Double {
    #[new]
    fn new(py: Python, context: Py<Context>) -> (Self, Type) {
        let ty = {
            let context = context.borrow(py);
            let ty = context.f64_type();
            unsafe { transmute::<FloatType, FloatType<'static>>(ty) }
        };

        (Self(ty), Type { context })
    }
}

#[pyclass(extends = Type, unsendable)]
#[derive(Clone)]
pub(crate) struct Function(FunctionType<'static>);

#[pymethods]
impl Function {
    #[new]
    #[allow(clippy::needless_pass_by_value)]
    fn new(py: Python, return_: Py<Type>, params: Vec<Py<Type>>) -> PyResult<(Self, Type)> {
        let ty = {
            let params = params
                .iter()
                .map(|ty| {
                    let ty = any_type_enum(py, ty)?;
                    let ty = BasicTypeEnum::try_from(ty)
                        .map_err(|()| PyTypeError::new_err("Invalid parameter type."))?;
                    Ok(ty.into())
                })
                .collect::<PyResult<Vec<_>>>()?;

            let ty = match any_type_enum(py, &return_)? {
                AnyTypeEnum::VoidType(void) => void.fn_type(&params, false),
                ret => BasicTypeEnum::try_from(ret)
                    .expect("Invalid return type.")
                    .fn_type(&params, false),
            };

            unsafe { transmute::<FunctionType, FunctionType<'static>>(ty) }
        };

        // TODO (safety): What if not all types use the same context?
        let context = return_.borrow(py).context.clone();
        Ok((Self(ty), Type { context }))
    }
}

#[pyclass(extends = Type, unsendable)]
#[derive(Clone)]
pub(crate) struct Struct(StructType<'static>);

#[pymethods]
impl Struct {
    #[staticmethod]
    fn opaque(py: Python, context: Py<Context>, name: &str) -> PyResult<Py<Self>> {
        let ty = {
            let context = context.borrow(py);
            let ty = context.opaque_struct_type(name);
            unsafe { transmute::<StructType, StructType<'static>>(ty) }
        };

        Py::new(
            py,
            PyClassInitializer::from(Type { context }).add_subclass(Struct(ty)),
        )
    }
}

#[pyclass(extends = Type, unsendable)]
#[derive(Clone)]
pub(crate) struct Array(ArrayType<'static>);

#[pymethods]
impl Array {
    #[new]
    #[allow(clippy::needless_pass_by_value)]
    fn new(py: Python, element: Py<Type>, count: u32) -> PyResult<(Self, Type)> {
        let ty = {
            let element = any_type_enum(py, &element)?;
            let element = BasicTypeEnum::try_from(element)
                .map_err(|()| PyTypeError::new_err("Invalid element type."))?;
            let ty = element.array_type(count);
            unsafe { transmute::<ArrayType, ArrayType<'static>>(ty) }
        };

        let context = element.borrow(py).context.clone();
        Ok((Self(ty), Type { context }))
    }
}

#[pyclass(extends = Type, unsendable)]
#[derive(Clone)]
pub(crate) struct Pointer(PointerType<'static>);

#[pymethods]
impl Pointer {
    #[new]
    #[allow(clippy::needless_pass_by_value)]
    fn new(py: Python, pointee: Py<Type>, address_space: u32) -> PyResult<(Self, Type)> {
        let ty = {
            let pointee = any_type_enum(py, &pointee)?;
            let pointee = BasicTypeEnum::try_from(pointee)
                .map_err(|()| PyTypeError::new_err("Invalid pointee type."))?;
            let address_space = address_space
                .try_into()
                .map_err(|()| PyValueError::new_err("Invalid address space."))?;
            let ty = pointee.ptr_type(address_space);
            unsafe { transmute::<PointerType, PointerType<'static>>(ty) }
        };

        let context = pointee.borrow(py).context.clone();
        Ok((Self(ty), Type { context }))
    }
}

pub(crate) fn any_type_enum(py: Python, ty: &Py<Type>) -> PyResult<AnyTypeEnum<'static>> {
    if let Ok(void) = ty.extract::<Void>(py) {
        return Ok(AnyTypeEnum::VoidType(void.0));
    }
    if let Ok(integer) = ty.extract::<Integer>(py) {
        return Ok(AnyTypeEnum::IntType(integer.0));
    }
    if let Ok(double) = ty.extract::<Double>(py) {
        return Ok(AnyTypeEnum::FloatType(double.0));
    }
    if let Ok(function) = ty.extract::<Function>(py) {
        return Ok(AnyTypeEnum::FunctionType(function.0));
    }
    if let Ok(structure) = ty.extract::<Struct>(py) {
        return Ok(AnyTypeEnum::StructType(structure.0));
    }
    if let Ok(array) = ty.extract::<Array>(py) {
        return Ok(AnyTypeEnum::ArrayType(array.0));
    }
    if let Ok(pointer) = ty.extract::<Pointer>(py) {
        return Ok(AnyTypeEnum::PointerType(pointer.0));
    }
    Err(PyTypeError::new_err("Invalid type."))
}

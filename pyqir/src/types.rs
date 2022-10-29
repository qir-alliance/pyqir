// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{context::Context, utils::basic_to_any};
use inkwell::types::AnyTypeEnum;
use pyo3::{conversion::ToPyObject, prelude::*};
use qirlib::types;
use std::mem::transmute;

/// A type.
#[pyclass(subclass, unsendable)]
pub(crate) struct Type {
    ty: AnyTypeEnum<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Type {
    #[getter]
    fn is_void(&self) -> bool {
        self.ty.is_void_type()
    }

    #[getter]
    fn is_double(&self) -> bool {
        match self.ty {
            AnyTypeEnum::FloatType(float) => {
                float.size_of().get_zero_extended_constant() == Some(64)
            }
            _ => false,
        }
    }
}

impl Type {
    pub(crate) unsafe fn from_any(
        py: Python,
        context: Py<Context>,
        ty: AnyTypeEnum,
    ) -> PyResult<PyObject> {
        let ty = transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty);
        let base = PyClassInitializer::from(Self { ty, context });
        Ok(match ty {
            AnyTypeEnum::ArrayType(a) => {
                Py::new(py, base.add_subclass(ArrayType(a)))?.to_object(py)
            }
            AnyTypeEnum::FunctionType(f) => {
                Py::new(py, base.add_subclass(FunctionType(f)))?.to_object(py)
            }
            AnyTypeEnum::IntType(i) => Py::new(py, base.add_subclass(IntType(i)))?.to_object(py),
            AnyTypeEnum::PointerType(p) => {
                Py::new(py, base.add_subclass(PointerType(p)))?.to_object(py)
            }
            AnyTypeEnum::StructType(s) => {
                Py::new(py, base.add_subclass(StructType(s)))?.to_object(py)
            }
            AnyTypeEnum::FloatType(_) | AnyTypeEnum::VectorType(_) | AnyTypeEnum::VoidType(_) => {
                Py::new(py, base)?.to_object(py)
            }
        })
    }

    pub(crate) fn get(&self) -> AnyTypeEnum {
        self.ty
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }
}

#[pyclass(extends = Type, unsendable)]
pub(crate) struct IntType(inkwell::types::IntType<'static>);

#[pymethods]
impl IntType {
    #[getter]
    fn width(&self) -> u32 {
        self.0.get_bit_width()
    }
}

#[pyclass(extends = Type, unsendable)]
pub(crate) struct FunctionType(inkwell::types::FunctionType<'static>);

#[pymethods]
impl FunctionType {
    #[getter]
    fn return_(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ty = basic_to_any(slf.0.get_return_type().unwrap());
        let context = slf.into_super().context.clone();
        unsafe { Type::from_any(py, context, ty) }
    }

    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let params = slf.0.get_param_types();
        let context = &slf.into_super().context;
        params
            .into_iter()
            .map(|ty| unsafe { Type::from_any(py, context.clone(), basic_to_any(ty)) })
            .collect()
    }
}

#[pyclass(extends = Type, unsendable)]
pub(crate) struct StructType(inkwell::types::StructType<'static>);

#[pymethods]
impl StructType {
    #[getter]
    fn name(&self) -> Option<&str> {
        self.0
            .get_name()
            .map(|n| n.to_str().expect("Name is not valid UTF-8."))
    }

    #[getter]
    fn fields(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let fields = slf.0.get_field_types();
        let context = &slf.into_super().context;
        fields
            .into_iter()
            .map(|ty| unsafe { Type::from_any(py, context.clone(), basic_to_any(ty)) })
            .collect()
    }
}

#[pyclass(extends = Type, unsendable)]
pub(crate) struct ArrayType(inkwell::types::ArrayType<'static>);

#[pymethods]
impl ArrayType {
    #[getter]
    fn element(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ty = basic_to_any(slf.0.get_element_type());
        let context = slf.into_super().context.clone();
        unsafe { Type::from_any(py, context, ty) }
    }

    #[getter]
    fn count(&self) -> u32 {
        self.0.len()
    }
}

#[pyclass(extends = Type, unsendable)]
pub(crate) struct PointerType(inkwell::types::PointerType<'static>);

#[pymethods]
impl PointerType {
    #[getter]
    fn pointee(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ty = slf.0.get_element_type();
        let context = slf.into_super().context.clone();
        unsafe { Type::from_any(py, context, ty) }
    }

    #[getter]
    fn address_space(&self) -> u32 {
        self.0.get_address_space() as u32
    }
}

#[pyfunction]
pub(crate) fn is_qubit(ty: &Type) -> bool {
    types::is_qubit(ty.ty)
}

#[pyfunction]
pub(crate) fn is_result(ty: &Type) -> bool {
    types::is_result(ty.ty)
}

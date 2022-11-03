// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::context::Context;
use inkwell::types::{AnyTypeEnum, BasicTypeEnum};
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
    /// Whether this type is the void type.
    ///
    /// :type: bool
    #[getter]
    fn is_void(&self) -> bool {
        self.ty.is_void_type()
    }

    /// Whether this type is the bool type.
    ///
    /// :type: bool
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

    pub(crate) unsafe fn get(&self) -> AnyTypeEnum<'static> {
        self.ty
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }
}

/// An integer type.
#[pyclass(extends = Type, unsendable)]
pub(crate) struct IntType(inkwell::types::IntType<'static>);

#[pymethods]
impl IntType {
    /// The number of bits in the integer.
    ///
    /// :type: int
    #[getter]
    fn width(&self) -> u32 {
        self.0.get_bit_width()
    }
}

/// A function type.
#[pyclass(extends = Type, unsendable)]
pub(crate) struct FunctionType(inkwell::types::FunctionType<'static>);

#[pymethods]
impl FunctionType {
    /// The return type of the function.
    ///
    /// :type: Type
    #[getter]
    fn ret(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ty = basic_to_any(slf.0.get_return_type().unwrap());
        let context = slf.into_super().context.clone();
        unsafe { Type::from_any(py, context, ty) }
    }

    /// The types of the function parameters.
    ///
    /// :type: List[Type]
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

/// A structure type.
#[pyclass(extends = Type, unsendable)]
pub(crate) struct StructType(inkwell::types::StructType<'static>);

#[pymethods]
impl StructType {
    /// The name of the structure or the empty string if the structure is anonymous.
    #[getter]
    fn name(&self) -> Option<&str> {
        self.0
            .get_name()
            .map(|n| n.to_str().expect("Name is not valid UTF-8."))
    }

    /// The types of the structure fields.
    ///
    /// :type: List[Type]
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

/// An array type.
#[pyclass(extends = Type, unsendable)]
pub(crate) struct ArrayType(inkwell::types::ArrayType<'static>);

#[pymethods]
impl ArrayType {
    /// The type of the array elements.
    ///
    /// :type: Type
    #[getter]
    fn element(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ty = basic_to_any(slf.0.get_element_type());
        let context = slf.into_super().context.clone();
        unsafe { Type::from_any(py, context, ty) }
    }

    /// The number of elements in the a rray.
    ///
    /// :type: int
    #[getter]
    fn count(&self) -> u32 {
        self.0.len()
    }
}

/// A pointer type.
#[pyclass(extends = Type, unsendable)]
pub(crate) struct PointerType(inkwell::types::PointerType<'static>);

#[pymethods]
impl PointerType {
    /// The type being pointed to.
    ///
    /// :type: Type
    #[getter]
    fn pointee(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ty = slf.0.get_element_type();
        let context = slf.into_super().context.clone();
        unsafe { Type::from_any(py, context, ty) }
    }

    /// The pointer address space.
    ///
    /// :type: int
    #[getter]
    fn address_space(&self) -> u32 {
        self.0.get_address_space() as u32
    }
}

/// Whether the type is the QIR qubit type.
///
/// :param Type ty: The type.
/// :rtype: bool
/// :returns: True if the type is the QIR qubit type.
#[pyfunction]
pub(crate) fn is_qubit(ty: &Type) -> bool {
    types::is_qubit(ty.ty)
}

/// Whether the type is the QIR result type.
///
/// :param Type ty: The type.
/// :rtype: bool
/// :returns: True if the type is the QIR result type.
#[pyfunction]
pub(crate) fn is_result(ty: &Type) -> bool {
    types::is_result(ty.ty)
}

fn basic_to_any(ty: BasicTypeEnum) -> AnyTypeEnum {
    match ty {
        BasicTypeEnum::ArrayType(a) => a.into(),
        BasicTypeEnum::FloatType(f) => f.into(),
        BasicTypeEnum::IntType(i) => i.into(),
        BasicTypeEnum::PointerType(p) => p.into(),
        BasicTypeEnum::StructType(s) => s.into(),
        BasicTypeEnum::VectorType(v) => v.into(),
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::context::{self, Context};
use inkwell::{
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum},
    AddressSpace,
};
use pyo3::{conversion::ToPyObject, exceptions::PyValueError, prelude::*};
use qirlib::types;
use std::{convert::TryFrom, mem::transmute};

/// A type.
#[pyclass(subclass, unsendable)]
#[derive(Clone)]
pub(crate) struct Type {
    ty: AnyTypeEnum<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Type {
    #[staticmethod]
    fn void(py: Python, context: Py<Context>) -> Self {
        let ty = {
            let context = context.borrow(py);
            let ty = context.void_type().into();
            unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty) }
        };
        Type { ty, context }
    }

    #[staticmethod]
    fn double(py: Python, context: Py<Context>) -> Self {
        let ty = {
            let context = context.borrow(py);
            let ty = context.f64_type().into();
            unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty) }
        };
        Type { ty, context }
    }

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
    #[new]
    fn new(py: Python, context: Py<Context>, width: u32) -> (Self, Type) {
        let ty = {
            let context = context.borrow(py);
            let ty = context.custom_width_int_type(width);
            unsafe {
                transmute::<inkwell::types::IntType<'_>, inkwell::types::IntType<'static>>(ty)
            }
        };

        (
            Self(ty),
            Type {
                ty: ty.into(),
                context,
            },
        )
    }

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
#[derive(Clone)]
pub(crate) struct FunctionType(inkwell::types::FunctionType<'static>);

#[pymethods]
impl FunctionType {
    #[new]
    #[allow(clippy::needless_pass_by_value)]
    fn new(py: Python, ret: &Type, params: Vec<Type>) -> PyResult<(Self, Type)> {
        context::require_same(
            py,
            params.iter().map(|ty| &ty.context).chain([&ret.context]),
        )?;

        let ty = function(&ret.ty, params.iter().map(|ty| ty.ty))
            .ok_or_else(|| PyValueError::new_err("Not a valid function type."))?;

        Ok((
            Self(ty),
            Type {
                ty: ty.into(),
                context: ret.context.clone(),
            },
        ))
    }

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

impl FunctionType {
    pub(crate) unsafe fn get(&self) -> inkwell::types::FunctionType<'static> {
        self.0
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
    #[new]
    fn new(pointee: &Type) -> PyResult<(Self, Type)> {
        let ty = match pointee.ty {
            AnyTypeEnum::ArrayType(a) => Ok(a.ptr_type(AddressSpace::Generic)),
            AnyTypeEnum::FloatType(f) => Ok(f.ptr_type(AddressSpace::Generic)),
            AnyTypeEnum::FunctionType(f) => Ok(f.ptr_type(AddressSpace::Generic)),
            AnyTypeEnum::IntType(i) => Ok(i.ptr_type(AddressSpace::Generic)),
            AnyTypeEnum::PointerType(p) => Ok(p.ptr_type(AddressSpace::Generic)),
            AnyTypeEnum::StructType(s) => Ok(s.ptr_type(AddressSpace::Generic)),
            AnyTypeEnum::VectorType(v) => Ok(v.ptr_type(AddressSpace::Generic)),
            AnyTypeEnum::VoidType(_) => Err(PyValueError::new_err("Pointer to void type.")),
        }?;

        Ok((
            Self(ty),
            Type {
                ty: ty.into(),
                context: pointee.context.clone(),
            },
        ))
    }

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
#[pyo3(text_signature = "(ty)")]
pub(crate) fn is_qubit(ty: &Type) -> bool {
    types::is_qubit(ty.ty)
}

/// Whether the type is the QIR result type.
///
/// :param Type ty: The type.
/// :rtype: bool
/// :returns: True if the type is the QIR result type.
#[pyfunction]
#[pyo3(text_signature = "(ty)")]
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

pub(crate) fn function<'ctx>(
    ret: &impl AnyType<'ctx>,
    params: impl IntoIterator<Item = AnyTypeEnum<'ctx>>,
) -> Option<inkwell::types::FunctionType<'ctx>> {
    let params = params
        .into_iter()
        .map(|ty| BasicTypeEnum::try_from(ty).map(Into::into).ok())
        .collect::<Option<Vec<_>>>()?;

    match ret.as_any_type_enum() {
        AnyTypeEnum::VoidType(void) => Some(void.fn_type(&params, false)),
        any => BasicTypeEnum::try_from(any)
            .map(|basic| basic.fn_type(&params, false))
            .ok(),
    }
}

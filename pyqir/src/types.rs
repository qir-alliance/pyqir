// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{context::Context, values::Owner};
use inkwell::{
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum},
    AddressSpace, LLVMReference,
};
use llvm_sys::{core::LLVMGetTypeKind, LLVMTypeKind};
use pyo3::{conversion::ToPyObject, exceptions::PyValueError, prelude::*};
use qirlib::types;
use std::{convert::TryFrom, mem::transmute};

/// A type.
#[pyclass(subclass, unsendable)]
pub(crate) struct Type {
    ty: AnyTypeEnum<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Type {
    /// The void type.
    ///
    /// :param Context context: The global context.
    /// :returns: The void type.
    /// :rtype: Type
    #[staticmethod]
    #[pyo3(text_signature = "(context)")]
    fn void(py: Python, context: Py<Context>) -> Self {
        let ty = {
            let context = context.borrow(py);
            let ty = context.void_type().into();
            unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty) }
        };
        Type { ty, context }
    }

    /// The double type.
    ///
    /// :param Context context: The global context.
    /// :returns: The double type.
    /// :rtype: Type
    #[staticmethod]
    #[pyo3(text_signature = "(context)")]
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
        (unsafe { LLVMGetTypeKind(self.ty.get_ref()) }) == LLVMTypeKind::LLVMDoubleTypeKind
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
///
/// :param Context context: The global context.
/// :param int width: The number of bits in the integer.
#[pyclass(extends = Type, unsendable)]
#[pyo3(text_signature = "(context, width)")]
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
///
/// :param Type ret: The return type.
/// :param typing.Sequence[Type] params: The parameter types.
#[pyclass(extends = Type, unsendable)]
#[pyo3(text_signature = "(ret, params)")]
pub(crate) struct FunctionType(inkwell::types::FunctionType<'static>);

#[pymethods]
impl FunctionType {
    #[new]
    fn new(py: Python, ret: &Type, params: Vec<PyRef<Type>>) -> PyResult<(Self, Type)> {
        Owner::merge(
            py,
            params
                .iter()
                .map(|ty| Owner::Context(ty.context.clone_ref(py)))
                .chain([ret.context.clone_ref(py).into()]),
        )?;

        let ty = function(&ret.ty, params.iter().map(|ty| ty.ty))
            .ok_or_else(|| PyValueError::new_err("Not a valid function type."))?;

        Ok((
            Self(ty),
            Type {
                ty: ty.into(),
                context: ret.context.clone_ref(py),
            },
        ))
    }

    /// The return type of the function.
    ///
    /// :type: Type
    #[getter]
    fn ret(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ret = slf.0.get_return_type();
        let context = slf.into_super().context.clone_ref(py);
        match ret {
            None => Ok(Py::new(py, Type::void(py, context))?.to_object(py)),
            Some(ret) => unsafe { Type::from_any(py, context, basic_to_any(ret)) },
        }
    }

    /// The types of the function parameters.
    ///
    /// :type: typing.List[Type]
    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let params = slf.0.get_param_types();
        let context = &slf.into_super().context;
        params
            .into_iter()
            .map(|ty| unsafe { Type::from_any(py, context.clone_ref(py), basic_to_any(ty)) })
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
    /// :type: typing.List[Type]
    #[getter]
    fn fields(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let fields = slf.0.get_field_types();
        let context = &slf.into_super().context;
        fields
            .into_iter()
            .map(|ty| unsafe { Type::from_any(py, context.clone_ref(py), basic_to_any(ty)) })
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
        let context = slf.into_super().context.clone_ref(py);
        unsafe { Type::from_any(py, context, ty) }
    }

    /// The number of elements in the array.
    ///
    /// :type: int
    #[getter]
    fn count(&self) -> u32 {
        self.0.len()
    }
}

/// A pointer type.
///
/// :param Type pointee: The type being pointed to.
#[pyclass(extends = Type, unsendable)]
#[pyo3(text_signature = "(pointee)")]
pub(crate) struct PointerType(inkwell::types::PointerType<'static>);

#[pymethods]
impl PointerType {
    #[new]
    fn new(py: Python, pointee: &Type) -> PyResult<(Self, Type)> {
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
                context: pointee.context.clone_ref(py),
            },
        ))
    }

    /// The type being pointed to.
    ///
    /// :type: Type
    #[getter]
    fn pointee(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let ty = slf.0.get_element_type();
        let context = slf.into_super().context.clone_ref(py);
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

/// The QIR qubit type.
///
/// :param Context context: The global context.
/// :returns: The qubit type.
/// :rtype: Type
#[pyfunction]
#[pyo3(text_signature = "(context)")]
pub(crate) fn qubit_type(py: Python, context: Py<Context>) -> PyResult<PyObject> {
    let ty = {
        let context = context.borrow(py);
        let ty = types::qubit(&context.void_type().get_context()).into();
        unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty) }
    };
    unsafe { Type::from_any(py, context, ty) }
}

/// Whether the type is the QIR qubit type.
///
/// :param Type ty: The type.
/// :rtype: bool
/// :returns: True if the type is the QIR qubit type.
#[pyfunction]
#[pyo3(text_signature = "(ty)")]
pub(crate) fn is_qubit_type(ty: &Type) -> bool {
    types::is_qubit(ty.ty)
}

/// The QIR result type.
///
/// :param Context context: The global context.
/// :returns: The result type.
/// :rtype: Type
#[pyfunction]
#[pyo3(text_signature = "(context)")]
pub(crate) fn result_type(py: Python, context: Py<Context>) -> PyResult<PyObject> {
    let ty = {
        let context = context.borrow(py);
        let ty = types::result(&context.void_type().get_context()).into();
        unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty) }
    };
    unsafe { Type::from_any(py, context, ty) }
}

/// Whether the type is the QIR result type.
///
/// :param Type ty: The type.
/// :rtype: bool
/// :returns: True if the type is the QIR result type.
#[pyfunction]
#[pyo3(text_signature = "(ty)")]
pub(crate) fn is_result_type(ty: &Type) -> bool {
    types::is_result(ty.ty)
}

pub(crate) fn basic_to_any(ty: BasicTypeEnum) -> AnyTypeEnum {
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

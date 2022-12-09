// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{context::Context, values::Owner};
use inkwell::LLVMReference;
use llvm_sys::{
    core::{
        LLVMCountParamTypes, LLVMCountStructElementTypes, LLVMDoubleTypeInContext,
        LLVMFunctionType, LLVMGetArrayLength, LLVMGetElementType, LLVMGetIntTypeWidth,
        LLVMGetParamTypes, LLVMGetPointerAddressSpace, LLVMGetReturnType,
        LLVMGetStructElementTypes, LLVMGetStructName, LLVMGetTypeKind, LLVMIntTypeInContext,
        LLVMPointerType, LLVMVoidTypeInContext,
    },
    prelude::*,
    LLVMTypeKind,
};
use pyo3::{conversion::ToPyObject, prelude::*};
use qirlib::types;
use std::{ffi::CStr, ops::Deref};

/// A type.
#[pyclass(subclass, unsendable)]
pub(crate) struct Type {
    ty: LLVMTypeRef,
    context: Py<Context>,
}

#[pymethods]
impl Type {
    /// The void type.
    ///
    /// :param Context context: The LLVM context.
    /// :returns: The void type.
    /// :rtype: Type
    #[staticmethod]
    #[pyo3(text_signature = "(context)")]
    fn void(py: Python, context: Py<Context>) -> Self {
        let ty = {
            let context = context.borrow(py);
            unsafe { LLVMVoidTypeInContext(context.get_ref()) }
        };
        Type { ty, context }
    }

    /// The double type.
    ///
    /// :param Context context: The LLVM context.
    /// :returns: The double type.
    /// :rtype: Type
    #[staticmethod]
    #[pyo3(text_signature = "(context)")]
    fn double(py: Python, context: Py<Context>) -> Self {
        let ty = {
            let context = context.borrow(py);
            unsafe { LLVMDoubleTypeInContext(context.get_ref()) }
        };
        Type { ty, context }
    }

    /// Whether this type is the void type.
    ///
    /// :type: bool
    #[getter]
    fn is_void(&self) -> bool {
        unsafe { LLVMGetTypeKind(self.ty) == LLVMTypeKind::LLVMVoidTypeKind }
    }

    /// Whether this type is the bool type.
    ///
    /// :type: bool
    #[getter]
    fn is_double(&self) -> bool {
        unsafe { LLVMGetTypeKind(self.ty) == LLVMTypeKind::LLVMDoubleTypeKind }
    }
}

impl Type {
    pub(crate) unsafe fn from_ptr(
        py: Python,
        context: Py<Context>,
        ty: LLVMTypeRef,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Self { ty, context });
        match LLVMGetTypeKind(ty) {
            LLVMTypeKind::LLVMArrayTypeKind => {
                Ok(Py::new(py, base.add_subclass(ArrayType))?.to_object(py))
            }
            LLVMTypeKind::LLVMFunctionTypeKind => {
                Ok(Py::new(py, base.add_subclass(FunctionType))?.to_object(py))
            }
            LLVMTypeKind::LLVMIntegerTypeKind => {
                Ok(Py::new(py, base.add_subclass(IntType))?.to_object(py))
            }
            LLVMTypeKind::LLVMPointerTypeKind => {
                Ok(Py::new(py, base.add_subclass(PointerType))?.to_object(py))
            }
            LLVMTypeKind::LLVMStructTypeKind => {
                Ok(Py::new(py, base.add_subclass(StructType))?.to_object(py))
            }
            _ => Ok(Py::new(py, base)?.to_object(py)),
        }
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }
}

impl Deref for Type {
    type Target = LLVMTypeRef;

    fn deref(&self) -> &Self::Target {
        &self.ty
    }
}

/// An integer type.
///
/// :param Context context: The LLVM context.
/// :param int width: The number of bits in the integer.
#[pyclass(extends = Type)]
#[pyo3(text_signature = "(context, width)")]
pub(crate) struct IntType;

#[pymethods]
impl IntType {
    #[new]
    fn new(py: Python, context: Py<Context>, width: u32) -> (Self, Type) {
        let ty = {
            let context = context.borrow(py);
            unsafe { LLVMIntTypeInContext(context.get_ref(), width) }
        };
        (Self, Type { ty, context })
    }

    /// The number of bits in the integer.
    ///
    /// :type: int
    #[getter]
    fn width(slf: PyRef<Self>) -> u32 {
        unsafe { LLVMGetIntTypeWidth(slf.into_super().ty) }
    }
}

/// A function type.
///
/// :param Type ret: The return type.
/// :param typing.Sequence[Type] params: The parameter types.
#[pyclass(extends = Type)]
#[pyo3(text_signature = "(ret, params)")]
pub(crate) struct FunctionType;

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

        let mut params: Vec<_> = params.iter().map(|ty| ty.ty).collect();
        let ty = unsafe {
            LLVMFunctionType(
                **ret,
                params.as_mut_ptr(),
                params.len().try_into().unwrap(),
                0,
            )
        };

        Ok((
            Self,
            Type {
                ty,
                context: ret.context.clone_ref(py),
            },
        ))
    }

    /// The return type of the function.
    ///
    /// :type: Type
    #[getter]
    fn ret(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let slf = slf.into_super();
        let context = slf.context.clone_ref(py);
        unsafe { Type::from_ptr(py, context, LLVMGetReturnType(slf.ty)) }
    }

    /// The types of the function parameters.
    ///
    /// :type: typing.List[Type]
    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let slf = slf.into_super();
        unsafe {
            let count = LLVMCountParamTypes(slf.ty).try_into().unwrap();
            let mut params = Vec::with_capacity(count);
            LLVMGetParamTypes(slf.ty, params.as_mut_ptr());
            params.set_len(count);
            params
                .into_iter()
                .map(|ty| Type::from_ptr(py, slf.context.clone_ref(py), ty))
                .collect()
        }
    }
}

/// A structure type.
#[pyclass(extends = Type)]
pub(crate) struct StructType;

#[pymethods]
impl StructType {
    /// The name of the structure or the empty string if the structure is anonymous.
    #[getter]
    fn name(slf: PyRef<Self>) -> Option<&str> {
        unsafe {
            let name = LLVMGetStructName(slf.into_super().ty);
            if name.is_null() {
                None
            } else {
                Some(CStr::from_ptr(name).to_str().unwrap())
            }
        }
    }

    /// The types of the structure fields.
    ///
    /// :type: typing.List[Type]
    #[getter]
    fn fields(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let slf = slf.into_super();
        unsafe {
            let count = LLVMCountStructElementTypes(slf.ty).try_into().unwrap();
            let mut fields = Vec::with_capacity(count);
            LLVMGetStructElementTypes(slf.ty, fields.as_mut_ptr());
            fields.set_len(count);
            fields
                .into_iter()
                .map(|ty| Type::from_ptr(py, slf.context.clone_ref(py), ty))
                .collect()
        }
    }
}

/// An array type.
#[pyclass(extends = Type, unsendable)]
pub(crate) struct ArrayType;

#[pymethods]
impl ArrayType {
    /// The type of the array elements.
    ///
    /// :type: Type
    #[getter]
    fn element(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let slf = slf.into_super();
        unsafe { Type::from_ptr(py, slf.context.clone_ref(py), LLVMGetElementType(slf.ty)) }
    }

    /// The number of elements in the array.
    ///
    /// :type: int
    #[getter]
    fn count(slf: PyRef<Self>) -> u32 {
        unsafe { LLVMGetArrayLength(slf.into_super().ty) }
    }
}

/// A pointer type.
///
/// :param Type pointee: The type being pointed to.
#[pyclass(extends = Type)]
#[pyo3(text_signature = "(pointee)")]
pub(crate) struct PointerType;

#[pymethods]
impl PointerType {
    #[new]
    fn new(py: Python, pointee: &Type) -> (Self, Type) {
        (
            Self,
            Type {
                ty: unsafe { LLVMPointerType(pointee.ty, 0) },
                context: pointee.context.clone_ref(py),
            },
        )
    }

    /// The type being pointed to.
    ///
    /// :type: Type
    #[getter]
    fn pointee(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let slf = slf.into_super();
        unsafe { Type::from_ptr(py, slf.context.clone_ref(py), LLVMGetElementType(slf.ty)) }
    }

    /// The pointer address space.
    ///
    /// :type: int
    #[getter]
    fn address_space(slf: PyRef<Self>) -> u32 {
        unsafe { LLVMGetPointerAddressSpace(slf.into_super().ty) }
    }
}

/// The QIR qubit type.
///
/// :param Context context: The LLVM context.
/// :returns: The qubit type.
/// :rtype: Type
#[pyfunction]
#[pyo3(text_signature = "(context)")]
pub(crate) fn qubit_type(py: Python, context: Py<Context>) -> PyResult<PyObject> {
    unsafe {
        let ty = types::qubit(context.borrow(py).get_ref());
        Type::from_ptr(py, context, ty)
    }
}

/// Whether the type is the QIR qubit type.
///
/// :param Type ty: The type.
/// :returns: True if the type is the QIR qubit type.
/// :rtype: bool
#[pyfunction]
#[pyo3(text_signature = "(ty)")]
pub(crate) fn is_qubit_type(ty: &Type) -> bool {
    unsafe { types::is_qubit(ty.ty) }
}

/// The QIR result type.
///
/// :param Context context: The LLVM context.
/// :returns: The result type.
/// :rtype: Type
#[pyfunction]
#[pyo3(text_signature = "(context)")]
pub(crate) fn result_type(py: Python, context: Py<Context>) -> PyResult<PyObject> {
    unsafe {
        let ty = types::result(context.borrow(py).get_ref());
        Type::from_ptr(py, context, ty)
    }
}

/// Whether the type is the QIR result type.
///
/// :param Type ty: The type.
/// :returns: True if the type is the QIR result type.
/// :rtype: bool
#[pyfunction]
#[pyo3(text_signature = "(ty)")]
pub(crate) fn is_result_type(ty: &Type) -> bool {
    unsafe { types::is_result(ty.ty) }
}

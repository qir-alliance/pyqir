// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{core::Context, values::Owner};
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, LLVMType, LLVMTypeKind};
use pyo3::{conversion::ToPyObject, prelude::*};
use qirlib::types;
use std::{ffi::CStr, ops::Deref, ptr::NonNull};

/// A type.
#[pyclass(subclass, unsendable)]
pub(crate) struct Type {
    ty: NonNull<LLVMType>,
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
        let ty = unsafe { LLVMVoidTypeInContext(context.borrow(py).as_ptr()) };
        Type {
            ty: NonNull::new(ty).unwrap(),
            context,
        }
    }

    /// The double type.
    ///
    /// :param Context context: The LLVM context.
    /// :returns: The double type.
    /// :rtype: Type
    #[staticmethod]
    #[pyo3(text_signature = "(context)")]
    fn double(py: Python, context: Py<Context>) -> Self {
        let ty = unsafe { LLVMDoubleTypeInContext(context.borrow(py).as_ptr()) };
        Type {
            ty: NonNull::new(ty).unwrap(),
            context,
        }
    }

    /// Whether this type is the void type.
    ///
    /// :type: bool
    #[getter]
    fn is_void(&self) -> bool {
        unsafe { LLVMGetTypeKind(self.as_ptr()) == LLVMTypeKind::LLVMVoidTypeKind }
    }

    /// Whether this type is the bool type.
    ///
    /// :type: bool
    #[getter]
    fn is_double(&self) -> bool {
        unsafe { LLVMGetTypeKind(self.as_ptr()) == LLVMTypeKind::LLVMDoubleTypeKind }
    }
}

impl Type {
    pub(crate) unsafe fn from_ptr(
        py: Python,
        context: Py<Context>,
        ty: NonNull<LLVMType>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Self { ty, context });
        match LLVMGetTypeKind(ty.as_ptr()) {
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
    type Target = NonNull<LLVMType>;

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
        let ty = unsafe { LLVMIntTypeInContext(context.borrow(py).as_ptr(), width) };
        (
            Self,
            Type {
                ty: NonNull::new(ty).unwrap(),
                context,
            },
        )
    }

    /// The number of bits in the integer.
    ///
    /// :type: int
    #[getter]
    fn width(slf: PyRef<Self>) -> u32 {
        unsafe { LLVMGetIntTypeWidth(slf.into_super().as_ptr()) }
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

        let mut params: Vec<_> = params.iter().map(|ty| ty.as_ptr()).collect();
        let ty = unsafe {
            LLVMFunctionType(
                ret.as_ptr(),
                params.as_mut_ptr(),
                params.len().try_into().unwrap(),
                0,
            )
        };

        Ok((
            Self,
            Type {
                ty: NonNull::new(ty).unwrap(),
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
        unsafe {
            let ty = LLVMGetReturnType(slf.as_ptr());
            Type::from_ptr(py, context, NonNull::new(ty).unwrap())
        }
    }

    /// The types of the function parameters.
    ///
    /// :type: typing.List[Type]
    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let slf = slf.into_super();
        unsafe {
            let count = LLVMCountParamTypes(slf.as_ptr()).try_into().unwrap();
            let mut params = Vec::with_capacity(count);
            LLVMGetParamTypes(slf.as_ptr(), params.as_mut_ptr());
            params.set_len(count);
            params
                .into_iter()
                .map(|ty| Type::from_ptr(py, slf.context.clone_ref(py), NonNull::new(ty).unwrap()))
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
            let name = LLVMGetStructName(slf.into_super().as_ptr());
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
            let count = LLVMCountStructElementTypes(slf.as_ptr())
                .try_into()
                .unwrap();
            let mut fields = Vec::with_capacity(count);
            LLVMGetStructElementTypes(slf.as_ptr(), fields.as_mut_ptr());
            fields.set_len(count);
            fields
                .into_iter()
                .map(|ty| Type::from_ptr(py, slf.context.clone_ref(py), NonNull::new(ty).unwrap()))
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
        unsafe {
            let ty = LLVMGetElementType(slf.as_ptr());
            Type::from_ptr(py, slf.context.clone_ref(py), NonNull::new(ty).unwrap())
        }
    }

    /// The number of elements in the array.
    ///
    /// :type: int
    #[getter]
    fn count(slf: PyRef<Self>) -> u32 {
        unsafe { LLVMGetArrayLength(slf.into_super().as_ptr()) }
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
        let ty = unsafe { LLVMPointerType(pointee.as_ptr(), 0) };
        (
            Self,
            Type {
                ty: NonNull::new(ty).unwrap(),
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
        unsafe {
            let ty = LLVMGetElementType(slf.as_ptr());
            Type::from_ptr(py, slf.context.clone_ref(py), NonNull::new(ty).unwrap())
        }
    }

    /// The pointer address space.
    ///
    /// :type: int
    #[getter]
    fn address_space(slf: PyRef<Self>) -> u32 {
        unsafe { LLVMGetPointerAddressSpace(slf.into_super().as_ptr()) }
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
        let ty = types::qubit(context.borrow(py).as_ptr());
        Type::from_ptr(py, context, NonNull::new(ty).unwrap())
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
    unsafe { types::is_qubit(ty.as_ptr()) }
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
        let ty = types::result(context.borrow(py).as_ptr());
        Type::from_ptr(py, context, NonNull::new(ty).unwrap())
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
    unsafe { types::is_result(ty.as_ptr()) }
}

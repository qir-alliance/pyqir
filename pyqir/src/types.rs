// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{core::Context, values::Owner};
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, prelude::*, LLVMType, LLVMTypeKind};
use pyo3::{prelude::*, IntoPyObjectExt};
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
        let ty = unsafe { LLVMVoidTypeInContext(context.borrow(py).cast().as_ptr()) };
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
        let ty = unsafe { LLVMDoubleTypeInContext(context.borrow(py).cast().as_ptr()) };
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
        unsafe { LLVMGetTypeKind(self.cast().as_ptr()) == LLVMTypeKind::LLVMVoidTypeKind }
    }

    /// Whether this type is the bool type.
    ///
    /// :type: bool
    #[getter]
    fn is_double(&self) -> bool {
        unsafe { LLVMGetTypeKind(self.cast().as_ptr()) == LLVMTypeKind::LLVMDoubleTypeKind }
    }
}

impl Type {
    pub(crate) unsafe fn from_raw(
        py: Python<'_>,
        context: Py<Context>,
        ty: LLVMTypeRef,
    ) -> PyResult<Bound<'_, PyAny>> {
        let ty = NonNull::new(ty).expect("Type is null.");
        let base = PyClassInitializer::from(Self { ty, context });
        match LLVMGetTypeKind(ty.cast().as_ptr()) {
            LLVMTypeKind::LLVMArrayTypeKind => {
                Ok(Py::new(py, base.add_subclass(ArrayType))?.into_bound_py_any(py)?)
            }
            LLVMTypeKind::LLVMFunctionTypeKind => {
                Ok(Py::new(py, base.add_subclass(FunctionType))?.into_bound_py_any(py)?)
            }
            LLVMTypeKind::LLVMIntegerTypeKind => {
                Ok(Py::new(py, base.add_subclass(IntType))?.into_bound_py_any(py)?)
            }
            LLVMTypeKind::LLVMPointerTypeKind => {
                Ok(Py::new(py, base.add_subclass(PointerType))?.into_bound_py_any(py)?)
            }
            LLVMTypeKind::LLVMStructTypeKind => {
                Ok(Py::new(py, base.add_subclass(StructType))?.into_bound_py_any(py)?)
            }
            _ => Ok(Py::new(py, base)?.into_bound_py_any(py)?),
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
pub(crate) struct IntType;

#[pymethods]
impl IntType {
    #[new]
    #[pyo3(text_signature = "(context, width)")]
    fn new(py: Python, context: Py<Context>, width: u32) -> (Self, Type) {
        let ty = unsafe { LLVMIntTypeInContext(context.borrow(py).cast().as_ptr(), width) };
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
        unsafe { LLVMGetIntTypeWidth(slf.into_super().cast().as_ptr()) }
    }
}

/// A function type.
///
/// :param Type ret: The return type.
/// :param typing.Sequence[Type] params: The parameter types.
#[pyclass(extends = Type)]
pub(crate) struct FunctionType;

#[pymethods]
impl FunctionType {
    #[new]
    #[pyo3(text_signature = "(ret, params)")]
    fn new(py: Python, ret: &Type, params: Vec<PyRef<Type>>) -> PyResult<(Self, Type)> {
        Owner::merge(
            py,
            params
                .iter()
                .map(|ty| Owner::Context(ty.context.clone_ref(py)))
                .chain([ret.context.clone_ref(py).into()]),
        )?;

        let mut params: Vec<_> = params.iter().map(|ty| ty.cast().as_ptr()).collect();
        let ty = unsafe {
            LLVMFunctionType(
                ret.cast().as_ptr(),
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
    fn ret<'py>(slf: PyRef<Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let slf = slf.into_super();
        let context = slf.context.clone_ref(py);
        unsafe { Type::from_raw(py, context, LLVMGetReturnType(slf.cast().as_ptr())) }
    }

    /// The types of the function parameters.
    ///
    /// :type: typing.List[Type]
    #[getter]
    fn params<'py>(slf: PyRef<Self>, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyAny>>> {
        let slf = slf.into_super();
        unsafe {
            let count = LLVMCountParamTypes(slf.cast().as_ptr()).try_into().unwrap();
            let mut params = Vec::with_capacity(count);
            LLVMGetParamTypes(slf.cast().as_ptr(), params.as_mut_ptr());
            params.set_len(count);
            params
                .into_iter()
                .map(|ty| Type::from_raw(py, slf.context.clone_ref(py), ty))
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
            let name = LLVMGetStructName(slf.into_super().cast().as_ptr());
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
    fn fields<'py>(slf: PyRef<Self>, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyAny>>> {
        let slf = slf.into_super();
        unsafe {
            let count = LLVMCountStructElementTypes(slf.cast().as_ptr())
                .try_into()
                .unwrap();
            let mut fields = Vec::with_capacity(count);
            LLVMGetStructElementTypes(slf.cast().as_ptr(), fields.as_mut_ptr());
            fields.set_len(count);
            fields
                .into_iter()
                .map(|ty| Type::from_raw(py, slf.context.clone_ref(py), ty))
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
    fn element<'py>(slf: PyRef<Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let slf = slf.into_super();
        unsafe {
            let ty = LLVMGetElementType(slf.cast().as_ptr());
            Type::from_raw(py, slf.context.clone_ref(py), ty)
        }
    }

    /// The number of elements in the array.
    ///
    /// :type: int
    #[getter]
    fn count(slf: PyRef<Self>) -> u64 {
        unsafe { LLVMGetArrayLength2(slf.into_super().cast().as_ptr()) }
    }
}

/// A pointer type.
///
/// :param Type pointee: The type being pointed to.
#[pyclass(extends = Type)]
pub(crate) struct PointerType;

#[pymethods]
impl PointerType {
    /// TODO: remove argument and update for opaque pointers?
    #[new]
    #[pyo3(text_signature = "(pointee)")]
    fn new(py: Python, pointee: &Type) -> (Self, Type) {
        let ty = unsafe { LLVMPointerType(pointee.cast().as_ptr(), 0) };
        (
            Self,
            Type {
                ty: NonNull::new(ty).unwrap(),
                context: pointee.context.clone_ref(py),
            },
        )
    }

    /// The type being pointed to. With opaque pointers, always treat this as void.
    ///
    /// :type: Type
    #[getter]
    fn pointee<'py>(slf: PyRef<Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let slf = slf.into_super();
        unsafe {
            if LLVMPointerTypeIsOpaque(slf.ty.as_ptr()) != 0 {
                return Py::new(py, Type::void(py, slf.context.clone_ref(py)))?
                    .into_bound_py_any(py);
            }
            let ty = LLVMGetElementType(slf.cast().as_ptr());
            Type::from_raw(py, slf.context.clone_ref(py), ty)
        }
    }

    /// The pointer address space.
    ///
    /// :type: int
    #[getter]
    fn address_space(slf: PyRef<Self>) -> u32 {
        unsafe { LLVMGetPointerAddressSpace(slf.into_super().cast().as_ptr()) }
    }
}

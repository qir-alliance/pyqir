// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    core::{Context, Message},
    values::{Constant, Owner},
};
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, prelude::*};
use llvm_sys::{
    debuginfo::{LLVMGetMetadataKind, LLVMMetadataKind},
    LLVMOpaqueMetadata,
};
use pyo3::{conversion::ToPyObject, exceptions::PyValueError, prelude::*};
use std::{ffi::CString, ops::Deref, ptr::NonNull, slice, str};

/// A metadata value or node.
#[pyclass(subclass, unsendable)]
pub(crate) struct Metadata {
    value: NonNull<LLVMOpaqueMetadata>,
    owner: Owner,
}

#[pymethods]
impl Metadata {
    fn __str__(&self, py: Python) -> String {
        unsafe {
            let context = self.owner.context(py).borrow(py).as_ptr();
            let value = LLVMMetadataAsValue(context, self.as_ptr());
            Message::from_raw(LLVMPrintValueToString(value))
                .to_str()
                .unwrap()
                .to_string()
        }
    }
}

impl Metadata {
    pub(crate) unsafe fn from_raw(
        py: Python,
        owner: Owner,
        md: LLVMMetadataRef,
    ) -> PyResult<PyObject> {
        match LLVMGetMetadataKind(md) {
            LLVMMetadataKind::LLVMMDStringMetadataKind => {
                Ok(Py::new(py, MetadataString::from_raw(py, owner, md)?)?.to_object(py))
            }
            LLVMMetadataKind::LLVMConstantAsMetadataMetadataKind => {
                ConstantAsMetadata::from_raw(py, owner, md)
            }
            _ => {
                let value = NonNull::new(md).expect("Value is null.");
                Ok(Py::new(py, Self { value, owner })?.to_object(py))
            }
        }
    }

    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }
}

impl Deref for Metadata {
    type Target = NonNull<LLVMOpaqueMetadata>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// A metadata string
#[pyclass(extends = Metadata, subclass)]
#[pyo3(text_signature = "(context, string)")]
pub(crate) struct MetadataString;

#[pymethods]
impl MetadataString {
    /// Creates a metadata string
    ///
    /// :param context: The LLVM context.
    /// :param string: the value of the metadata string to create
    #[new]
    pub(crate) unsafe fn new(
        py: Python,
        context: Py<Context>,
        string: &str,
    ) -> PyResult<PyClassInitializer<Self>> {
        let owner = context.clone_ref(py).into();
        let c_string = CString::new(string).unwrap();
        let context = context.borrow(py).as_ptr();
        let md = unsafe { LLVMMDStringInContext2(context, c_string.as_ptr(), string.len()) };
        unsafe { MetadataString::from_raw(py, owner, md) }
    }

    /// The underlying metadata string value.
    ///
    /// :type: str
    #[getter]
    fn value(slf: PyRef<Self>, py: Python) -> String {
        let mut len = 0;
        unsafe {
            let slf = slf.into_super();
            let context = slf.owner.context(py).borrow(py).as_ptr();
            let value = LLVMMetadataAsValue(context, slf.as_ptr());
            let mds = LLVMGetMDString(value, &mut len);
            str::from_utf8(slice::from_raw_parts(mds.cast(), len as usize))
                .unwrap()
                .to_string()
        }
    }
}

impl MetadataString {
    unsafe fn from_raw(
        py: Python,
        owner: Owner,
        value: LLVMMetadataRef,
    ) -> PyResult<PyClassInitializer<Self>> {
        let value = NonNull::new(value).expect("Value is null.");
        let context = owner.context(py).borrow(py).as_ptr();
        let valueref = LLVMMetadataAsValue(context, value.as_ptr());
        if LLVMIsAMDString(valueref) == valueref {
            Ok(PyClassInitializer::from(Metadata { value, owner }).add_subclass(MetadataString))
        } else {
            Err(PyValueError::new_err("Value is not a metadata string."))
        }
    }
}

/// A metadata constant value.
#[pyclass(extends = Metadata, subclass)]
pub(crate) struct ConstantAsMetadata;

#[pymethods]
impl ConstantAsMetadata {
    /// The value.
    ///
    /// :type: int
    #[getter]
    fn value(slf: PyRef<Self>, py: Python) -> Option<PyObject> {
        let slf = slf.into_super();
        let context = slf.owner.context(py).borrow(py).as_ptr();
        let valueref = unsafe { LLVMMetadataAsValue(context, slf.as_ptr()) };

        if let Some(value) = unsafe { qirlib::metadata::extract_constant(valueref) } {
            return unsafe { Constant::from_raw(py, slf.owner().clone_ref(py), value).ok() };
        }
        None
    }
}

impl ConstantAsMetadata {
    unsafe fn from_raw(py: Python, owner: Owner, value: LLVMMetadataRef) -> PyResult<PyObject> {
        let value = NonNull::new(value).expect("Value is null.");
        let context = owner.context(py).borrow(py).as_ptr();
        let valueref = LLVMMetadataAsValue(context, value.as_ptr());
        if qirlib::metadata::extract_constant(valueref).is_some() {
            let initializer =
                PyClassInitializer::from(Metadata { value, owner }).add_subclass(Self);
            Ok(Py::new(py, initializer)?.to_object(py))
        } else {
            println!("Could not extract constant.");
            Err(PyValueError::new_err("Could not extract constant."))
        }
    }
}

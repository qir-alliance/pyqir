// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    core::{Context, Message},
    types::Type,
    values::Owner,
};
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, prelude::*, LLVMValue};
use llvm_sys::{
    debuginfo::{LLVMGetMetadataKind, LLVMMetadataKind},
    LLVMValueKind,
};
use pyo3::{conversion::ToPyObject, exceptions::PyValueError, prelude::*};
use qirlib::llvm_wrapper::{LLVMRustExtractMDConstant, LLVMRustIsAMDConstant};
use std::{
    ffi::CString,
    ops::Deref,
    ptr::{null_mut, NonNull},
    slice, str,
};

/// A metadata value or node.
#[pyclass(subclass, unsendable)]
pub(crate) struct Metadata {
    value: NonNull<LLVMValue>,
    owner: Owner,
}

#[pymethods]
impl Metadata {
    fn __str__(&self) -> String {
        unsafe {
            Message::from_raw(LLVMPrintValueToString(self.as_ptr()))
                .to_str()
                .unwrap()
                .to_string()
        }
    }
}

impl Metadata {
    pub(crate) unsafe fn new(owner: Owner, value: NonNull<LLVMValue>) -> Self {
        Self { value, owner }
    }

    pub(crate) unsafe fn from_raw(
        py: Python,
        owner: Owner,
        value: LLVMValueRef,
    ) -> PyResult<PyObject> {
        let md = LLVMValueAsMetadata(value);
        match LLVMGetMetadataKind(md) {
            LLVMMetadataKind::LLVMMDStringMetadataKind => {
                Ok(Py::new(py, MetadataString::from_raw(owner, value)?)?.to_object(py))
            }
            LLVMMetadataKind::LLVMConstantAsMetadataMetadataKind => {
                ConstantAsMetadata::from_raw(py, owner, value)
            }
            _ => {
                let value = NonNull::new(value).expect("Value is null.");
                Ok(Py::new(py, Self { value, owner })?.to_object(py))
            }
        }
    }

    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }
}

impl Deref for Metadata {
    type Target = NonNull<LLVMValue>;

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
        let md = unsafe {
            LLVMMDStringInContext2(context, c_string.as_ptr(), string.len().try_into().unwrap())
        };
        let value = unsafe { LLVMMetadataAsValue(context, md) };
        unsafe { MetadataString::from_raw(owner, value) }
    }

    /// The underlying metadata string value.
    ///
    /// :type: str
    #[getter]
    fn value(slf: PyRef<Self>) -> &str {
        let mut len = 0;
        unsafe {
            let mds = LLVMGetMDString(slf.into_super().as_ptr(), &mut len);
            str::from_utf8(slice::from_raw_parts(mds.cast(), len as usize)).unwrap()
        }
    }
}

impl MetadataString {
    unsafe fn from_raw(owner: Owner, value: LLVMValueRef) -> PyResult<PyClassInitializer<Self>> {
        let value = NonNull::new(value).expect("Value is null.");
        if LLVMIsAMDString(value.as_ptr()) != value.as_ptr() {
            Err(PyValueError::new_err("Value is not a metadata string."))
        } else {
            Ok(PyClassInitializer::from(Metadata { value, owner }).add_subclass(MetadataString))
        }
    }
}

/// A metadata constant value.
#[pyclass(extends = Metadata, subclass)]
pub(crate) struct ConstantAsMetadata;

impl ConstantAsMetadata {
    unsafe fn from_raw(py: Python, owner: Owner, value: LLVMValueRef) -> PyResult<PyObject> {
        let value = NonNull::new(value).expect("Value is null.");

        if LLVMRustIsAMDConstant(value.as_ptr()) == std::ptr::null_mut() {
            println!("Value is not constant.");
            Err(PyValueError::new_err("Value is not constant."))
        } else {
            let constant = LLVMRustExtractMDConstant(value.as_ptr());
            if constant == null_mut() {
                println!("Could not extract constant.");
                return Err(PyValueError::new_err("Could not extract constant."));
            }
            match LLVMGetValueKind(constant) {
                LLVMValueKind::LLVMConstantIntValueKind => {
                    let initializer = PyClassInitializer::from(Metadata { value, owner })
                        .add_subclass(Self)
                        .add_subclass(MetadataIntConstant);
                    Ok(Py::new(py, initializer)?.to_object(py))
                }
                _ => {
                    let initializer =
                        PyClassInitializer::from(Metadata { value, owner }).add_subclass(Self);
                    Ok(Py::new(py, initializer)?.to_object(py))
                }
            }
        }
    }
}

/// A metadata constant integer value.
#[pyclass(extends = ConstantAsMetadata)]
pub(crate) struct MetadataIntConstant;

#[pymethods]
impl MetadataIntConstant {
    /// The value.
    ///
    /// :type: int
    #[getter]
    fn value(slf: PyRef<Self>) -> u64 {
        let value = unsafe { LLVMRustExtractMDConstant(slf.into_super().into_super().as_ptr()) };
        unsafe { LLVMConstIntGetZExtValue(value) }
    }
}

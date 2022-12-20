// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{core::Message, types::Type, values::Owner};
use llvm_sys::debuginfo::LLVMGetMetadataKind;
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, prelude::*, LLVMValue};
use pyo3::{conversion::ToPyObject, prelude::*};
use std::{ops::Deref, ptr::NonNull, slice, str};

/// A metadata value or node.
#[pyclass(subclass, unsendable)]
pub(crate) struct Metadata {
    value: NonNull<LLVMValue>,
    owner: Owner,
}

#[pymethods]
impl Metadata {
    /// The type of this value.
    ///
    /// :type: Type
    #[getter]
    fn r#type(&self, py: Python) -> PyResult<PyObject> {
        unsafe { Type::from_raw(py, self.owner.context(py), LLVMTypeOf(self.as_ptr())) }
    }

    /// The name of this value or the empty string if this value is anonymous.
    #[getter]
    fn name(&self) -> &str {
        let mut len = 0;
        unsafe {
            let name = LLVMGetValueName2(self.as_ptr(), &mut len).cast();
            str::from_utf8(slice::from_raw_parts(name, len)).unwrap()
        }
    }

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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use inkwell::values::AnyValueEnum;
use pyo3::prelude::*;
use std::ops::Deref;

use crate::values::Value;

/// The context owns global state needed by most LLVM objects.
#[pyclass]
#[derive(Eq, PartialEq)]
pub(crate) struct Context(inkwell::context::Context);

#[pymethods]
impl Context {
    #[new]
    pub(crate) fn new() -> Self {
        Self(inkwell::context::Context::create())
    }

    /// Creates a metadata string
    ///
    /// :param string: the value of the metadata string to create
    /// :returns: metadata string value of the supplied string
    #[pyo3(text_signature = "(string)")]
    fn create_metadata_string(slf: Py<Context>, py: Python, string: &str) -> PyResult<PyObject> {
        let owner = slf.clone_ref(py).into();
        let context = slf.borrow(py);
        let md = context.metadata_string(string);
        let ave = AnyValueEnum::MetadataValue(md);
        unsafe { Value::from_any(py, owner, ave) }
    }
}

impl Deref for Context {
    type Target = inkwell::context::Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

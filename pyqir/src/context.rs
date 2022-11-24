// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pyo3::prelude::*;
use std::ops::Deref;

#[pyclass]
#[derive(Eq, PartialEq)]
pub(crate) struct Context(inkwell::context::Context);

/// The context owns global state needed by most LLVM objects.
#[pymethods]
impl Context {
    #[new]
    pub(crate) fn new() -> Self {
        Self(inkwell::context::Context::create())
    }
}

impl Deref for Context {
    type Target = inkwell::context::Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

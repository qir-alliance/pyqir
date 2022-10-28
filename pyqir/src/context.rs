// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::utils::is_all_same;
use pyo3::{exceptions::PyValueError, prelude::*};
use std::{borrow::Borrow, ops::Deref};

#[pyclass]
#[derive(Eq, PartialEq)]
pub(crate) struct Context(inkwell::context::Context);

impl Context {
    pub(crate) fn new(context: inkwell::context::Context) -> Self {
        Self(context)
    }
}

impl Deref for Context {
    type Target = inkwell::context::Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn require_same(
    py: Python,
    contexts: impl IntoIterator<Item = impl Borrow<Py<Context>>>,
) -> PyResult<()> {
    // then_some is stabilized in Rust 1.62.
    #[allow(clippy::unnecessary_lazy_evaluations)]
    is_all_same(py, contexts)
        .then(|| ())
        .ok_or_else(|| PyValueError::new_err("Some objects come from a different context."))
}

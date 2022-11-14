// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pyo3::{exceptions::PyValueError, prelude::*, PyClass};
use std::{borrow::Borrow, ops::Deref};

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

fn is_all_same<T>(py: Python, items: impl IntoIterator<Item = impl Borrow<Py<T>>>) -> bool
where
    T: Eq + PyClass,
{
    let mut items = items.into_iter();
    if let Some(mut prev) = items.next() {
        for item in items {
            if *item.borrow().borrow(py) != *prev.borrow().borrow(py) {
                return false;
            }
            prev = item;
        }
    }
    true
}

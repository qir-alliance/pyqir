// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

#[cfg(feature = "llvm11-0")]
extern crate llvm_sys_110 as llvm_sys;
#[cfg(feature = "llvm12-0")]
extern crate llvm_sys_120 as llvm_sys;
#[cfg(feature = "llvm13-0")]
extern crate llvm_sys_130 as llvm_sys;
#[cfg(feature = "llvm14-0")]
extern crate llvm_sys_140 as llvm_sys;

mod builder;
mod evaluator;
mod instructions;
mod module;
mod python;
mod qis;
mod types;
mod values;

use inkwell::attributes::AttributeLoc;
use pyo3::prelude::*;
use std::ops::Deref;

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
}

impl Deref for Context {
    type Target = inkwell::context::Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An attribute.
#[pyclass(unsendable)]
pub(crate) struct Attribute(pub(crate) inkwell::attributes::Attribute);

#[pymethods]
impl Attribute {
    /// The value of this attribute as a string, or `None` if this is not a string attribute.
    ///
    /// :type: typing.Optional[str]
    #[getter]
    fn string_value(&self) -> Option<&str> {
        if self.0.is_string() {
            Some(
                self.0
                    .get_string_value()
                    .to_str()
                    .expect("Value is not valid UTF-8."),
            )
        } else {
            None
        }
    }
}

/// The position of an attribute within a function declaration.
#[pyclass]
pub(crate) struct AttributeIndex(pub(crate) AttributeLoc);

#[pymethods]
impl AttributeIndex {
    #[classattr]
    const FUNCTION: Self = Self(AttributeLoc::Function);

    #[classattr]
    const RETURN: Self = Self(AttributeLoc::Return);

    /// The attribute index for the nth parameter, starting from zero.
    ///
    /// :param int n: The parameter number.
    /// :returns: The attribute index.
    /// :rtype: AttributeIndex
    #[staticmethod]
    fn param(n: u32) -> Self {
        Self(AttributeLoc::Param(n))
    }
}

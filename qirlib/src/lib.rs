// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]

#[cfg(feature = "llvm18-1")]
extern crate llvm_sys_181 as llvm_sys;
#[cfg(feature = "llvm19-1")]
extern crate llvm_sys_191 as llvm_sys;
#[cfg(feature = "llvm20-1")]
extern crate llvm_sys_201 as llvm_sys;

#[cfg(not(feature = "no-llvm-linking"))]
pub mod builder;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod context;
#[cfg(not(feature = "no-llvm-linking"))]
pub(crate) mod llvm_wrapper;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod metadata;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod module;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod qis;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod rt;
#[cfg(all(test, not(feature = "no-llvm-linking")))]
mod tests;
#[cfg(not(feature = "no-llvm-linking"))]
pub(crate) mod utils;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod values;

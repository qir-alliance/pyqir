// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(unknown_lints, clippy::unnecessary_fallible_conversions)] // can be removed when MSRV is 1.75+

#[cfg(feature = "llvm18-1")]
extern crate llvm_sys_181 as llvm_sys;
#[cfg(feature = "llvm19-1")]
extern crate llvm_sys_191 as llvm_sys;
#[cfg(feature = "llvm20-1")]
extern crate llvm_sys_201 as llvm_sys;

mod builder;
mod core;
mod instructions;
mod metadata;
mod module;
mod python;
mod qis;
mod rt;
mod types;
mod values;

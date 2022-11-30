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
mod context;
mod evaluator;
mod instructions;
mod module;
mod python;
mod qis;
mod simple;
mod types;
mod values;

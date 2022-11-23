// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
//
// To store Inkwell objects in Python classes, we transmute the `'ctx` lifetime to static. You need
// to be careful when using Inkwell types with unsafely extended lifetimes. Follow these rules:
//
// 1. When storing in a data type, always include a `Py<Context>` field containing the context
//    originally referred to by `'ctx`.
// 2. Before calling Inkwell methods that use `'ctx`, call `context::require_same` to assert that
//    all contexts being used are the same.

#![warn(clippy::all, clippy::pedantic)]

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
mod rt;
mod simple;
mod types;
mod values;

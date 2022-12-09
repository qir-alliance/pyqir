// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

#[cfg(feature = "llvm11-0")]
extern crate llvm_sys_110 as llvm_sys;
#[cfg(feature = "llvm12-0")]
extern crate llvm_sys_120 as llvm_sys;
#[cfg(feature = "llvm13-0")]
extern crate llvm_sys_130 as llvm_sys;
#[cfg(feature = "llvm14-0")]
extern crate llvm_sys_140 as llvm_sys;

#[cfg(not(feature = "no-llvm-linking"))]
pub mod builder;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod evaluation;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod extensions;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod qis;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod rt;
#[cfg(all(test, not(feature = "no-llvm-linking")))]
mod tests;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod types;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod values;

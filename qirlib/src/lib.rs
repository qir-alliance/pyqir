// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

#[cfg(not(feature = "no-llvm-linking"))]
pub use builder::Builder;
#[cfg(not(feature = "no-llvm-linking"))]
pub use qis::BuilderBasicQisExt;

#[cfg(not(feature = "no-llvm-linking"))]
mod builder;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod evaluation;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod module;
#[cfg(not(feature = "no-llvm-linking"))]
mod qis;
#[cfg(all(test, not(feature = "no-llvm-linking")))]
mod tests;
#[cfg(not(feature = "no-llvm-linking"))]
pub mod types;

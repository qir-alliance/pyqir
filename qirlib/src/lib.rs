// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

#[cfg(not(any(feature = "no-llvm-linking")))]
pub use inkwell;

#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod builder;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod evaluation;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod module;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod qis;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod types;

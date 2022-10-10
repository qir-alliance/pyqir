// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

#[cfg(not(any(feature = "no-llvm-linking")))]
pub use builder::Builder;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub use inkwell;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub use qis::BuilderBasicQisExt;

#[cfg(not(any(feature = "no-llvm-linking")))]
mod builder;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod evaluation;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod module;
#[cfg(not(any(feature = "no-llvm-linking")))]
mod qis;
#[cfg(not(any(feature = "no-llvm-linking")))]
pub mod types;

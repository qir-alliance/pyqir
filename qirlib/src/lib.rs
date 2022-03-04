// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

#[cfg(not(any(feature = "package-llvm", feature = "install-llvm")))]
pub mod codegen;

#[cfg(not(any(feature = "package-llvm", feature = "install-llvm")))]
pub mod evaluation;

#[cfg(not(any(feature = "package-llvm", feature = "install-llvm")))]
pub mod generation;

#[cfg(not(any(feature = "package-llvm", feature = "install-llvm")))]
pub mod module;

#[cfg(not(any(feature = "package-llvm", feature = "install-llvm")))]
pub mod passes;

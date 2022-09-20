// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

#[cfg(feature = "python-bindings")]
pub mod python;

#[cfg(feature = "python-bindings")]
mod types;

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

pub mod parse;

#[cfg(feature = "python-bindings")]
pub mod python;

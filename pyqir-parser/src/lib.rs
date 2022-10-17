// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]

mod parse;
#[cfg(feature = "python-bindings")]
pub mod python;

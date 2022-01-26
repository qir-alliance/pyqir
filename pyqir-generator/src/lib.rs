// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]
// pyo3 generates errors with _obj and _tmp values
#![allow(clippy::used_underscore_binding)]

pub mod emit;
pub mod interop;
pub mod python;
pub mod qir;

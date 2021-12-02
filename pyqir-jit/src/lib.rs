// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![deny(clippy::all, clippy::pedantic)]
// pyo3 generates errors with _obj and _tmp values
#![allow(clippy::used_underscore_binding)]

pub mod gates;
pub mod interop;
pub mod intrinsics;
pub mod jit;
pub mod python;
pub mod runtime;

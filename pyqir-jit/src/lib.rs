// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]
// pyo3 generates errors with _obj and _tmp values
#![allow(clippy::used_underscore_binding)]
// NB: Some arguments get turned into Deref by PyO3, which is meaningless
//     for Option of a type that's already Deref. We ignore the warning
//     here since it's introduced by an upstream macro and not somethi8ng
//     we can directly control in our code.
#![allow(clippy::needless_option_as_deref)]

pub mod gates;
pub mod interop;
pub mod intrinsics;
pub mod jit;
pub mod python;
pub mod runtime;

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::all, clippy::pedantic)]
// pyo3 generates errors with _obj and _tmp values
#![allow(clippy::used_underscore_binding)]
// NB: Some arguments get turned into Deref by PyO3, which is meaningless
//     for Option of a type that's already Deref. We ignore the warning
//     here since it's introduced by an upstream macro and not something
//     we can directly control in our code.
#![allow(clippy::needless_option_as_deref)]
// This was introduced in 1.62, but we can't update the dependency to
// to resolve it until we move to a newer version of python.
#![allow(clippy::format_push_string)]

#[cfg(feature = "python-bindings")]
pub mod python;

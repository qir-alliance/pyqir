// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![deny(clippy::all, clippy::pedantic)]

pub mod constants;
pub mod context;
pub mod intrinsics;
pub(crate) mod module;
pub mod runtime_library;
pub mod passes;
pub mod types;
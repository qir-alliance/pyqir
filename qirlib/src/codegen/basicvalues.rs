// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    context::Context,
    values::{BasicMetadataValueEnum, BasicValue},
};

#[must_use]
pub(crate) fn f64_to_f64(context: &Context, value: f64) -> BasicMetadataValueEnum {
    context
        .f64_type()
        .const_float(value)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub fn u64_to_i64(context: &Context, value: u64) -> BasicMetadataValueEnum {
    context
        .i64_type()
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    context::Context,
    values::{BasicMetadataValueEnum, BasicValue},
};

#[must_use]
pub(crate) fn i8_null_ptr(context: &Context) -> BasicMetadataValueEnum {
    context
        .i8_type()
        .ptr_type(inkwell::AddressSpace::Generic)
        .const_null()
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub(crate) fn f64_to_f64(context: &Context, value: f64) -> BasicMetadataValueEnum {
    context
        .f64_type()
        .const_float(value)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub(crate) fn u64_to_i32(context: &Context, value: u64) -> BasicMetadataValueEnum {
    context
        .i32_type()
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub(crate) fn i64_to_i32(context: &Context, value: i64) -> BasicMetadataValueEnum {
    // convert to capture negative values.
    #[allow(clippy::cast_sign_loss)]
    let target: u64 = value as u64;

    context
        .i32_type()
        .const_int(target, false)
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

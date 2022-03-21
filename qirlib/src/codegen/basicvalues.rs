// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::types::{double, int32, int64, int8};
use inkwell::values::{BasicMetadataValueEnum, BasicValue};

#[must_use]
pub(crate) fn i8_null_ptr(context: &'_ inkwell::context::Context) -> BasicMetadataValueEnum<'_> {
    int8(context)
        .ptr_type(inkwell::AddressSpace::Generic)
        .const_null()
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub(crate) fn f64_to_f64(
    context: &inkwell::context::Context,
    value: f64,
) -> BasicMetadataValueEnum {
    double(context)
        .const_float(value)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub(crate) fn u64_to_i32(
    context: &'_ inkwell::context::Context,
    value: u64,
) -> BasicMetadataValueEnum<'_> {
    int32(context)
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub(crate) fn i64_to_i32(
    context: &inkwell::context::Context,
    value: i64,
) -> BasicMetadataValueEnum {
    // convert to capture negative values.
    #[allow(clippy::cast_sign_loss)]
    let target: u64 = value as u64;

    int32(context)
        .const_int(target, false)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub(crate) fn u64_to_i64(
    context: &inkwell::context::Context,
    value: u64,
) -> BasicMetadataValueEnum {
    int64(context)
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

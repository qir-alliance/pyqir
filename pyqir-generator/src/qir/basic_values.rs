// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicMetadataValueEnum, BasicValue};

use qirlib::context::Context;

pub(crate) fn i8_null_ptr<'ctx>(context: &Context<'ctx>) -> BasicMetadataValueEnum<'ctx> {
    context
        .context
        .i8_type()
        .ptr_type(inkwell::AddressSpace::Generic)
        .const_null()
        .as_basic_value_enum()
        .into()
}

pub(crate) fn f64_to_f64<'ctx>(
    context: &Context<'ctx>,
    value: f64,
) -> BasicMetadataValueEnum<'ctx> {
    context
        .types
        .double
        .const_float(value)
        .as_basic_value_enum()
        .into()
}

pub(crate) fn u64_to_i32<'ctx>(
    context: &Context<'ctx>,
    value: u64,
) -> BasicMetadataValueEnum<'ctx> {
    context
        .context
        .i32_type()
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

pub(crate) fn i64_to_i32<'ctx>(
    context: &Context<'ctx>,
    value: i64,
) -> BasicMetadataValueEnum<'ctx> {
    // convert to capture negative values.
    #[allow(clippy::cast_sign_loss)]
    let target: u64 = value as u64;

    context
        .context
        .i32_type()
        .const_int(target, false)
        .as_basic_value_enum()
        .into()
}

pub(crate) fn u64_to_i64<'ctx>(
    context: &Context<'ctx>,
    value: u64,
) -> BasicMetadataValueEnum<'ctx> {
    context
        .types
        .int
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

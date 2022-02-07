// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicMetadataValueEnum, BasicValue};

use super::{
    types::{double_type, int32_type, int64_type, int8_type},
    CodeGenerator,
};

pub trait BasicValues<'ctx> {
    fn i8_null_ptr(&self) -> BasicMetadataValueEnum<'ctx>;
    fn f64_to_f64(&self, value: f64) -> BasicMetadataValueEnum<'ctx>;
    fn u64_to_i32(&self, value: u64) -> BasicMetadataValueEnum<'ctx>;
    fn i64_to_i32(&self, value: i64) -> BasicMetadataValueEnum<'ctx>;
    fn u64_to_i64(&self, value: u64) -> BasicMetadataValueEnum<'ctx>;
}

impl<'ctx> BasicValues<'ctx> for CodeGenerator<'ctx> {
    fn i8_null_ptr(&self) -> BasicMetadataValueEnum<'ctx> {
        i8_null_ptr(self.context)
    }

    fn f64_to_f64(&self, value: f64) -> BasicMetadataValueEnum<'ctx> {
        f64_to_f64(self.context, value)
    }

    fn u64_to_i32(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i32(self.context, value)
    }

    fn i64_to_i32(&self, value: i64) -> BasicMetadataValueEnum<'ctx> {
        i64_to_i32(self.context, value)
    }

    fn u64_to_i64(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i64(self.context, value)
    }
}

#[must_use]
pub fn i8_null_ptr(context: &'_ inkwell::context::Context) -> BasicMetadataValueEnum<'_> {
    int8_type(context)
        .ptr_type(inkwell::AddressSpace::Generic)
        .const_null()
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub fn f64_to_f64(context: &inkwell::context::Context, value: f64) -> BasicMetadataValueEnum {
    double_type(context)
        .const_float(value)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub fn u64_to_i32(
    context: &'_ inkwell::context::Context,
    value: u64,
) -> BasicMetadataValueEnum<'_> {
    int32_type(context)
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub fn i64_to_i32(context: &inkwell::context::Context, value: i64) -> BasicMetadataValueEnum {
    // convert to capture negative values.
    #[allow(clippy::cast_sign_loss)]
    let target: u64 = value as u64;

    int32_type(context)
        .const_int(target, false)
        .as_basic_value_enum()
        .into()
}

#[must_use]
pub fn u64_to_i64(context: &inkwell::context::Context, value: u64) -> BasicMetadataValueEnum {
    int64_type(context)
        .const_int(value, false)
        .as_basic_value_enum()
        .into()
}

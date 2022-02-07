// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicMetadataValueEnum, BasicValue};

use super::{types::Types, CodeGenerator};

pub trait BasicValues<'ctx> {
    fn i8_null_ptr(&self) -> BasicMetadataValueEnum<'ctx>;
    fn f64_to_f64(&self, value: f64) -> BasicMetadataValueEnum<'ctx>;
    fn u64_to_i32(&self, value: u64) -> BasicMetadataValueEnum<'ctx>;
    fn i64_to_i32(&self, value: i64) -> BasicMetadataValueEnum<'ctx>;
    fn u64_to_i64(&self, value: u64) -> BasicMetadataValueEnum<'ctx>;
}

impl<'ctx> BasicValues<'ctx> for CodeGenerator<'ctx> {
    fn i8_null_ptr(&self) -> BasicMetadataValueEnum<'ctx> {
        self.context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::Generic)
            .const_null()
            .as_basic_value_enum()
            .into()
    }

    fn f64_to_f64(&self, value: f64) -> BasicMetadataValueEnum<'ctx> {
        self.double_type()
            .const_float(value)
            .as_basic_value_enum()
            .into()
    }

    fn u64_to_i32(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        self.context
            .i32_type()
            .const_int(value, false)
            .as_basic_value_enum()
            .into()
    }

    fn i64_to_i32(&self, value: i64) -> BasicMetadataValueEnum<'ctx> {
        // convert to capture negative values.
        #[allow(clippy::cast_sign_loss)]
        let target: u64 = value as u64;

        self.context
            .i32_type()
            .const_int(target, false)
            .as_basic_value_enum()
            .into()
    }

    fn u64_to_i64(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        self.int_type()
            .const_int(value, false)
            .as_basic_value_enum()
            .into()
    }
}

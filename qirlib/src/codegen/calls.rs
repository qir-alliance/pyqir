// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    builder::Builder,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, InstructionValue},
};

use super::CodeGenerator;

pub trait Calls<'ctx> {
    fn emit_void_call(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> InstructionValue<'ctx>;
    fn emit_call_with_return(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> BasicValueEnum<'ctx>;
}

impl<'ctx> Calls<'ctx> for CodeGenerator<'ctx> {
    fn emit_void_call(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> InstructionValue<'ctx> {
        self.builder
            .build_call(function, args, "")
            .try_as_basic_value()
            .right()
            .unwrap()
    }

    fn emit_call_with_return(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> BasicValueEnum<'ctx> {
        self.builder
            .build_call(function, args, name)
            .try_as_basic_value()
            .left()
            .unwrap()
    }
}

#[must_use]
pub fn emit_void_call<'ctx>(
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
) -> InstructionValue<'ctx> {
    builder
        .build_call(function, args, "")
        .try_as_basic_value()
        .right()
        .expect("Failed to create void call for target function.")
}

#[must_use]
pub fn emit_call_with_return<'ctx>(
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
    name: &str,
) -> BasicValueEnum<'ctx> {
    builder
        .build_call(function, args, name)
        .try_as_basic_value()
        .left()
        .expect("Failed to create call for target function.")
}

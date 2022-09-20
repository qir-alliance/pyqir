// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    builder::Builder,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, InstructionValue},
};

#[allow(clippy::must_use_candidate)]
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
pub(crate) fn emit_call_with_return<'ctx>(
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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qirlib::context::Context;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue};

pub(crate) fn emit_void_call<'ctx>(
    context: &Context<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
) {
    let _ = context
        .builder
        .build_call(function, args, "")
        .try_as_basic_value()
        .right()
        .unwrap();
}

pub(crate) fn emit_call_with_return<'ctx>(
    context: &Context<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
    name: &str,
) -> BasicValueEnum<'ctx> {
    context
        .builder
        .build_call(function, args, name)
        .try_as_basic_value()
        .left()
        .unwrap()
}

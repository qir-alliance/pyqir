// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue};
use qirlib::codegen::CodeGenerator;

pub(crate) fn emit_void_call<'ctx>(
    generator: &CodeGenerator<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
) {
    let _ = generator
        .builder
        .build_call(function, args, "")
        .try_as_basic_value()
        .right()
        .unwrap();
}

pub(crate) fn emit_call_with_return<'ctx>(
    generator: &CodeGenerator<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
    name: &str,
) -> BasicValueEnum<'ctx> {
    generator
        .builder
        .build_call(function, args, name)
        .try_as_basic_value()
        .left()
        .unwrap()
}

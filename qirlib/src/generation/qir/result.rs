// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::codegen::CodeGenerator;
use inkwell::values::{BasicMetadataValueEnum, IntValue, PointerValue};

pub(crate) fn get_zero<'a>(generator: &CodeGenerator<'a>) -> PointerValue<'a> {
    generator
        .emit_call_with_return(generator.rt_result_get_zero(), &[], "zero")
        .into_pointer_value()
}

pub(crate) fn get_one<'a>(generator: &CodeGenerator<'a>) -> PointerValue<'a> {
    generator
        .emit_call_with_return(generator.rt_result_get_one(), &[], "one")
        .into_pointer_value()
}

pub(crate) fn equal<'a>(
    generator: &CodeGenerator<'a>,
    result1: PointerValue<'a>,
    result2: PointerValue<'a>,
) -> IntValue<'a> {
    let result1 = BasicMetadataValueEnum::PointerValue(result1);
    let result2 = BasicMetadataValueEnum::PointerValue(result2);
    generator
        .emit_call_with_return(generator.rt_result_equal(), &[result1, result2], "equal")
        .into_int_value()
}

pub(crate) fn read_result<'a>(
    generator: &CodeGenerator<'a>,
    result: PointerValue<'a>,
) -> IntValue<'a> {
    let result = BasicMetadataValueEnum::PointerValue(result);
    generator
        .emit_call_with_return(generator.qis_read_result(), &[result], "equal")
        .into_int_value()
}

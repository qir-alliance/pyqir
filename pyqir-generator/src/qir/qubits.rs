// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicValue, BasicValueEnum};
use qirlib::codegen::CodeGenerator;

use super::calls;

pub(crate) fn emit_allocate<'ctx>(
    generator: &CodeGenerator<'ctx>,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    let args = [];
    calls::emit_call_with_return(
        &generator.builder,
        generator.runtime_library.qubit_allocate,
        &args,
        result_name,
    )
}

pub(crate) fn emit_release<'ctx>(generator: &CodeGenerator<'ctx>, qubit: &BasicValueEnum<'ctx>) {
    let args = [qubit.as_basic_value_enum().into()];
    calls::emit_void_call(
        &generator.builder,
        generator.runtime_library.qubit_release,
        &args,
    );
}

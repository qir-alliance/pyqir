// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicValue, BasicValueEnum};
use qirlib::codegen::{calls::Calls, rt::RuntimeLibrary, CodeGenerator};

pub(crate) fn emit_allocate<'ctx>(
    generator: &CodeGenerator<'ctx>,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    let args = [];
    generator.emit_call_with_return(generator.qubit_allocate(), &args, result_name)
}

pub(crate) fn emit_release<'ctx>(generator: &CodeGenerator<'ctx>, qubit: &BasicValueEnum<'ctx>) {
    let args = [qubit.as_basic_value_enum().into()];
    generator.emit_void_call(generator.qubit_release(), &args);
}

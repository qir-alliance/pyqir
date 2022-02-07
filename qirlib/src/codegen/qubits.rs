// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    builder::Builder,
    module::Module,
    values::{BasicValue, BasicValueEnum, InstructionValue},
};

use super::{
    calls::{emit_call_with_return, emit_void_call},
    rt::{qubit_allocate, qubit_release},
};

pub fn emit_allocate_qubit<'ctx>(
    context: &'ctx inkwell::context::Context,
    builder: &Builder<'ctx>,
    module: &Module<'ctx>,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    let args = [];
    emit_call_with_return(builder, qubit_allocate(context, module), &args, result_name)
}

pub fn emit_release_qubit<'ctx>(
    context: &'ctx inkwell::context::Context,
    builder: &Builder<'ctx>,
    module: &Module<'ctx>,
    qubit: &BasicValueEnum<'ctx>,
) -> InstructionValue<'ctx> {
    let args = [qubit.as_basic_value_enum().into()];
    emit_void_call(builder, qubit_release(context, module), &args)
}

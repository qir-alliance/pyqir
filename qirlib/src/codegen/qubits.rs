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

pub(crate) fn emit_allocate_qubit<'ctx>(
    context: &'ctx inkwell::context::Context,
    builder: &Builder<'ctx>,
    module: &Module<'ctx>,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    let args = [];
    let function = qubit_allocate(context, module);
    emit_call_with_return(builder, function, &args, result_name)
}

pub(crate) fn emit_release_qubit<'ctx>(
    context: &'ctx inkwell::context::Context,
    builder: &Builder<'ctx>,
    module: &Module<'ctx>,
    qubit: &BasicValueEnum<'ctx>,
) -> InstructionValue<'ctx> {
    let args = [qubit.as_basic_value_enum().into()];
    let function = qubit_release(context, module);
    emit_void_call(builder, function, &args)
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicValue, BasicValueEnum, InstructionValue};

use super::{calls::Calls, rt::RuntimeLibrary, CodeGenerator};

pub trait Qubits<'ctx> {
    fn allocate_qubit(&self, result_name: &str) -> BasicValueEnum<'ctx>;
    fn release_qubit(&self, qubit: &BasicValueEnum<'ctx>) -> InstructionValue<'ctx>;
}

impl<'ctx> Qubits<'ctx> for CodeGenerator<'ctx> {
    fn allocate_qubit(&self, result_name: &str) -> BasicValueEnum<'ctx> {
        emit_allocate(self, result_name)
    }

    fn release_qubit(&self, qubit: &BasicValueEnum<'ctx>) -> InstructionValue<'ctx> {
        emit_release(self, qubit)
    }
}

fn emit_allocate<'ctx>(generator: &CodeGenerator<'ctx>, result_name: &str) -> BasicValueEnum<'ctx> {
    let args = [];
    generator.emit_call_with_return(generator.qubit_allocate(), &args, result_name)
}

fn emit_release<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubit: &BasicValueEnum<'ctx>,
) -> InstructionValue<'ctx> {
    let args = [qubit.as_basic_value_enum().into()];
    generator.emit_void_call(generator.qubit_release(), &args)
}

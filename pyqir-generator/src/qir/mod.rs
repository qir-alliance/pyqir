// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::FunctionValue;
use qirlib::context::Context;

pub mod array1d;
pub mod basic_values;
pub mod calls;
pub mod instructions;
pub mod qubits;

pub(crate) fn get_entry_function<'ctx>(context: &Context<'ctx>) -> FunctionValue<'ctx> {
    let ns = "QuantumApplication";
    let method = "Run";
    let entrypoint_name = format!("{}__{}__body", ns, method);
    let entrypoint = context.module.get_function(&entrypoint_name).unwrap();

    while let Some(basic_block) = entrypoint.get_last_basic_block() {
        unsafe {
            basic_block.delete().unwrap();
        }
    }
    entrypoint
}

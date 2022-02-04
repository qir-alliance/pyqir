// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{module::Module, values::FunctionValue};

pub mod instructions;
pub mod qubits;

pub(crate) fn get_entry_function<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let ns = "QuantumApplication";
    let method = "Run";
    let entrypoint_name = format!("{}__{}__body", ns, method);
    let entrypoint = module.get_function(&entrypoint_name).unwrap();

    while let Some(basic_block) = entrypoint.get_last_basic_block() {
        unsafe {
            basic_block.delete().unwrap();
        }
    }
    entrypoint
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::BasicValue;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

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

pub(crate) fn remove_quantumapplication_run<'ctx>(context: &Context<'ctx>) -> FunctionValue<'ctx> {
    let ns = "QuantumApplication";
    let method = "Run";
    let entrypoint_name = format!("{}__{}", ns, method);
    let entrypoint = context.module.get_function(&entrypoint_name).unwrap();
    while let Some(basic_block) = entrypoint.get_last_basic_block() {
        unsafe {
            basic_block.delete().unwrap();
        }
    }
    entrypoint
}
pub(crate) fn remove_quantumapplication_run_interop<'ctx>(
    context: &Context<'ctx>,
) -> FunctionValue<'ctx> {
    let ns = "QuantumApplication";
    let method = "Run";
    let entrypoint_name = format!("{}__{}__Interop", ns, method);
    let entrypoint = context.module.get_function(&entrypoint_name).unwrap();
    while let Some(basic_block) = entrypoint.get_last_basic_block() {
        unsafe {
            basic_block.delete().unwrap();
        }
    }
    let entry = context.context.append_basic_block(entrypoint, "entry");
    context.builder.position_at_end(entry);

    let v = entrypoint
        .get_type()
        .ptr_type(AddressSpace::Generic)
        .const_null()
        .as_basic_value_enum();
    context.builder.build_return(Some(&v));
    entrypoint
}

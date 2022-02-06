// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{attributes::AttributeLoc, module::Module, values::FunctionValue};

pub mod instructions;

pub(crate) fn create_entrypoint_function<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> Result<FunctionValue<'ctx>, String> {
    let ns = "QuantumApplication";
    let method = "Run";
    let entrypoint_name = format!("{}__{}__body", ns, method);

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function(entrypoint_name.as_str(), fn_type, None);

    let entrypoint_attribute = context.create_string_attribute("EntryPoint", "");
    fn_value.add_attribute(AttributeLoc::Function, entrypoint_attribute);

    let entrypoint = module
        .get_function(&entrypoint_name)
        .ok_or("Could not create entrypoint.")?;

    Ok(entrypoint)
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log;

use inkwell::module::Module;
use inkwell::values::FunctionValue;

pub fn cnot_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "cnot")
}

pub fn cz_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "cz")
}

pub fn h_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "h")
}

pub fn s_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "s")
}

pub fn s_adj<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_adj(module, "s")
}

pub fn t_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "t")
}

pub fn t_adj<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_adj(module, "t")
}

pub fn x_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "x")
}

pub fn y_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "y")
}

pub fn z_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "z")
}

pub fn rx_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "rx")
}

pub fn ry_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "ry")
}

pub fn rz_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "rz")
}

pub fn reset_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "reset")
}

pub fn m_body<'ctx>(module: &Module<'ctx>) -> Option<FunctionValue<'ctx>> {
    get_intrinsic_function_body(module, "m")
}

pub fn get_intrinsic_function_body<'ctx>(
    module: &Module<'ctx>,
    name: &str,
) -> Option<FunctionValue<'ctx>> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    get_function(module, function_name.as_str())
}

pub fn get_intrinsic_function_adj<'ctx>(
    module: &Module<'ctx>,
    name: &str,
) -> Option<FunctionValue<'ctx>> {
    let function_name = format!("__quantum__qis__{}__adj", name.to_lowercase());
    get_function(module, function_name.as_str())
}

pub fn get_function<'ctx>(
    module: &Module<'ctx>,
    function_name: &str,
) -> Option<FunctionValue<'ctx>> {
    let defined_function = module.get_function(function_name);
    match defined_function {
        None => {
            log::debug!(
                "{} global function was not defined in the module",
                function_name
            );
            None
        }
        Some(value) => Some(value),
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{module::Module, values::FunctionValue};

pub fn result_get_zero<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_function(module, "result_get_zero").expect("result_get_zero must be defined")
}

pub fn result_get_one<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_function(module, "result_get_one").expect("result_get_one must be defined")
}

pub fn result_equal<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_function(module, "result_equal").expect("result_equal must be defined")
}

pub fn qubit_allocate<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_function(module, "qubit_allocate").expect("qubit_allocate must be defined")
}

pub fn qubit_release<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_function(module, "qubit_release").expect("qubit_release must be defined")
}

pub fn get_function<'ctx>(module: &Module<'ctx>, name: &str) -> Option<FunctionValue<'ctx>> {
    let function_name = format!("__quantum__rt__{}", name);
    let defined_function = module.get_function(&function_name[..]);

    match defined_function {
        None => {
            log::debug!("{} was not defined in the module", function_name);
            None
        }
        Some(value) => Some(value),
    }
}

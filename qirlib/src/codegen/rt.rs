// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::FunctionValue;

use super::CodeGenerator;

pub trait RuntimeLibrary<'ctx> {
    fn result_get_zero(&self) -> FunctionValue<'ctx>;
    fn result_get_one(&self) -> FunctionValue<'ctx>;
    fn result_equal(&self) -> FunctionValue<'ctx>;
    fn qubit_allocate(&self) -> FunctionValue<'ctx>;
    fn qubit_release(&self) -> FunctionValue<'ctx>;
}

impl<'ctx> RuntimeLibrary<'ctx> for CodeGenerator<'ctx> {
    fn result_get_zero(&self) -> FunctionValue<'ctx> {
        get_function(self, "result_get_zero").expect("result_get_zero must be defined")
    }

    fn result_get_one(&self) -> FunctionValue<'ctx> {
        get_function(self, "result_get_one").expect("result_get_one must be defined")
    }

    fn result_equal(&self) -> FunctionValue<'ctx> {
        get_function(self, "result_equal").expect("result_equal must be defined")
    }

    fn qubit_allocate(&self) -> FunctionValue<'ctx> {
        get_function(self, "qubit_allocate").expect("qubit_allocate must be defined")
    }

    fn qubit_release(&self) -> FunctionValue<'ctx> {
        get_function(self, "qubit_release").expect("qubit_release must be defined")
    }
}

fn get_function<'ctx>(generator: &CodeGenerator<'ctx>, name: &str) -> Option<FunctionValue<'ctx>> {
    let function_name = format!("__quantum__rt__{}", name);
    let defined_function = generator.module.get_function(&function_name[..]);

    match defined_function {
        None => {
            log::debug!("{} was not defined in the module", function_name);
            None
        }
        Some(value) => Some(value),
    }
}

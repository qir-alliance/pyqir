// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    module::Module,
    passes::{PassManager, PassManagerBuilder},
    OptimizationLevel,
};

pub enum Status {
    Changed,
    NoOp,
}

pub fn run_basic(module: &Module) -> Status {
    let builder = PassManagerBuilder::create();
    builder.set_optimization_level(OptimizationLevel::None);
    let manager = PassManager::create(());
    manager.add_global_dce_pass();
    manager.add_strip_dead_prototypes_pass();
    builder.populate_module_pass_manager(&manager);
    if manager.run_on(module) {
        Status::Changed
    } else {
        Status::NoOp
    }
}

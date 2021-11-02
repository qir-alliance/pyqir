// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::context::Context;
use inkwell::passes::PassManager;
use inkwell::{
    passes::PassManagerBuilder,
    OptimizationLevel,
};

pub fn run_basic_passes_on<'ctx>(context: &Context<'ctx>) -> bool {
    let pass_manager_builder = PassManagerBuilder::create();
    pass_manager_builder.set_optimization_level(OptimizationLevel::None);
    let fpm = PassManager::create(());
    fpm.add_global_dce_pass();
    fpm.add_strip_dead_prototypes_pass();
    pass_manager_builder.populate_module_pass_manager(&fpm);
    fpm.run_on(&context.module)
}

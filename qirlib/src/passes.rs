// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::{passes::PassManagerBuilder, OptimizationLevel};

// This method returns true if any of the passes modified the function or module and false otherwise.
pub fn run_basic_passes_on(module: &Module) -> bool {
    let pass_manager_builder = PassManagerBuilder::create();
    pass_manager_builder.set_optimization_level(OptimizationLevel::None);
    let fpm = PassManager::create(());
    // TODO: This breaks PyQIR JIT.
    // fpm.add_global_dce_pass();
    fpm.add_strip_dead_prototypes_pass();
    pass_manager_builder.populate_module_pass_manager(&fpm);
    fpm.run_on(module)
}

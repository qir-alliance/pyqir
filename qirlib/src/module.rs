// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    module::Module,
    passes::{PassManager, PassManagerBuilder},
    OptimizationLevel,
};

#[allow(clippy::missing_errors_doc)]
pub fn simple_finalize(module: &Module) -> Result<(), String> {
    run_basic_passes(module);
    module.verify().map_err(|e| e.to_string())
}

// This method returns true if any of the passes modified the function or module and false otherwise.
pub(crate) fn run_basic_passes(module: &Module) -> bool {
    let builder = PassManagerBuilder::create();
    builder.set_optimization_level(OptimizationLevel::None);
    let manager = PassManager::create(());
    manager.add_global_dce_pass();
    manager.add_strip_dead_prototypes_pass();
    builder.populate_module_pass_manager(&manager);
    manager.run_on(module)
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_reference_ir;

    #[test]
    fn zero_required_qubits_results() -> Result<(), String> {
        assert_reference_ir("module/zero_required_qubits_results", 0, 0, |_| ())
    }

    #[test]
    fn one_required_qubit() -> Result<(), String> {
        assert_reference_ir("module/one_required_qubit", 1, 0, |_| ())
    }

    #[test]
    fn one_required_result() -> Result<(), String> {
        assert_reference_ir("module/one_required_result", 0, 1, |_| ())
    }

    #[test]
    fn many_required_qubits_results() -> Result<(), String> {
        assert_reference_ir("module/many_required_qubits_results", 5, 7, |_| ())
    }
}

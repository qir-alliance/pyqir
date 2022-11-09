// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    attributes::AttributeLoc,
    builder::Builder,
    module::Module,
    passes::{PassManager, PassManagerBuilder},
    values::FunctionValue,
    OptimizationLevel,
};

pub fn simple_init(
    module: &Module,
    builder: &Builder,
    required_num_qubits: u64,
    required_num_results: u64,
) {
    let context = module.get_context();
    let entry_point = create_entry_point(module);
    add_num_attribute(entry_point, "requiredQubits", required_num_qubits);
    add_num_attribute(entry_point, "requiredResults", required_num_results);
    let entry = context.append_basic_block(entry_point, "entry");
    builder.position_at_end(entry);
}

#[allow(clippy::missing_errors_doc)]
pub fn simple_finalize(module: &Module) -> Result<(), String> {
    run_basic_passes(module);
    module.verify().map_err(|e| e.to_string())
}

// This method returns true if any of the passes modified the function or module and false otherwise.
pub(crate) fn run_basic_passes(module: &Module) -> bool {
    let pass_manager_builder = PassManagerBuilder::create();
    pass_manager_builder.set_optimization_level(OptimizationLevel::None);
    let fpm = PassManager::create(());
    fpm.add_global_dce_pass();
    fpm.add_strip_dead_prototypes_pass();
    pass_manager_builder.populate_module_pass_manager(&fpm);
    fpm.run_on(module)
}

fn create_entry_point<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let context = module.get_context();
    let fn_type = context.void_type().fn_type(&[], false);
    let fn_value = module.add_function("main", fn_type, None);

    let entry_point_attribute = context.create_string_attribute("EntryPoint", "");
    fn_value.add_attribute(AttributeLoc::Function, entry_point_attribute);
    fn_value
}

fn add_num_attribute(function: FunctionValue, key: &str, value: u64) {
    let context = function.get_type().get_context();
    let attribute = context.create_string_attribute(key, &value.to_string());
    function.add_attribute(AttributeLoc::Function, attribute);
}

#[cfg(test)]
mod tests {
    use crate::{module::create_entry_point, tests::assert_reference_ir};
    use inkwell::context::Context;

    #[test]
    fn entry_point_function_has_correct_signature_and_default_attribute() {
        let context = Context::create();
        let module = context.create_module("test");
        let builder = context.create_builder();

        let entry_point = create_entry_point(&module);
        let entry = context.append_basic_block(entry_point, "entry");
        builder.position_at_end(entry);
        builder.build_return(None);
        let ir_string = module.print_to_string().to_string();
        let expected = "; ModuleID = 'test'\nsource_filename = \"test\"\n\ndefine void @main() #0 {\nentry:\n  ret void\n}\n\nattributes #0 = { \"EntryPoint\" }\n";
        assert_eq!(expected, ir_string);
    }

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

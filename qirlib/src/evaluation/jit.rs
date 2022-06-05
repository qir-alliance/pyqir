// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::evaluation::{
    interop::SemanticModel,
    intrinsics::{reset_max_qubit_id, reset_static_result_cache, set_measure_stream},
    runtime::Simulator,
};
use crate::{module, passes::run_basic_passes_on};
use bitvec::prelude::BitVec;
use inkwell::{
    attributes::AttributeLoc,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    targets::{InitializationConfig, Target, TargetMachine},
    values::FunctionValue,
    OptimizationLevel,
};
use std::path::Path;

/// # Errors
///
/// - Path has an unsupported extension.
/// - Module fails to load.
/// - LLVM fails to initialize local JIT Engine and components
/// - Entrypoint cannot be resolved
/// - Module contains unknown external functions
pub fn run_module_file(
    path: impl AsRef<Path>,
    entry_point: Option<&str>,
    result_stream: Option<BitVec>,
) -> Result<SemanticModel, String> {
    let context = Context::create();
    let module = module::load_file(path, &context)?;
    run_module(&module, entry_point, result_stream)
}

/// # Errors
///
/// - LLVM fails to initialize local JIT Engine and components
/// - Entrypoint cannot be resolved
/// - Module contains unknown external functions
pub fn run_module(
    module: &Module,
    entry_point: Option<&str>,
    result_stream: Option<BitVec>,
) -> Result<SemanticModel, String> {
    Target::initialize_native(&InitializationConfig::default())?;

    let default_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&default_triple).map_err(|e| e.to_string())?;

    if !target.has_asm_backend() {
        return Err("Target doesn't have an ASM backend.".to_owned());
    }
    if !target.has_target_machine() {
        return Err("Target doesn't have a target machine.".to_owned());
    }

    run_basic_passes_on(module);
    let entry_point = choose_entry_point(module_functions(module), entry_point)?;

    // load the symbols for the current process (empty/null string)
    inkwell::support::load_library_permanently("");

    reset_max_qubit_id();
    reset_static_result_cache();

    set_measure_stream(&result_stream.unwrap_or_default());

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| e.to_string())?;

    let _simulator = Simulator::new(module, &execution_engine)?;

    unsafe {
        run_entry_point(&execution_engine, entry_point)?;
    }

    Ok(Simulator::get_model())
}

unsafe fn run_entry_point(
    execution_engine: &ExecutionEngine,
    entry_point: FunctionValue,
) -> Result<(), String> {
    if entry_point.count_params() == 0 && entry_point.get_type().get_return_type().is_none() {
        execution_engine.run_function(entry_point, &[]);
        Ok(())
    } else {
        Err("Entry point has parameters or a non-void return type.".to_owned())
    }
}

fn choose_entry_point<'ctx>(
    functions: impl Iterator<Item = FunctionValue<'ctx>>,
    name: Option<&str>,
) -> Result<FunctionValue<'ctx>, String> {
    let mut entry_points = functions
        .filter(|f| is_entry_point(*f) && name.iter().all(|n| f.get_name().to_str() == Ok(n)));

    let entry_point = entry_points
        .next()
        .ok_or_else(|| "No matching entry point found.".to_owned())?;

    if entry_points.next().is_some() {
        Err("Multiple matching entry points found.".to_owned())
    } else {
        Ok(entry_point)
    }
}

fn is_entry_point(function: FunctionValue) -> bool {
    function
        .get_string_attribute(AttributeLoc::Function, "EntryPoint")
        .is_some()
}

pub fn module_functions<'ctx>(module: &Module<'ctx>) -> impl Iterator<Item = FunctionValue<'ctx>> {
    struct FunctionValueIter<'ctx>(Option<FunctionValue<'ctx>>);

    impl<'ctx> Iterator for FunctionValueIter<'ctx> {
        type Item = FunctionValue<'ctx>;

        fn next(&mut self) -> Option<Self::Item> {
            let function = self.0;
            self.0 = function.and_then(inkwell::values::FunctionValue::get_next_function);
            function
        }
    }

    FunctionValueIter(module.get_first_function())
}

#[cfg(test)]
mod tests {
    use super::{run_module, run_module_file};
    use crate::evaluation::interop::{Instruction, SemanticModel, Single};
    use crate::module;
    use inkwell::context::Context;
    use serial_test::serial;
    use std::path::PathBuf;

    const BELL_QIR_MEASURE: &[u8] = include_bytes!("../../resources/tests/bell_qir_measure.bc");
    const CUSTOM_ENTRY_POINT_NAME: &[u8] =
        include_bytes!("../../resources/tests/custom_entry_point_name.bc");
    const MULTIPLE_ENTRY_POINTS: &[u8] =
        include_bytes!("../../resources/tests/multiple_entry_points.bc");
    const ENTRY_POINT_TYPES: &[u8] = include_bytes!("../../resources/tests/entry_point_types.bc");

    #[serial]
    #[test]
    fn runs_bell_qir_measure() -> Result<(), String> {
        let model = run_test_module(BELL_QIR_MEASURE, None)?;
        assert_eq!(model.instructions.len(), 4);
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_single_entry_point_with_custom_name() -> Result<(), String> {
        let model = run_test_module(CUSTOM_ENTRY_POINT_NAME, None)?;
        assert_eq!(
            model.instructions,
            vec![Instruction::X(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_entry_point_by_name() -> Result<(), String> {
        let model = run_test_module(CUSTOM_ENTRY_POINT_NAME, Some("App__Foo"))?;
        assert_eq!(
            model.instructions,
            vec![Instruction::X(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_wrong_name_single_entry_point() -> Result<(), String> {
        let result = run_test_module(CUSTOM_ENTRY_POINT_NAME, Some("nonexistent"));
        assert_eq!(
            result.err(),
            Some("No matching entry point found.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_without_name_if_multiple_entry_points() -> Result<(), String> {
        let result = run_test_module(MULTIPLE_ENTRY_POINTS, None);
        assert_eq!(
            result.err(),
            Some("Multiple matching entry points found.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_first_entry_point_by_name() -> Result<(), String> {
        let model = run_test_module(MULTIPLE_ENTRY_POINTS, Some("App__Foo"))?;
        assert_eq!(
            model.instructions,
            vec![Instruction::X(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_second_entry_point_by_name() -> Result<(), String> {
        let model = run_test_module(MULTIPLE_ENTRY_POINTS, Some("App__Bar"))?;
        assert_eq!(
            model.instructions,
            vec![Instruction::H(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_wrong_name_multiple_entry_points() -> Result<(), String> {
        let result = run_test_module(MULTIPLE_ENTRY_POINTS, Some("nonexistent"));
        assert_eq!(
            result.err(),
            Some("No matching entry point found.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_entry_point_has_params() -> Result<(), String> {
        let result = run_test_module(ENTRY_POINT_TYPES, Some("App__IntParam"));
        assert_eq!(
            result.err(),
            Some("Entry point has parameters or a non-void return type.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_entry_point_has_return_value() -> Result<(), String> {
        let result = run_test_module(ENTRY_POINT_TYPES, Some("App__IntReturn"));
        assert_eq!(
            result.err(),
            Some("Entry point has parameters or a non-void return type.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_unknown_external_func() -> Result<(), String> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources");
        path.push("tests");
        path.push("test_unknown_external_func");
        path.set_extension("ll");

        let result = run_module_file(path, None, None);
        assert_eq!(
            result.err(),
            Some("Unsupported function `__quantum__rt__bool_to_string`.".to_owned())
        );
        Ok(())
    }

    fn run_test_module(bytes: &[u8], entry_point: Option<&str>) -> Result<SemanticModel, String> {
        let context = Context::create();
        let module = module::load_memory(bytes, "test", &context)?;
        run_module(&module, entry_point, None)
    }
}

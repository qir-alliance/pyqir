// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{interop::SemanticModel, runtime::Simulator};
use inkwell::{
    attributes::AttributeLoc,
    module::Module,
    targets::{InitializationConfig, Target, TargetMachine},
    values::FunctionValue,
    OptimizationLevel,
};
use microsoft_quantum_qir_runtime_sys::runtime::BasicRuntimeDriver;
use qirlib::{
    context::{BareContext, ContextType},
    passes::run_basic_passes_on,
};
use std::path::Path;

pub fn run_module_file(
    path: impl AsRef<Path>,
    entry_point: Option<&str>,
) -> Result<SemanticModel, String> {
    let ctx = inkwell::context::Context::create();
    let path_str = path
        .as_ref()
        .to_str()
        .expect("Did not find a valid Unicode path string")
        .to_owned();
    let context_type = ContextType::File(&path_str);
    let context = BareContext::new(&ctx, context_type)?;
    let model = run_module(&context.module, entry_point)?;
    Ok(model)
}

pub fn run_module<'ctx>(
    module: &Module<'ctx>,
    entry_point: Option<&str>,
) -> Result<SemanticModel, String> {
    Target::initialize_native(&InitializationConfig::default()).unwrap();
    let default_triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&default_triple).expect("Unable to create target machine");
    assert!(target.has_asm_backend());
    assert!(target.has_target_machine());

    run_basic_passes_on(&module);
    let entry_point = choose_entry_point(module_functions(module), entry_point)?;

    unsafe {
        BasicRuntimeDriver::initialize_qir_context(true);
        let _ = microsoft_quantum_qir_runtime_sys::foundation::QSharpFoundation::new();
        let _ = inkwell::support::load_library_permanently("");
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("Could not create JIT Engine");
        let simulator = Simulator::new(&module, &execution_engine);
        let main = execution_engine
            .get_function::<unsafe extern "C" fn() -> ()>(&entry_point)
            .unwrap();

        main.call();
        Ok(simulator.get_model())
    }
}

fn choose_entry_point<'ctx>(
    functions: impl Iterator<Item = FunctionValue<'ctx>>,
    name: Option<&str>,
) -> Result<String, String> {
    let mut entry_points = functions
        .filter(is_entry_point)
        .map(|f| f.get_name().to_str().unwrap().to_owned())
        .filter(|function_name| name.iter().all(|n| function_name == n));

    let entry_point = entry_points
        .next()
        .ok_or("No matching entry point found.".to_owned())?;

    if entry_points.next().is_some() {
        Err("Multiple matching entry points found.".to_owned())
    } else {
        Ok(entry_point)
    }
}

fn is_entry_point(function: &FunctionValue) -> bool {
    function
        .get_string_attribute(AttributeLoc::Function, "EntryPoint")
        .is_some()
}

fn module_functions<'ctx>(module: &Module<'ctx>) -> impl Iterator<Item = FunctionValue<'ctx>> {
    struct FunctionValueIter<'ctx>(Option<FunctionValue<'ctx>>);

    impl<'ctx> Iterator for FunctionValueIter<'ctx> {
        type Item = FunctionValue<'ctx>;

        fn next(&mut self) -> Option<Self::Item> {
            let function = self.0;
            self.0 = function.and_then(|f| f.get_next_function());
            function
        }
    }

    FunctionValueIter(module.get_first_function())
}

#[cfg(test)]
mod tests {
    use super::run_module_file;
    use std::{fs::File, io::Write};
    use tempfile::tempdir;

    #[test]
    fn eval_test() -> Result<(), String> {
        let bell_qir_measure_contents = include_bytes!("../tests/bell_qir_measure.ll");
        let dir = tempdir().expect("Could not create temp dir");
        let file_path = dir.path().join("bell_qir_measure.ll");
        let mut buffer = File::create(&file_path).unwrap();
        buffer.write_all(bell_qir_measure_contents).unwrap();

        let generated_model = run_module_file(file_path, None)?;
        assert_eq!(generated_model.instructions.len(), 2);
        Ok(())
    }

    #[test]
    fn eval_entry_points_test() {
        let custom_entry_point_name = "tests/custom_entry_point_name.ll";
        run_module_file(custom_entry_point_name, None).unwrap();
        run_module_file(custom_entry_point_name, Some("App__Foo")).unwrap();
        assert!(run_module_file(custom_entry_point_name, Some("nonexistent")).is_err());

        let multiple_entry_points = "tests/multiple_entry_points.ll";
        assert!(run_module_file(multiple_entry_points, None).is_err());
        run_module_file(multiple_entry_points, Some("App__Foo")).unwrap();
        run_module_file(multiple_entry_points, Some("App__Bar")).unwrap();
        assert!(run_module_file(multiple_entry_points, Some("nonexistent")).is_err());
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{interop::SemanticModel, runtime::Simulator};
use inkwell::{
    attributes::AttributeLoc,
    execution_engine::ExecutionEngine,
    module::Module,
    targets::{InitializationConfig, Target, TargetMachine},
    values::FunctionValue,
    OptimizationLevel,
};
use microsoft_quantum_qir_runtime_sys::runtime::BasicRuntimeDriver;

use qirlib::{context::ModuleSource, module, passes::run_basic_passes_on};
use std::path::Path;

/// # Panics
///
/// Path to module was not a valid unicode path string
/// # Errors
///
/// Will return `Err` if
///   - Module fails to load
///   - LLVM native target fails to initialize.
///   - Unable to create target machine
///   - Target doesn't have an ASM backend.
///   - Target doesn't have a target machine
///   - No matching entry point found.
///   - Multiple matching entry points found.
///   - JIT Engine could not created.
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

    let module_source = ModuleSource::File(&path_str);
    let module = module::load(&ctx, module_source)?;
    run_module(&module, entry_point)
}

/// # Errors
///
/// Will return `Err` if
///   - LLVM native target fails to initialize.
///   - Unable to create target machine
///   - Target doesn't have an ASM backend.
///   - Target doesn't have a target machine
///   - No matching entry point found.
///   - Multiple matching entry points found.
///   - JIT Engine could not created.
pub fn run_module(module: &Module<'_>, entry_point: Option<&str>) -> Result<SemanticModel, String> {
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

    unsafe {
        BasicRuntimeDriver::initialize_qir_context(true);
        microsoft_quantum_qir_runtime_sys::foundation::QSharpFoundation::new();
        inkwell::support::load_library_permanently("");
    }

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| e.to_string())?;

    let _simulator = Simulator::new(module, &execution_engine);

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
        .filter(is_entry_point)
        .filter(|f| name.iter().all(|n| f.get_name().to_str() == Ok(n)));

    let entry_point = entry_points
        .next()
        .ok_or_else(|| "No matching entry point found.".to_owned())?;

    if entry_points.next().is_some() {
        Err("Multiple matching entry points found.".to_owned())
    } else {
        Ok(entry_point)
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
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
            self.0 = function.and_then(inkwell::values::FunctionValue::get_next_function);
            function
        }
    }

    FunctionValueIter(module.get_first_function())
}

#[cfg(test)]
mod tests {
    use super::run_module_file;
    use crate::interop::{Instruction, Single};
    use serial_test::serial;
    use std::io::{self, Write};
    use tempfile::NamedTempFile;

    const BELL_QIR_MEASURE: &[u8] = include_bytes!("../tests/bell_qir_measure.bc");
    const CUSTOM_ENTRY_POINT_NAME: &[u8] = include_bytes!("../tests/custom_entry_point_name.bc");
    const MULTIPLE_ENTRY_POINTS: &[u8] = include_bytes!("../tests/multiple_entry_points.bc");
    const ENTRY_POINT_TYPES: &[u8] = include_bytes!("../tests/entry_point_types.bc");

    #[serial]
    #[test]
    fn runs_bell_qir_measure() -> Result<(), String> {
        let module_file = temp_bc_file(BELL_QIR_MEASURE).map_err(|e| e.to_string())?;
        let generated_model = run_module_file(&module_file, None)?;
        assert_eq!(generated_model.instructions.len(), 2);
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_single_entry_point_with_custom_name() -> Result<(), String> {
        let module_file = temp_bc_file(CUSTOM_ENTRY_POINT_NAME).map_err(|e| e.to_string())?;
        let model = run_module_file(&module_file, None)?;
        assert_eq!(
            model.instructions,
            vec![Instruction::X(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_entry_point_by_name() -> Result<(), String> {
        let module_file = temp_bc_file(CUSTOM_ENTRY_POINT_NAME).map_err(|e| e.to_string())?;
        let model = run_module_file(&module_file, Some("App__Foo"))?;
        assert_eq!(
            model.instructions,
            vec![Instruction::X(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_wrong_name_single_entry_point() -> Result<(), String> {
        let module_file = temp_bc_file(CUSTOM_ENTRY_POINT_NAME).map_err(|e| e.to_string())?;
        assert_eq!(
            run_module_file(&module_file, Some("nonexistent")).err(),
            Some("No matching entry point found.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_without_name_if_multiple_entry_points() -> Result<(), String> {
        let module_file = temp_bc_file(MULTIPLE_ENTRY_POINTS).map_err(|e| e.to_string())?;
        assert_eq!(
            run_module_file(&module_file, None).err(),
            Some("Multiple matching entry points found.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_first_entry_point_by_name() -> Result<(), String> {
        let module_file = temp_bc_file(MULTIPLE_ENTRY_POINTS).map_err(|e| e.to_string())?;
        let model = run_module_file(&module_file, Some("App__Foo"))?;
        assert_eq!(
            model.instructions,
            vec![Instruction::X(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn runs_second_entry_point_by_name() -> Result<(), String> {
        let module_file = temp_bc_file(MULTIPLE_ENTRY_POINTS).map_err(|e| e.to_string())?;
        let model = run_module_file(&module_file, Some("App__Bar"))?;
        assert_eq!(
            model.instructions,
            vec![Instruction::H(Single::new("0".to_owned()))]
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_wrong_name_multiple_entry_points() -> Result<(), String> {
        let module_file = temp_bc_file(MULTIPLE_ENTRY_POINTS).map_err(|e| e.to_string())?;
        assert_eq!(
            run_module_file(&module_file, Some("nonexistent")).err(),
            Some("No matching entry point found.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_entry_point_has_params() -> Result<(), String> {
        let module_file = temp_bc_file(ENTRY_POINT_TYPES).map_err(|e| e.to_string())?;
        assert_eq!(
            run_module_file(&module_file, Some("App__IntParam")).err(),
            Some("Entry point has parameters or a non-void return type.".to_owned())
        );
        Ok(())
    }

    #[serial]
    #[test]
    fn fails_if_entry_point_has_return_value() -> Result<(), String> {
        let module_file = temp_bc_file(ENTRY_POINT_TYPES).map_err(|e| e.to_string())?;
        assert_eq!(
            run_module_file(&module_file, Some("App__IntReturn")).err(),
            Some("Entry point has parameters or a non-void return type.".to_owned())
        );
        Ok(())
    }

    fn temp_bc_file(buf: &[u8]) -> io::Result<NamedTempFile> {
        let mut temp_file = tempfile::Builder::new().suffix(".bc").tempfile()?;
        temp_file.write_all(buf)?;
        Ok(temp_file)
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::Path;

use crate::interop::SemanticModel;
use crate::runtime::Simulator;
use inkwell::targets::TargetMachine;
use inkwell::targets::{InitializationConfig, Target};
use microsoft_quantum_qir_runtime_sys::runtime::BasicRuntimeDriver;
use qirlib::context::{Context, ModuleType};
use qirlib::passes::run_basic_passes_on;

/// # Errors
///
/// Will return `Err` if module fails to load
pub fn run_module<P: AsRef<Path>>(path: P) -> Result<SemanticModel, String> {
    let ctx = inkwell::context::Context::create();
    let path_str = path
        .as_ref()
        .to_str()
        .expect("Did not find a valid Unicode path string")
        .to_owned();
    let context_type = ModuleType::File(&path_str);
    let context = Context::new(&ctx, context_type)?;
    let model = run_ctx(&context)?;
    Ok(model)
}

/// # Errors
///
/// Will return `Err` if LLVM native target fails to initialize
pub fn run_ctx(context: &Context<'_>) -> Result<SemanticModel, String> {
    Target::initialize_native(&InitializationConfig::default())?;

    let default_triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&default_triple).expect("Unable to create target machine");

    if !target.has_asm_backend() {
        return Err("Target doesn't have an ASM backend.".to_owned());
    }
    if !target.has_target_machine() {
        return Err("Target doesn't have a target machine.".to_owned());
    }

    run_basic_passes_on(context);

    unsafe {
        BasicRuntimeDriver::initialize_qir_context(true);
        let _ = microsoft_quantum_qir_runtime_sys::foundation::QSharpFoundation::new();

        let _ = inkwell::support::load_library_permanently("");
        let _simulator = Simulator::new(context, &context.execution_engine);
        let main = context
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> ()>("QuantumApplication__Run")
            .expect("Could not load entrypoint.");
        main.call();
        Ok(Simulator::get_model())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn eval_test() -> Result<(), String> {
        let bell_qir_measure_contents = include_bytes!("../tests/bell_qir_measure.ll");
        let dir = tempdir().expect("Could not create temp dir");
        let file_path = dir.path().join("bell_qir_measure.ll");
        let mut buffer = File::create(&file_path).unwrap();
        buffer.write_all(bell_qir_measure_contents).unwrap();

        let generated_model = super::run_module(file_path)?;

        assert_eq!(generated_model.instructions.len(), 2);
        Ok(())
    }
}

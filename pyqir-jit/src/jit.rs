// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{interop::SemanticModel, runtime::Simulator};
use inkwell::{
    module::Module,
    targets::{InitializationConfig, Target, TargetMachine},
    OptimizationLevel,
};
use microsoft_quantum_qir_runtime_sys::runtime::BasicRuntimeDriver;
use qirlib::{
    context::{BareContext, ContextType},
    passes::run_basic_passes_on,
};

pub fn run_context_module(context_type: ContextType) -> Result<SemanticModel, String> {
    let ctx = inkwell::context::Context::create();
    let context = BareContext::new(&ctx, context_type)?;
    run_module(&context.module)
}

pub fn run_module<'ctx>(module: &Module<'ctx>) -> Result<SemanticModel, String> {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let default_triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&default_triple).expect("Unable to create target machine");

    assert!(target.has_asm_backend());
    assert!(target.has_target_machine());

    run_basic_passes_on(&module);

    unsafe {
        BasicRuntimeDriver::initialize_qir_context(true);
        let _ = microsoft_quantum_qir_runtime_sys::foundation::QSharpFoundation::new();

        let _ = inkwell::support::load_library_permanently("");
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("Could not create JIT Engine");
        let simulator = Simulator::new(&module, &execution_engine);
        let main = execution_engine
            .get_function::<unsafe extern "C" fn() -> ()>("QuantumApplication__Run")
            .unwrap();
        main.call();
        Ok(simulator.get_model())
    }
}

#[cfg(test)]
mod tests {
    use qirlib::context::ContextType;
    use std::{fs::File, io::Write};
    use tempfile::tempdir;

    #[test]
    fn eval_test() -> Result<(), String> {
        let bell_qir_measure_contents = include_bytes!("../tests/bell_qir_measure.ll");
        let dir = tempdir().expect("Could not create temp dir");
        let file_path = dir.path().join("bell_qir_measure.ll");
        let mut buffer = File::create(&file_path).unwrap();
        buffer.write_all(bell_qir_measure_contents).unwrap();

        let generated_model = super::run_context_module(ContextType::File(&file_path))?;

        assert_eq!(generated_model.instructions.len(), 2);
        Ok(())
    }
}

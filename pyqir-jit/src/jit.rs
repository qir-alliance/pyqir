// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qirlib::context::{Context, ContextType};
use crate::runtime::Simulator;
use crate::interop::SemanticModel;
use inkwell::targets::TargetMachine;
use inkwell::{
    passes::PassManagerBuilder,
    targets::{InitializationConfig, Target},
    OptimizationLevel,
};
use microsoft_quantum_qir_runtime_sys::runtime::BasicRuntimeDriver;
use qirlib::passes::run_basic_passes_on;

pub fn run_module(module: String) -> Result<SemanticModel, String> {
    let ctx = inkwell::context::Context::create();
    let context_type = ContextType::File(&module);
    let context = Context::new(&ctx, context_type)?;
    let model = run_ctx(context)?;
    Ok(model)
}

pub fn run_ctx<'ctx>(context: Context<'ctx>) -> Result<SemanticModel, String> {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let default_triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&default_triple).expect("Unable to create target machine");

    assert!(target.has_asm_backend());
    assert!(target.has_target_machine());

    run_basic_passes_on(&context);

    unsafe {
        BasicRuntimeDriver::initialize_qir_context(true);
        let _ = microsoft_quantum_qir_runtime_sys::foundation::QSharpFoundation::new();

        let _ = inkwell::support::load_library_permanently("");
        let simulator = Simulator::new(&context, &context.execution_engine);
        let main = context
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> ()>("QuantumApplication__Run")
            .unwrap();
        main.call();
        Ok(simulator.get_model())
    }
}

#[cfg(test)]
mod tests {

    use crate::interop::{ClassicalRegister, Measured, QuantumRegister, SemanticModel};
    use crate::interop::{Controlled, Instruction, Single};
    use tempfile::tempdir;
    use super::run_ctx;

    #[ignore = "CI Requires runtime recompilation"]
    #[test]
    fn eval_test() -> Result<(), String> {
        let dir = tempdir().expect("");
        let tmp_path = dir.into_path();

        let name = String::from("Bell circuit");
        let mut model = SemanticModel::new(name);
        model.add_reg(QuantumRegister::new(String::from("qr"), 0).as_register());
        model.add_reg(QuantumRegister::new(String::from("qr"), 1).as_register());
        model.add_reg(ClassicalRegister::new(String::from("qc"), 2).as_register());

        model.add_inst(Instruction::H(Single::new(String::from("qr0"))));
        model.add_inst(Instruction::Cx(Controlled::new(
            String::from("qr0"),
            String::from("qr1"),
        )));

        model.add_inst(Instruction::M(Measured::new(
            String::from("qr0"),
            String::from("qc0"),
        )));
        model.add_inst(Instruction::M(Measured::new(
            String::from("qr1"),
            String::from("qc1"),
        )));

        let generated_model = run(&model)?;

        assert!(generated_model.instructions.len() == 2);
        Ok(())
    }

    pub fn run(model: &SemanticModel) -> Result<SemanticModel, String> {
        //let ctx = inkwell::context::Context::create();
        //let context = pyqir_generator::populate_context(&ctx, &model).unwrap();
        //let model = run_ctx(context)?;
        Ok(model.clone())
    }
}

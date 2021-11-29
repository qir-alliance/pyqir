// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{interop::SemanticModel, runtime::Simulator};
use inkwell::{
    attributes::AttributeLoc,
    module::Module,
    targets::{InitializationConfig, Target, TargetMachine},
    values::FunctionValue,
};
use microsoft_quantum_qir_runtime_sys::runtime::BasicRuntimeDriver;
use qirlib::{
    context::{Context, ContextType},
    passes::run_basic_passes_on,
};

pub fn run_module(module: String) -> Result<SemanticModel, String> {
    let ctx = inkwell::context::Context::create();
    let context_type = ContextType::File(&module);
    let context = Context::new(&ctx, context_type)?;
    run_ctx(context)
}

pub fn run_ctx<'ctx>(context: Context<'ctx>) -> Result<SemanticModel, String> {
    Target::initialize_native(&InitializationConfig::default()).unwrap();
    let default_triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&default_triple).expect("Unable to create target machine");
    assert!(target.has_asm_backend());
    assert!(target.has_target_machine());

    run_basic_passes_on(&context);

    let entry_point = module_functions(&context.module)
        .filter(is_entry_point)
        .next()
        .ok_or("Module contains no matching entry point.")?;

    let entry_point_name = entry_point.get_name().to_str().unwrap();

    unsafe {
        BasicRuntimeDriver::initialize_qir_context(true);
        let _ = microsoft_quantum_qir_runtime_sys::foundation::QSharpFoundation::new();
        let _ = inkwell::support::load_library_permanently("");
        let simulator = Simulator::new(&context, &context.execution_engine);

        let main = context
            .execution_engine
            .get_function::<unsafe extern "C" fn() -> ()>(entry_point_name)
            .unwrap();

        main.call();
        Ok(simulator.get_model())
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
    use super::run_module;
    use crate::interop::{ClassicalRegister, Measured, QuantumRegister, SemanticModel};
    use crate::interop::{Controlled, Instruction, Single};
    use tempfile::tempdir;

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

    #[test]
    fn eval_my_qir_file() {
        let path = "C:\\Users\\samarsha\\Code\\samarsha\\qsharp-sandbox\\App\\qir\\App.ll";
        run_module(path.to_string()).unwrap();
    }
}

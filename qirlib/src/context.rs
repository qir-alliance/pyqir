// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::OptimizationLevel;

use std::path::Path;

use crate::{
    constants::Constants, intrinsics::Intrinsics, module, runtime_library::RuntimeLibrary,
    types::Types,
};

pub struct BareContext<'ctx> {
    pub context: &'ctx inkwell::context::Context,
    pub module: inkwell::module::Module<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx> BareContext<'ctx> {
    pub fn new(
        context: &'ctx inkwell::context::Context,
        context_type: ContextType<'ctx>,
    ) -> Result<Self, String> {
        let builder = context.create_builder();
        let module = module::load_module(context, context_type)?;
        Ok(BareContext {
            builder,
            module,
            context,
        })
    }
}

pub struct Context<'ctx> {
    pub context: &'ctx inkwell::context::Context,
    pub module: inkwell::module::Module<'ctx>,
    #[cfg(feature = "jit")]
    pub execution_engine: inkwell::execution_engine::ExecutionEngine<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
    pub types: Types<'ctx>,
    pub runtime_library: RuntimeLibrary<'ctx>,
    pub intrinsics: Intrinsics<'ctx>,
    pub constants: Constants<'ctx>,
}

#[derive(Clone, Copy)]
pub enum ModuleType<'ctx> {
    Template(&'ctx String),
    File(&'ctx String),
}

#[cfg(feature = "jit")]
impl<'ctx> Context<'ctx> {
    /// # Errors
    ///
    /// Will return `Err` if module fails to load or LLVM native target fails to initialize
    pub fn new(
        context: &'ctx inkwell::context::Context,
        context_type: ModuleType<'ctx>,
    ) -> Result<Self, String> {
        let builder = context.create_builder();
        let module = module::load_module(context, context_type)?;
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("Could not create JIT Engine");
        let types = Types::new(context, &module);
        let runtime_library = RuntimeLibrary::new(&module);
        let intrinsics = Intrinsics::new(&module);
        let constants = Constants::new(&module, &types);
        Ok(Context {
            context,
            module,
            execution_engine,
            builder,
            types,
            runtime_library,
            intrinsics,
            constants,
        })
    }
}

#[cfg(not(feature = "jit"))]
impl<'ctx> Context<'ctx> {
    /// # Errors
    ///
    /// Will return `Err` if module fails to load
    pub fn new(
        context: &'ctx inkwell::context::Context,
        context_type: ModuleType<'ctx>,
    ) -> Result<Self, String> {
        let builder = context.create_builder();
        let module = module::load_module(context, context_type)?;
        let types = Types::new(&context, &module);
        let runtime_library = RuntimeLibrary::new(&module);
        let intrinsics = Intrinsics::new(&module);
        let constants = Constants::new(&module, &types);
        Ok(Context {
            context,
            module,
            builder,
            types,
            runtime_library,
            intrinsics,
            constants,
        })
    }
}

impl<'ctx> Context<'ctx> {
    pub fn emit_bitcode(&self, file_path: &str) {
        let bitcode_path = Path::new(file_path);
        self.module.write_bitcode_to_path(bitcode_path);
    }

    /// # Errors
    ///
    /// Will return `Err` if LLVM Module fails validation
    pub fn emit_ir(&self, file_path: &str) -> Result<(), String> {
        let ir_path = Path::new(file_path);
        if let Err(llvmstr) = self.module.print_to_file(ir_path) {
            return Err(llvmstr.to_string());
        }
        Ok(())
    }

    pub fn get_ir_string(&self) -> String {
        let ir = self.module.print_to_string();
        ir.to_string()
    }

    pub fn get_bitcode_base64_string(&self) -> String {
        let buffer = self.module.write_bitcode_to_memory();
        let bytes = buffer.as_slice();
        base64::encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{Context, ModuleType};
    use std::fs::File;
    use std::io::prelude::*;

    use tempfile::tempdir;

    #[test]
    fn emitted_bitcode_files_are_identical_to_base64_encoded() {
        let dir = tempdir().expect("");
        let tmp_path = dir.into_path();
        let name = String::from("test");
        let file_path = tmp_path.join(format!("{}.bc", name));
        let file_path_string = file_path.display().to_string();

        let ctx = inkwell::context::Context::create();
        let name = String::from("temp");
        let context = Context::new(&ctx, ModuleType::Template(&name)).unwrap();
        context.emit_bitcode(file_path_string.as_str());
        let mut emitted_bitcode_file =
            File::open(file_path_string.as_str()).expect("Could not open emitted bitcode file");
        let mut buffer = vec![];

        emitted_bitcode_file
            .read_to_end(&mut buffer)
            .expect("Could not read emitted bitcode file");
        let emitted_bitcode_bytes = buffer.as_slice();

        let b64_bitcode = context.get_bitcode_base64_string();
        let decoded = base64::decode(b64_bitcode).expect("could not decode base64 encoded module");
        let decoded_bitcode_bytes = decoded.as_slice();

        assert_eq!(emitted_bitcode_bytes, decoded_bitcode_bytes);
    }
}

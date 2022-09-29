// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::codegen::calls::{emit_call_with_return, emit_void_call};
use inkwell::{
    builder::Builder,
    context::Context,
    memory_buffer::MemoryBuffer,
    module::Module,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, InstructionValue},
};
use std::path::Path;

pub mod basicvalues;
pub mod calls;
pub mod qis;
pub mod types;

pub struct CodeGenerator<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// # Errors
    ///
    /// Will return `Err` if module fails to load
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> Self {
        Self {
            context,
            module,
            builder: context.create_builder(),
        }
    }

    pub fn emit_bitcode(&self, path: impl AsRef<Path>) {
        self.module.write_bitcode_to_path(path.as_ref());
    }

    /// # Errors
    ///
    /// Will return `Err` if LLVM Module fails validation
    pub fn emit_ir(&self, path: impl AsRef<Path>) -> Result<(), String> {
        self.module.print_to_file(path).map_err(|e| e.to_string())
    }

    pub fn get_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn get_bitcode(&self) -> MemoryBuffer {
        self.module.write_bitcode_to_memory()
    }

    pub fn emit_void_call(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> InstructionValue<'ctx> {
        emit_void_call(&self.builder, function, args)
    }

    pub fn emit_call_with_return(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> BasicValueEnum<'ctx> {
        emit_call_with_return(&self.builder, function, args, name)
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::CodeGenerator;
    use inkwell::context::Context;
    use std::{fs::File, io::prelude::*};
    use tempfile::tempdir;

    #[test]
    fn emitted_bitcode_files_are_identical_to_base64_encoded() {
        let dir = tempdir().expect("");
        let tmp_path = dir.into_path();
        let name = "test";
        let file_path = tmp_path.join(format!("{}.bc", name));
        let file_path_string = file_path.display().to_string();

        let context = Context::create();
        let module = context.create_module(name);
        let generator = CodeGenerator::new(&context, module);
        generator.emit_bitcode(file_path_string.as_str());

        let mut emitted_bitcode_file =
            File::open(file_path_string.as_str()).expect("Could not open emitted bitcode file");
        let mut emitted_bitcode_bytes = vec![];
        emitted_bitcode_file
            .read_to_end(&mut emitted_bitcode_bytes)
            .expect("Could not read emitted bitcode file");

        let decoded_bitcode_bytes = generator.get_bitcode();

        assert_eq!(
            emitted_bitcode_bytes.as_slice(),
            decoded_bitcode_bytes.as_slice()
        );
    }
}

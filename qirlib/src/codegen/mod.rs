// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{builder::Builder, context::ContextRef, memory_buffer::MemoryBuffer, module::Module};
use std::path::Path;

pub mod qis;
pub mod types;

pub struct CodeGenerator<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// # Errors
    ///
    /// Will return `Err` if module fails to load
    pub fn new(module: Module<'ctx>) -> Self {
        let builder = module.get_context().create_builder();
        Self { module, builder }
    }

    pub(crate) fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub(crate) fn builder(&self) -> &Builder<'ctx> {
        &self.builder
    }

    pub(crate) fn context(&self) -> ContextRef<'ctx> {
        self.module.get_context()
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
        let generator = CodeGenerator::new(module);
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

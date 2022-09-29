// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{builder::Builder, context::ContextRef, module::Module};

pub mod qis;
pub mod types;

pub struct CodeGenerator<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGenerator<'ctx> {
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
}

#[cfg(test)]
mod tests {
    use crate::codegen::CodeGenerator;
    use inkwell::context::Context;
    use std::{fs::File, io::prelude::*, path::Path};
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
        generator
            .module()
            .write_bitcode_to_path(Path::new(&file_path_string));

        let mut emitted_bitcode_file =
            File::open(file_path_string.as_str()).expect("Could not open emitted bitcode file");
        let mut emitted_bitcode_bytes = vec![];
        emitted_bitcode_file
            .read_to_end(&mut emitted_bitcode_bytes)
            .expect("Could not read emitted bitcode file");

        let decoded_bitcode_bytes = generator.module().write_bitcode_to_memory();

        assert_eq!(
            emitted_bitcode_bytes.as_slice(),
            decoded_bitcode_bytes.as_slice()
        );
    }
}

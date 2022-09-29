// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{builder::Builder, module::Module};
use std::ops::Deref;

pub mod qis;
pub mod types;

// TODO: With LLVM, it's possible to get the module that a builder is positioned in using only the
// builder itself. But it's not possible with Inkwell, so we have to bundle the references together.
// See https://github.com/TheDan64/inkwell/issues/347
#[derive(Clone, Copy)]
pub struct BuilderRef<'ctx, 'a> {
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
}

impl<'ctx, 'a> Deref for BuilderRef<'ctx, 'a> {
    type Target = Builder<'ctx>;

    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl<'ctx, 'a> BuilderRef<'ctx, 'a> {
    pub fn new(builder: &'a Builder<'ctx>, module: &'a Module<'ctx>) -> Self {
        Self { builder, module }
    }

    pub(crate) fn module(&self) -> &'a Module<'ctx> {
        self.module
    }
}

#[cfg(test)]
mod tests {
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
        module.write_bitcode_to_path(Path::new(&file_path_string));

        let mut emitted_bitcode_file =
            File::open(file_path_string.as_str()).expect("Could not open emitted bitcode file");
        let mut emitted_bitcode_bytes = vec![];
        emitted_bitcode_file
            .read_to_end(&mut emitted_bitcode_bytes)
            .expect("Could not read emitted bitcode file");

        let decoded_bitcode_bytes = module.write_bitcode_to_memory();

        assert_eq!(
            emitted_bitcode_bytes.as_slice(),
            decoded_bitcode_bytes.as_slice()
        );
    }
}

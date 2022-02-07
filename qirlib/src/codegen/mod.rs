// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::module::Module;
use std::path::Path;

pub mod basicvalues;
pub mod calls;
pub mod ext;
pub mod qis;
pub mod qubits;
pub mod rt;
pub mod types;

pub struct CodeGenerator<'ctx> {
    pub context: &'ctx inkwell::context::Context,
    pub module: inkwell::module::Module<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// # Errors
    ///
    /// Will return `Err` if module fails to load
    pub fn new(
        context: &'ctx inkwell::context::Context,
        module: Module<'ctx>,
    ) -> Result<Self, String> {
        let builder = context.create_builder();
        Ok(CodeGenerator {
            context,
            module,
            builder,
        })
    }
}

impl<'ctx> CodeGenerator<'ctx> {
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
    use crate::{codegen::CodeGenerator, module};
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
        let module = module::load_template(name, &context).unwrap();
        let generator = CodeGenerator::new(&context, module).unwrap();
        generator.emit_bitcode(file_path_string.as_str());
        let mut emitted_bitcode_file =
            File::open(file_path_string.as_str()).expect("Could not open emitted bitcode file");
        let mut buffer = vec![];

        emitted_bitcode_file
            .read_to_end(&mut buffer)
            .expect("Could not read emitted bitcode file");
        let emitted_bitcode_bytes = buffer.as_slice();

        let b64_bitcode = generator.get_bitcode_base64_string();
        let decoded = base64::decode(b64_bitcode).expect("could not decode base64 encoded module");
        let decoded_bitcode_bytes = decoded.as_slice();

        assert_eq!(emitted_bitcode_bytes, decoded_bitcode_bytes);
    }
}

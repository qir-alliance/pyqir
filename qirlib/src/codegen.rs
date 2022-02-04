// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    constants::Constants, intrinsics::Intrinsics, runtime_library::RuntimeLibrary, types::Types,
};
use inkwell::{memory_buffer::MemoryBuffer, module::Module};

pub struct CodeGenerator<'ctx> {
    pub context: &'ctx inkwell::context::Context,
    pub module: inkwell::module::Module<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
    pub types: Types<'ctx>,
    pub runtime_library: RuntimeLibrary<'ctx>,
    pub intrinsics: Intrinsics<'ctx>,
    pub constants: Constants<'ctx>,
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
        let types = Types::new(context, &module);
        let runtime_library = RuntimeLibrary::new(&module);
        let intrinsics = Intrinsics::new(&module);
        let constants = Constants::new(&module, &types);
        Ok(CodeGenerator {
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

impl<'ctx> CodeGenerator<'ctx> {
    pub fn ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn bitcode(&self) -> MemoryBuffer {
        self.module.write_bitcode_to_memory()
    }
}

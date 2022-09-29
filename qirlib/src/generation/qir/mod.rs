// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{attributes::AttributeLoc, module::Module, values::FunctionValue};

pub mod instructions;

pub fn create_entry_point<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let context = module.get_context();
    let fn_type = context.void_type().fn_type(&[], false);
    let fn_value = module.add_function("main", fn_type, None);

    let entry_point_attribute = context.create_string_attribute("EntryPoint", "");
    fn_value.add_attribute(AttributeLoc::Function, entry_point_attribute);
    fn_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::CodeGenerator;
    use inkwell::context::Context;

    #[test]
    fn entry_point_function_has_correct_signature_and_default_attribute() {
        let context = Context::create();
        let module = context.create_module("test");
        let generator = CodeGenerator::new(module);

        let entry_point = create_entry_point(generator.module());
        let entry = generator.context().append_basic_block(entry_point, "entry");
        generator.builder().position_at_end(entry);
        generator.builder().build_return(None);
        let ir_string = generator.module().print_to_string().to_string();
        let expected = "; ModuleID = 'test'\nsource_filename = \"test\"\n\ndefine void @main() #0 {\nentry:\n  ret void\n}\n\nattributes #0 = { \"EntryPoint\" }\n";
        assert_eq!(expected, ir_string);
    }
}

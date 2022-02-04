// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::types::FloatType;
use inkwell::types::IntType;
use inkwell::types::StructType;

use crate::codegen::CodeGenerator;

pub trait Types<'ctx> {
    fn int_type(&self) -> IntType<'ctx>;
    fn double_type(&self) -> FloatType<'ctx>;
    fn bool_type(&self) -> IntType<'ctx>;
    fn qubit_type(&self) -> StructType<'ctx>;
    fn result_type(&self) -> StructType<'ctx>;
    fn array_type(&self) -> StructType<'ctx>;
}

impl<'ctx> Types<'ctx> for CodeGenerator<'ctx> {
    fn int_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    fn double_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    fn qubit_type(&self) -> StructType<'ctx> {
        get_or_define_struct(self, "Qubit")
    }

    fn result_type(&self) -> StructType<'ctx> {
        get_or_define_struct(self, "Result")
    }

    fn array_type(&self) -> StructType<'ctx> {
        get_or_define_struct(self, "Array")
    }
}

fn get_struct<'ctx>(generator: &CodeGenerator<'ctx>, name: &str) -> Option<StructType<'ctx>> {
    let defined_struct = generator.module.get_struct_type(name);
    match defined_struct {
        None => {
            log::debug!("{} was not defined in the module", name);
            None
        }
        Some(value) => Some(value),
    }
}

fn get_or_define_struct<'ctx>(generator: &CodeGenerator<'ctx>, name: &str) -> StructType<'ctx> {
    if let Some(struct_type) = get_struct(generator, name) {
        struct_type
    } else {
        generator.context.opaque_struct_type(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{codegen::CodeGenerator, module};
    use inkwell::context::Context;

    #[test]
    fn qubit_can_be_declared() {
        let context = Context::create();
        let module = module::load_template("test", &context).unwrap();
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Qubit", generator.qubit_type());
    }

    #[test]
    fn result_can_be_declared() {
        let context = Context::create();
        let module = module::load_template("test", &context).unwrap();
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Result", generator.result_type());
    }

    #[test]
    fn array_can_be_declared() {
        let context = Context::create();
        let module = module::load_template("test", &context).unwrap();
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Array", generator.array_type());
    }

    fn verify_opaque_struct(name: &str, struct_type: StructType) {
        assert_eq!(struct_type.get_name().unwrap().to_str(), Ok(name));
        assert!(struct_type.is_opaque());
        assert_eq!(struct_type.get_field_types(), &[]);
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::types::FloatType;
use inkwell::types::IntType;
use inkwell::types::StructType;

use crate::codegen::CodeGenerator;

pub trait Types<'ctx> {
    fn int64_type(&self) -> IntType<'ctx>;
    fn int32_type(&self) -> IntType<'ctx>;
    fn int8_type(&self) -> IntType<'ctx>;
    fn double_type(&self) -> FloatType<'ctx>;
    fn bool_type(&self) -> IntType<'ctx>;
    fn qubit_type(&self) -> StructType<'ctx>;
    fn result_type(&self) -> StructType<'ctx>;
    fn array_type(&self) -> StructType<'ctx>;
}

impl<'ctx> Types<'ctx> for CodeGenerator<'ctx> {
    fn int64_type(&self) -> IntType<'ctx> {
        int64_type(self.context)
    }

    fn int32_type(&self) -> IntType<'ctx> {
        int32_type(self.context)
    }

    fn int8_type(&self) -> IntType<'ctx> {
        int8_type(self.context)
    }

    fn double_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    fn qubit_type(&self) -> StructType<'ctx> {
        get_or_define_struct(self.context, &self.module, "Qubit")
    }

    fn result_type(&self) -> StructType<'ctx> {
        get_or_define_struct(self.context, &self.module, "Result")
    }

    fn array_type(&self) -> StructType<'ctx> {
        get_or_define_struct(self.context, &self.module, "Array")
    }
}

#[must_use]
pub fn int64_type(context: &inkwell::context::Context) -> IntType {
    context.i64_type()
}

#[must_use]
pub fn int32_type(context: &inkwell::context::Context) -> IntType {
    context.i32_type()
}

#[must_use]
pub fn int8_type(context: &inkwell::context::Context) -> IntType {
    context.i8_type()
}

#[must_use]
pub fn double_type(context: &inkwell::context::Context) -> FloatType {
    context.f64_type()
}

#[must_use]
pub fn bool_type(context: &inkwell::context::Context) -> IntType {
    context.bool_type()
}

#[must_use]
pub fn qubit_type<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &inkwell::module::Module<'ctx>,
) -> StructType<'ctx> {
    get_or_define_struct(context, module, "Qubit")
}

#[must_use]
pub fn result_type<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &inkwell::module::Module<'ctx>,
) -> StructType<'ctx> {
    get_or_define_struct(context, module, "Result")
}

#[must_use]
pub fn array_type<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &inkwell::module::Module<'ctx>,
) -> StructType<'ctx> {
    get_or_define_struct(context, module, "Array")
}

#[must_use]
pub fn get_struct<'ctx>(
    module: &inkwell::module::Module<'ctx>,
    name: &str,
) -> Option<StructType<'ctx>> {
    let defined_struct = module.get_struct_type(name);
    match defined_struct {
        None => {
            log::debug!("{} was not defined in the module", name);
            None
        }
        Some(value) => Some(value),
    }
}

pub fn get_or_define_struct<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &inkwell::module::Module<'ctx>,
    name: &str,
) -> StructType<'ctx> {
    if let Some(struct_type) = get_struct(module, name) {
        struct_type
    } else {
        context.opaque_struct_type(name)
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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FloatType;
use inkwell::types::IntType;
use inkwell::types::PointerType;
use inkwell::types::StructType;
use inkwell::AddressSpace;

pub struct Types<'ctx> {
    pub int: IntType<'ctx>,
    pub double: FloatType<'ctx>,
    pub bool: IntType<'ctx>,
    pub pauli: IntType<'ctx>,
    pub range: Option<StructType<'ctx>>,
    pub result: PointerType<'ctx>,
    pub qubit: PointerType<'ctx>,
    pub string: PointerType<'ctx>,
    pub big_int: Option<PointerType<'ctx>>,
    pub tuple: Option<PointerType<'ctx>>,
    pub array: PointerType<'ctx>,
    pub callable: Option<PointerType<'ctx>>,
}

impl<'ctx> Types<'ctx> {
    pub fn new(context: &'ctx Context, module: &Module<'ctx>) -> Self {
        Types {
            int: context.i64_type(),
            double: context.f64_type(),
            bool: context.bool_type(),
            pauli: context.custom_width_int_type(2),

            range: Types::get_struct(module, "Range"),
            result: Types::get_struct_pointer(module, "Result").expect("Result must be defined"),
            qubit: Types::get_struct_pointer(module, "Qubit").expect("Qubit must be defined"),
            string: Types::get_struct_pointer(module, "String").expect("String must be defined"),
            big_int: Types::get_struct_pointer(module, "BigInt"),
            tuple: Types::get_struct_pointer(module, "Tuple"),
            array: Types::get_struct_pointer(module, "Array").expect("Array must be defined"),
            callable: Types::get_struct_pointer(module, "Callable"),
        }
    }

    fn get_struct(module: &Module<'ctx>, name: &str) -> Option<StructType<'ctx>> {
        let defined_struct = module.get_struct_type(name);
        match defined_struct {
            None => {
                log::debug!("{} was not defined in the module", name);
                None
            }
            Some(value) => Some(value),
        }
    }

    fn get_struct_pointer(module: &Module<'ctx>, name: &str) -> Option<PointerType<'ctx>> {
        let defined_struct = module.get_struct_type(name);
        match defined_struct {
            None => {
                log::debug!("{} struct was not defined in the module", name);
                None
            }
            Some(value) => Some(value.ptr_type(AddressSpace::Generic)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        codegen::CodeGenerator,
        module::{self, Source},
    };
    use inkwell::context::Context;

    #[test]
    fn types_can_be_loaded() {
        let context = Context::create();
        let module = module::load(&context, Source::Template).unwrap();
        let generator = CodeGenerator::new(&context, module).unwrap();
        let _ = Types::new(generator.context, &generator.module);
    }
}

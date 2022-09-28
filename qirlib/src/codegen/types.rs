// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    context::Context,
    module::Module,
    types::{FloatType, IntType, PointerType, StructType},
    AddressSpace,
};

#[must_use]
pub(crate) fn int64(context: &Context) -> IntType {
    context.i64_type()
}

#[must_use]
pub(crate) fn int32(context: &Context) -> IntType {
    context.i32_type()
}

#[must_use]
pub(crate) fn int8(context: &Context) -> IntType {
    context.i8_type()
}

#[must_use]
pub(crate) fn double(context: &Context) -> FloatType {
    context.f64_type()
}

#[must_use]
pub(crate) fn qubit<'ctx>(module: &Module<'ctx>) -> StructType<'ctx> {
    get_or_define_struct(module, "Qubit")
}

pub fn qubit_ptr<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    qubit(module).ptr_type(AddressSpace::Generic)
}

#[must_use]
pub(crate) fn result<'ctx>(module: &Module<'ctx>) -> StructType<'ctx> {
    get_or_define_struct(module, "Result")
}

pub fn result_ptr<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    result(module).ptr_type(AddressSpace::Generic)
}

#[must_use]
pub(crate) fn get_struct<'ctx>(module: &Module<'ctx>, name: &str) -> Option<StructType<'ctx>> {
    let defined_struct = module.get_struct_type(name);
    match defined_struct {
        None => {
            log::debug!("{} was not defined in the module", name);
            None
        }
        Some(value) => Some(value),
    }
}

pub(crate) fn get_or_define_struct<'ctx>(module: &Module<'ctx>, name: &str) -> StructType<'ctx> {
    if let Some(struct_type) = get_struct(module, name) {
        struct_type
    } else {
        module.get_context().opaque_struct_type(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::CodeGenerator;
    use inkwell::context::Context;

    #[test]
    fn qubit_can_be_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Qubit", qubit(&generator.module));
    }

    #[test]
    fn result_can_be_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Result", result(&generator.module));
    }

    fn verify_opaque_struct(name: &str, struct_type: StructType) {
        assert_eq!(struct_type.get_name().unwrap().to_str(), Ok(name));
        assert!(struct_type.is_opaque());
        assert_eq!(struct_type.get_field_types(), &[]);
    }
}

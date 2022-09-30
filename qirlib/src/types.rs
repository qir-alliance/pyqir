// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::builder::BuilderRef;
use inkwell::{
    module::Module,
    types::{PointerType, StructType},
    values::PointerValue,
    AddressSpace,
};

pub fn qubit<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    get_or_define_struct(module, "Qubit").ptr_type(AddressSpace::Generic)
}

pub fn result<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    get_or_define_struct(module, "Result").ptr_type(AddressSpace::Generic)
}

#[must_use]
pub fn qubit_id<'ctx>(builder: BuilderRef<'ctx, '_>, id: u64) -> PointerValue<'ctx> {
    let module = builder.module();
    let value = module.get_context().i64_type().const_int(id, false);
    builder.build_int_to_ptr(value, qubit(module), "")
}

#[must_use]
pub fn result_id<'ctx>(builder: BuilderRef<'ctx, '_>, id: u64) -> PointerValue<'ctx> {
    let module = builder.module();
    let value = module.get_context().i64_type().const_int(id, false);
    builder.build_int_to_ptr(value, result(module), "")
}

fn get_or_define_struct<'ctx>(module: &Module<'ctx>, name: &str) -> StructType<'ctx> {
    get_struct(module, name).unwrap_or_else(|| module.get_context().opaque_struct_type(name))
}

#[must_use]
fn get_struct<'ctx>(module: &Module<'ctx>, name: &str) -> Option<StructType<'ctx>> {
    let struct_type = module.get_struct_type(name);
    if struct_type.is_none() {
        log::debug!("{} was not defined in the module", name);
    }
    struct_type
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn qubit_can_be_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        verify_opaque_pointer("Qubit", qubit(&module));
    }

    #[test]
    fn result_can_be_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        verify_opaque_pointer("Result", result(&module));
    }

    fn verify_opaque_pointer(name: &str, ty: PointerType) {
        let pointee = ty.get_element_type().into_struct_type();
        assert_eq!(pointee.get_name().unwrap().to_str(), Ok(name));
        assert!(pointee.is_opaque());
        assert_eq!(pointee.get_field_types(), &[]);
    }
}

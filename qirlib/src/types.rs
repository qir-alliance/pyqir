// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    module::Module,
    types::{PointerType, StructType},
    AddressSpace,
};

pub fn qubit<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    get_or_create_struct(module, "Qubit").ptr_type(AddressSpace::Generic)
}

pub fn result<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    get_or_create_struct(module, "Result").ptr_type(AddressSpace::Generic)
}

fn get_or_create_struct<'ctx>(module: &Module<'ctx>, name: &str) -> StructType<'ctx> {
    module.get_struct_type(name).unwrap_or_else(|| {
        log::debug!("{} was not defined in the module", name);
        module.get_context().opaque_struct_type(name)
    })
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

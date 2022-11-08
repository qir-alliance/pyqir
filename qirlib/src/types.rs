// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    module::Module,
    types::{AnyTypeEnum, PointerType, StructType},
    AddressSpace,
};

pub fn qubit<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    get_or_create_struct(module, "Qubit").ptr_type(AddressSpace::Generic)
}

#[must_use]
pub fn is_qubit(ty: AnyTypeEnum) -> bool {
    is_opaque_pointer_to(ty, "Qubit")
}

pub fn result<'ctx>(module: &Module<'ctx>) -> PointerType<'ctx> {
    get_or_create_struct(module, "Result").ptr_type(AddressSpace::Generic)
}

#[must_use]
pub fn is_result(ty: AnyTypeEnum) -> bool {
    is_opaque_pointer_to(ty, "Result")
}

fn get_or_create_struct<'ctx>(module: &Module<'ctx>, name: &str) -> StructType<'ctx> {
    module.get_struct_type(name).unwrap_or_else(|| {
        log::debug!("{} was not defined in the module", name);
        module.get_context().opaque_struct_type(name)
    })
}

fn is_opaque_pointer_to(ty: AnyTypeEnum, name: &str) -> bool {
    match ty {
        AnyTypeEnum::PointerType(p) => match p.get_element_type() {
            AnyTypeEnum::StructType(s) => {
                let struct_name = s.get_name().and_then(|n| n.to_str().ok());
                struct_name == Some(name)
            }
            _ => false,
        },
        _ => false,
    }
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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    context::ContextRef,
    types::{AnyTypeEnum, PointerType, StructType},
    AddressSpace, LLVMReference,
};
use llvm_sys::core::{LLVMGetTypeByName2, LLVMStructCreateNamed};
use std::ffi::CStr;

#[must_use]
pub fn qubit<'ctx>(context: &ContextRef<'ctx>) -> PointerType<'ctx> {
    get_or_create_struct(context, qubit_name()).ptr_type(AddressSpace::Generic)
}

#[must_use]
pub fn is_qubit(ty: AnyTypeEnum) -> bool {
    is_opaque_pointer_to(ty, qubit_name())
}

#[must_use]
pub fn result<'ctx>(context: &ContextRef<'ctx>) -> PointerType<'ctx> {
    get_or_create_struct(context, result_name()).ptr_type(AddressSpace::Generic)
}

#[must_use]
pub fn is_result(ty: AnyTypeEnum) -> bool {
    is_opaque_pointer_to(ty, result_name())
}

fn qubit_name() -> &'static CStr {
    unsafe { CStr::from_bytes_with_nul_unchecked(b"Qubit\0") }
}

fn result_name() -> &'static CStr {
    unsafe { CStr::from_bytes_with_nul_unchecked(b"Result\0") }
}

fn get_or_create_struct<'ctx>(context: &ContextRef<'ctx>, name: &CStr) -> StructType<'ctx> {
    let context = unsafe { context.get_ref() };
    let name = name.as_ptr().cast();
    let ty = unsafe { LLVMGetTypeByName2(context, name) };
    if ty.is_null() {
        unsafe { StructType::new(LLVMStructCreateNamed(context, name)) }
    } else {
        unsafe { StructType::new(ty) }
    }
}

fn is_opaque_pointer_to(ty: AnyTypeEnum, name: &CStr) -> bool {
    match ty {
        AnyTypeEnum::PointerType(p) => match p.get_element_type() {
            AnyTypeEnum::StructType(s) => s.get_name() == Some(name),
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
        let context = context.void_type().get_context();
        verify_opaque_pointer("Qubit", qubit(&context));
    }

    #[test]
    fn result_can_be_declared() {
        let context = Context::create();
        let context = context.void_type().get_context();
        verify_opaque_pointer("Result", result(&context));
    }

    fn verify_opaque_pointer(name: &str, ty: PointerType) {
        let pointee = ty.get_element_type().into_struct_type();
        assert_eq!(pointee.get_name().unwrap().to_str(), Ok(name));
        assert!(pointee.is_opaque());
        assert_eq!(pointee.get_field_types(), &[]);
    }
}

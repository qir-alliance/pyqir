// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    context::ContextRef,
    types::{AnyTypeEnum, PointerType},
    LLVMReference,
};
use llvm_sys::{
    core::{LLVMGetTypeByName2, LLVMPointerType, LLVMStructCreateNamed},
    prelude::*,
};
use std::ffi::CStr;

const QUBIT: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Qubit\0") };
const RESULT: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Result\0") };

#[must_use]
pub fn qubit<'ctx>(context: &ContextRef<'ctx>) -> PointerType<'ctx> {
    unsafe { PointerType::new(qubit_unchecked(context.get_ref())) }
}

pub(crate) unsafe fn qubit_unchecked(context: LLVMContextRef) -> LLVMTypeRef {
    LLVMPointerType(get_or_create_struct(context, QUBIT), 0)
}

#[must_use]
pub fn is_qubit(ty: AnyTypeEnum) -> bool {
    is_opaque_pointer_to(ty, QUBIT)
}

#[must_use]
pub fn result<'ctx>(context: &ContextRef<'ctx>) -> PointerType<'ctx> {
    unsafe { PointerType::new(result_unchecked(context.get_ref())) }
}

pub(crate) unsafe fn result_unchecked(context: LLVMContextRef) -> LLVMTypeRef {
    LLVMPointerType(get_or_create_struct(context, RESULT), 0)
}

#[must_use]
pub fn is_result(ty: AnyTypeEnum) -> bool {
    is_opaque_pointer_to(ty, RESULT)
}

unsafe fn get_or_create_struct(context: LLVMContextRef, name: &CStr) -> LLVMTypeRef {
    let name = name.as_ptr().cast();
    let ty = LLVMGetTypeByName2(context, name);
    if ty.is_null() {
        LLVMStructCreateNamed(context, name)
    } else {
        ty
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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use const_str::cstr;
use llvm_sys::{
    core::{
        LLVMGetElementType, LLVMGetStructName, LLVMGetTypeByName2, LLVMGetTypeKind,
        LLVMIsOpaqueStruct, LLVMPointerType, LLVMStructCreateNamed,
    },
    prelude::*,
    LLVMTypeKind,
};
use std::ffi::CStr;

const QUBIT: &CStr = cstr!("Qubit");
const RESULT: &CStr = cstr!("Result");

pub unsafe fn qubit(context: LLVMContextRef) -> LLVMTypeRef {
    LLVMPointerType(get_or_create_struct(context, QUBIT), 0)
}

pub unsafe fn is_qubit(ty: LLVMTypeRef) -> bool {
    is_opaque_pointer_to(ty, QUBIT)
}

pub unsafe fn result(context: LLVMContextRef) -> LLVMTypeRef {
    LLVMPointerType(get_or_create_struct(context, RESULT), 0)
}

pub unsafe fn is_result(ty: LLVMTypeRef) -> bool {
    is_opaque_pointer_to(ty, RESULT)
}

unsafe fn get_or_create_struct(context: LLVMContextRef, name: &CStr) -> LLVMTypeRef {
    let ty = LLVMGetTypeByName2(context, name.as_ptr());
    if ty.is_null() {
        LLVMStructCreateNamed(context, name.as_ptr())
    } else {
        ty
    }
}

unsafe fn is_opaque_pointer_to(ty: LLVMTypeRef, name: &CStr) -> bool {
    if LLVMGetTypeKind(ty) == LLVMTypeKind::LLVMPointerTypeKind {
        let pointee = LLVMGetElementType(ty);
        LLVMGetTypeKind(pointee) == LLVMTypeKind::LLVMStructTypeKind
            && LLVMIsOpaqueStruct(ty) != 0
            && CStr::from_ptr(LLVMGetStructName(pointee)) == name
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Context;
    use llvm_sys::core::LLVMContextCreate;
    use std::ptr::NonNull;

    #[test]
    fn qubit_type() {
        unsafe {
            let context = Context::new(NonNull::new(LLVMContextCreate()).unwrap());
            let qubit = qubit(context.as_ptr());
            assert!(is_qubit(qubit));
            assert!(!is_result(qubit));
        }
    }

    #[test]
    fn result_type() {
        unsafe {
            let context = Context::new(NonNull::new(LLVMContextCreate()).unwrap());
            let result = result(context.as_ptr());
            assert!(is_result(result));
            assert!(!is_qubit(result));
        }
    }
}

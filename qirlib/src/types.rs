// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::{
    core::{
        LLVMGetElementType, LLVMGetStructName, LLVMGetTypeByName2, LLVMGetTypeKind,
        LLVMIsOpaqueStruct, LLVMPointerType, LLVMStructCreateNamed,
    },
    prelude::*,
    LLVMTypeKind,
};
use std::ffi::CStr;

const QUBIT: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Qubit\0") };
const RESULT: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Result\0") };

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
    let name = name.as_ptr().cast();
    let ty = LLVMGetTypeByName2(context, name);
    if ty.is_null() {
        LLVMStructCreateNamed(context, name)
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

    #[test]
    fn qubit_type() {
        unsafe {
            let context = Context::new(LLVMContextCreate());
            let qubit = qubit(*context);
            assert!(is_qubit(qubit));
            assert!(!is_result(qubit));
        }
    }

    #[test]
    fn result_type() {
        unsafe {
            let context = Context::new(LLVMContextCreate());
            let result = result(*context);
            assert!(is_result(result));
            assert!(!is_qubit(result));
        }
    }
}

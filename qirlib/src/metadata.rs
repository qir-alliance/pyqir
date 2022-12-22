// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::prelude::LLVMValueRef;

use crate::llvm_wrapper::{LLVMRustExtractMDConstant, LLVMRustIsAMDConstant};

pub unsafe fn extract_constant(value: LLVMValueRef) -> Option<LLVMValueRef> {
    let constant_value = unsafe { LLVMRustExtractMDConstant(value) };
    if constant_value.is_null() {
        None
    } else {
        Some(constant_value)
    }
}

pub unsafe fn is_constant(value: LLVMValueRef) -> bool {
    let result = unsafe { LLVMRustIsAMDConstant(value) };
    !result.is_null()
}

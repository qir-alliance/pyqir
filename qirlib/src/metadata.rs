// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::prelude::LLVMValueRef;

use crate::llvm_wrapper::LLVMRustExtractMDConstant;

pub unsafe fn extract_constant(value: LLVMValueRef) -> Option<LLVMValueRef> {
    let constant_value = LLVMRustExtractMDConstant(value);
    if constant_value.is_null() {
        None
    } else {
        Some(constant_value)
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::prelude::LLVMContextRef;

use crate::llvm_wrapper::LLVMRustContextCreate;

#[must_use]
pub unsafe fn create(use_opaque_pointers: bool) -> LLVMContextRef {
    unsafe { LLVMRustContextCreate(i32::from(use_opaque_pointers)) }
}

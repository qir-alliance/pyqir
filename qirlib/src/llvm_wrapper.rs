// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::prelude::{LLVMContextRef, LLVMMetadataRef, LLVMModuleRef, LLVMValueRef};

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LLVMRustModFlagBehavior {
    Error = 1,
    Warning = 2,
    Require = 3,
    Override = 4,
    Append = 5,
    AppendUnique = 6,
    Max = 7,
    #[cfg(feature = "llvm15-0")]
    Min = 8,
}

pub type LLVMBool = std::ffi::c_int;

extern "C" {
    /// Add a module-level flag to the module-level flags metadata if it doesn't already exist.
    pub fn LLVMRustAddModuleFlag(
        M: LLVMModuleRef,
        Behavior: LLVMRustModFlagBehavior,
        Key: *const std::ffi::c_char,
        KeyLen: std::ffi::c_uint,
        Val: LLVMMetadataRef,
    );
    pub fn LLVMRustContextCreate(OpaquePointers: LLVMBool) -> LLVMContextRef;
    pub fn LLVMRustIsAMDConstant(Val: LLVMValueRef) -> LLVMValueRef;
    pub fn LLVMRustExtractMDConstant(Val: LLVMValueRef) -> LLVMValueRef;
}

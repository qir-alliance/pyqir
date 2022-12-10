// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::prelude::{LLVMMetadataRef, LLVMModuleRef};

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
    #[cfg(feature = "llvm14-0")]
    Min = 8,
}

extern "C" {
    /// Add a module-level flag to the module-level flags metadata if it doesn't already exist.
    pub fn LLVMRustAddModuleFlag(
        M: LLVMModuleRef,
        Behavior: LLVMRustModFlagBehavior,
        Key: *const ::libc::c_char,
        KeyLen: ::libc::size_t,
        Val: LLVMMetadataRef,
    );
}

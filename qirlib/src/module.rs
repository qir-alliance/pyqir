// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::prelude::{LLVMMetadataRef, LLVMModuleRef};

use crate::llvm_wrapper::{LLVMRustAddModuleFlag, LLVMRustModFlagBehavior};

pub enum FlagBehavior {
    Error,
    Warning,
    Require,
    Override,
    Append,
    AppendUnique,
    Max,
}

impl From<LLVMRustModFlagBehavior> for FlagBehavior {
    fn from(flag: LLVMRustModFlagBehavior) -> Self {
        match flag {
            LLVMRustModFlagBehavior::Error => FlagBehavior::Error,
            LLVMRustModFlagBehavior::Warning => FlagBehavior::Warning,
            LLVMRustModFlagBehavior::Require => FlagBehavior::Require,
            LLVMRustModFlagBehavior::Override => FlagBehavior::Override,
            LLVMRustModFlagBehavior::Append => FlagBehavior::Append,
            LLVMRustModFlagBehavior::AppendUnique => FlagBehavior::AppendUnique,
            LLVMRustModFlagBehavior::Max => FlagBehavior::Max,
        }
    }
}

impl From<FlagBehavior> for LLVMRustModFlagBehavior {
    fn from(flag: FlagBehavior) -> Self {
        match flag {
            FlagBehavior::Error => LLVMRustModFlagBehavior::Error,
            FlagBehavior::Warning => LLVMRustModFlagBehavior::Warning,
            FlagBehavior::Require => LLVMRustModFlagBehavior::Require,
            FlagBehavior::Override => LLVMRustModFlagBehavior::Override,
            FlagBehavior::Append => LLVMRustModFlagBehavior::Append,
            FlagBehavior::AppendUnique => LLVMRustModFlagBehavior::AppendUnique,
            FlagBehavior::Max => LLVMRustModFlagBehavior::Max,
        }
    }
}

pub unsafe fn add_flag(
    module: LLVMModuleRef,
    behavior: FlagBehavior,
    id: &str,
    md: LLVMMetadataRef,
) {
    LLVMRustAddModuleFlag(
        module,
        behavior
            .try_into()
            .expect("Could not convert behavior for the current version of LLVM"),
        id.as_ptr() as *mut std::ffi::c_char,
        id.len().try_into().unwrap(),
        md,
    );
}

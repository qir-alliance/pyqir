// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::{
    core::{
        LLVMConstInt, LLVMConstIntGetZExtValue, LLVMGetModuleContext, LLVMGetModuleFlag,
        LLVMInt1TypeInContext, LLVMInt32TypeInContext, LLVMMetadataAsValue, LLVMValueAsMetadata,
    },
    prelude::{LLVMMetadataRef, LLVMModuleRef},
};

use crate::{
    llvm_wrapper::{LLVMRustAddModuleFlag, LLVMRustModFlagBehavior},
    metadata::extract_constant,
};

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

pub unsafe fn qir_major_version(module: LLVMModuleRef) -> Option<i32> {
    i32::try_from(get_u64_flag(module, "qir_major_version")?).ok()
}

pub unsafe fn set_qir_major_version(module: LLVMModuleRef, value: i32) {
    let context = LLVMGetModuleContext(module);
    let i32ty = LLVMInt32TypeInContext(context);
    let const_value = LLVMConstInt(i32ty, value.try_into().unwrap(), 0);
    let md = LLVMValueAsMetadata(const_value);
    add_flag(module, FlagBehavior::Error, "qir_major_version", md);
}

pub unsafe fn qir_minor_version(module: LLVMModuleRef) -> Option<i32> {
    i32::try_from(get_u64_flag(module, "qir_minor_version")?).ok()
}

pub unsafe fn set_qir_minor_version(module: LLVMModuleRef, value: i32) {
    let context = LLVMGetModuleContext(module);
    let i32ty = LLVMInt32TypeInContext(context);
    let const_value = LLVMConstInt(i32ty, value.try_into().unwrap(), 0);
    let md = LLVMValueAsMetadata(const_value);
    add_flag(module, FlagBehavior::Max, "qir_minor_version", md);
}

pub unsafe fn dynamic_qubit_management(module: LLVMModuleRef) -> Option<bool> {
    get_i1_flag(module, "dynamic_qubit_management")
}

pub unsafe fn set_dynamic_qubit_management(module: LLVMModuleRef, value: bool) {
    let context = LLVMGetModuleContext(module);
    let i1ty = LLVMInt1TypeInContext(context);
    let const_value = LLVMConstInt(i1ty, u64::from(value), 0);
    let md = LLVMValueAsMetadata(const_value);
    add_flag(module, FlagBehavior::Error, "dynamic_qubit_management", md);
}

pub unsafe fn dynamic_result_management(module: LLVMModuleRef) -> Option<bool> {
    get_i1_flag(module, "dynamic_result_management")
}

pub unsafe fn set_dynamic_result_management(module: LLVMModuleRef, value: bool) {
    let context = LLVMGetModuleContext(module);
    let i1ty = LLVMInt1TypeInContext(context);
    let const_value = LLVMConstInt(i1ty, u64::from(value), 0);
    let md = LLVMValueAsMetadata(const_value);
    add_flag(module, FlagBehavior::Error, "dynamic_result_management", md);
}

unsafe fn get_u64_flag(module: LLVMModuleRef, id: &str) -> Option<u64> {
    if let Some(flag) = get_flag(module, id) {
        if let Some(constant) =
            extract_constant(LLVMMetadataAsValue(LLVMGetModuleContext(module), flag))
        {
            let value = LLVMConstIntGetZExtValue(constant);
            return Some(value);
        }
    }
    None
}

unsafe fn get_i1_flag(module: LLVMModuleRef, id: &str) -> Option<bool> {
    if let Some(value) = get_u64_flag(module, id) {
        return Some(value != 0);
    }
    None
}

pub unsafe fn get_flag(module: LLVMModuleRef, id: &str) -> Option<LLVMMetadataRef> {
    let flag = LLVMGetModuleFlag(module, id.as_ptr().cast(), id.len());

    if flag.is_null() {
        return None;
    }
    Some(flag)
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

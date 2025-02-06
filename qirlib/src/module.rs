// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    mem::MaybeUninit,
};

use std::sync::Once;

static LLVM_INIT: Once = Once::new();

pub(crate) fn ensure_init() {
    LLVM_INIT.call_once(|| unsafe {
        LLVMInitializeWebAssemblyTargetInfo();
        LLVMInitializeWebAssemblyTarget();
        LLVMInitializeWebAssemblyTargetMC();
        LLVMInitializeWebAssemblyAsmPrinter();
        LLVMInitializeWebAssemblyAsmParser();
        LLVMInitializeWebAssemblyDisassembler();
    });
}

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMConstInt, LLVMConstIntGetZExtValue, LLVMDisposeMessage, LLVMGetModuleContext,
        LLVMGetModuleFlag, LLVMInt1TypeInContext, LLVMInt32TypeInContext, LLVMMetadataAsValue,
        LLVMValueAsMetadata,
    },
    prelude::{LLVMMetadataRef, LLVMModuleRef},
    target::{
        LLVMInitializeWebAssemblyAsmParser, LLVMInitializeWebAssemblyAsmPrinter,
        LLVMInitializeWebAssemblyDisassembler, LLVMInitializeWebAssemblyTarget,
        LLVMInitializeWebAssemblyTargetInfo, LLVMInitializeWebAssemblyTargetMC,
    },
    target_machine::{
        LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine,
        LLVMGetTargetFromTriple, LLVMRelocMode, LLVMTargetMachineEmitToFile,
    },
};
use std::ptr::null;

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
        behavior.into(),
        id.as_ptr() as *mut std::ffi::c_char,
        id.len().try_into().unwrap(),
        md,
    );
}

fn to_c_str(mut s: &str) -> Cow<'_, CStr> {
    if s.is_empty() {
        s = "\0";
    }
    if !s.chars().rev().any(|ch| ch == '\0') {
        return Cow::from(CString::new(s).expect("CString::new failed"));
    }

    unsafe { Cow::from(CStr::from_ptr(s.as_ptr() as *const _)) }
}

pub unsafe fn verify(module: LLVMModuleRef) -> Option<String> {
    let action = LLVMVerifierFailureAction::LLVMReturnStatusAction;
    let mut error = std::ptr::null_mut();
    if LLVMVerifyModule(module, action, &mut error) == 0 {
        None
    } else {
        let error_cstr = CStr::from_ptr(error);
        let string = error_cstr.to_str().unwrap().to_string();
        LLVMDisposeMessage(error);
        Some(string)
    }
}

pub unsafe fn write_wasm_to_file(module: LLVMModuleRef, file_path: &str) -> Result<(), String> {
    if let Some(error) = verify(module) {
        return Err(error);
    }

    ensure_init();

    let level = LLVMCodeGenOptLevel::LLVMCodeGenLevelNone;
    let reloc_mode = LLVMRelocMode::LLVMRelocStatic;
    let code_model = LLVMCodeModel::LLVMCodeModelDefault;

    let triple = "wasm32-unknown-unknown";
    let mut target = std::ptr::null_mut();
    let mut err_string = MaybeUninit::uninit();
    let success =
        LLVMGetTargetFromTriple(triple.as_ptr().cast(), &mut target, err_string.as_mut_ptr());

    if success != 0 {
        let error_ptr = err_string.assume_init();
        let message = CStr::from_ptr(error_ptr)
            .to_str()
            .expect("Failed to get error string");
        let message_string = message.to_string();
        LLVMDisposeMessage(error_ptr);
        return Err(message_string);
    }

    let target_machine = unsafe {
        LLVMCreateTargetMachine(
            target,
            triple.as_ptr().cast(),
            null(),
            null(),
            level.into(),
            reloc_mode.into(),
            code_model.into(),
        )
    };

    if target_machine.is_null() {
        return Err("Failed to create target machine".to_string());
    }
    let mut err_string = MaybeUninit::uninit();
    //let mut memory_buffer = std::ptr::null_mut();
    //let res = LLVMTargetMachineEmitToMemoryBuffer(target_machine, module, LLVMCodeGenFileType::LLVMObjectFile, err_string.as_mut_ptr(), &mut memory_buffer);
    let file = to_c_str(file_path);
    let success: i32 = LLVMTargetMachineEmitToFile(
        target_machine,
        module,
        file.as_ptr().cast_mut().cast(),
        LLVMCodeGenFileType::LLVMObjectFile,
        err_string.as_mut_ptr(),
    );

    if success != 0 {
        let error_ptr = err_string.assume_init();
        let message = CStr::from_ptr(error_ptr)
            .to_str()
            .expect("Failed to get error string");
        let message_string = message.to_string();
        LLVMDisposeMessage(error_ptr);
        return Err(message_string);
    }

    // this is segfaulting for some reason, need to debug
    // LLVMTargetMachineEmitToFile(
    //         target_machine,
    //         module,
    //         file_type_ptr,
    //         err_string.as_mut_ptr(),
    //         &mut memory_buffer,
    //     )
    // };

    Ok(())
}

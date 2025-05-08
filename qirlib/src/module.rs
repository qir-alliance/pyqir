// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    fs::File,
    io::Read,
    mem::MaybeUninit,
    slice, str,
};

use std::sync::Once;

use tempfile::NamedTempFile;

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
        LLVMConstInt, LLVMConstIntGetZExtValue, LLVMDisposeMessage, LLVMGetFirstFunction,
        LLVMGetModuleContext, LLVMGetModuleFlag, LLVMGetNextFunction, LLVMGetValueName2,
        LLVMInt1TypeInContext, LLVMInt32TypeInContext, LLVMMetadataAsValue, LLVMValueAsMetadata,
    },
    prelude::{LLVMMetadataRef, LLVMModuleRef, LLVMValueRef},
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
    llvm_wrapper::{LLVMRustAddModuleFlag, LLVMRustModFlagBehavior, SafeReturn},
    metadata::extract_constant,
    values::is_entry_point,
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

    unsafe { Cow::from(CStr::from_ptr(s.as_ptr().cast())) }
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

pub unsafe fn raw_wasm(module: LLVMModuleRef) -> Result<Vec<u8>, String> {
    use tempfile::NamedTempFile;
    let mut temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().into_owned();
    write_raw_wasm_to_file(module, &temp_path)?;
    let mut buffer = Vec::new();
    temp_file.read_to_end(&mut buffer).unwrap();
    Ok(buffer)
}

pub unsafe fn write_raw_wasm_to_file(module: LLVMModuleRef, file_path: &str) -> Result<(), String> {
    if let Some(error) = verify(module) {
        return Err(error);
    }

    ensure_init();

    let level = LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive;
    let reloc_mode = LLVMRelocMode::LLVMRelocDefault;
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
            level,
            reloc_mode,
            code_model,
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

pub unsafe fn compile_wasm(module: LLVMModuleRef) -> Result<Vec<u8>, String> {
    // Write the wasm object file to disk, this will be input to the linking stage

    let raw_wasm_file = NamedTempFile::new().unwrap();
    let raw_wasm_path = raw_wasm_file.path().to_string_lossy().into_owned();
    write_raw_wasm_to_file(module, &raw_wasm_path)?;

    let entry_point = choose_entry_point(get_functions(module).into_iter(), None)?;
    // if entry_point == "main" {
    //     // we can't have an entry point named "main" because it will conflict with llvm internals
    //     return Err("Entry point cannot be named 'main'".to_string());
    // }
    if entry_point.is_empty() {
        return Err("No entry point found".to_string());
    }

    let linked_wasm_file = NamedTempFile::new().unwrap();
    //let linked_wasm_path = linked_wasm_file.path().to_string_lossy().into_owned();
    let (_, linked_wasm_path) = linked_wasm_file.keep().unwrap();

    // build up and convert the args to a format that can be passed to the linker
    let entry_arg = format!("--entry={entry_point}");
    //let entry_arg = "--no-entry".to_string();
    let args = [
        "pyqir-wasm-ld",
        &entry_arg,
        "--allow-undefined",
        "--verbose",
        "-o",
        linked_wasm_path.to_str().unwrap(),
        raw_wasm_path.as_str(),
    ];
    let args = args
        .iter()
        .map(|arg| CString::new(*arg).unwrap())
        .collect::<Vec<CString>>();
    let args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const std::ffi::c_char>>();

    // call the linker inproc
    let (res, out, err) = link_wasm_wrapper(
        i32::try_from(args.len()).expect("could not convert number of args"),
        args.as_ptr(),
    );
    if !out.is_empty() {
        use std::io::Write;
        std::io::stdout().write_all(out.as_bytes()).unwrap();
    }

    if res.ret != 0 {
        if !res.can_run_again {
            return Err(
                "Failed to link wasm and memory was corrupted. Restart procssess/session."
                    .to_string(),
            );
        }

        let msg = format!("Failed to link wasm {res:?}: \nstdout:\n{out}\n\nstderr:\n{err}");
        return Err(msg);
    } else if !err.is_empty() {
        use std::io::Write;
        std::io::stderr().write_all(err.as_bytes()).unwrap();
    }

    let mut buffer = Vec::new();
    let mut tt = File::open(linked_wasm_path).map_err(|e| e.to_string())?;
    let read = tt.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    if read == 0 {
        return Err("No data written to file".to_string());
    }

    Ok(buffer)
}

#[must_use]
#[allow(clippy::similar_names)]
pub unsafe fn link_wasm_wrapper(
    argc: i32,
    argv: *const *const std::ffi::c_char,
) -> (SafeReturn, String, String) {
    let mut stdout: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut stderr: *mut std::ffi::c_char = std::ptr::null_mut();
    let res = crate::llvm_wrapper::safeLldMainWrapper(argc, argv, &mut stdout, &mut stderr);

    let out_string = extract_string_allocated_by_llvm(stdout);
    let err_string = extract_string_allocated_by_llvm(stderr);
    (res, out_string, err_string)
}

unsafe fn extract_string_allocated_by_llvm(ptr: *mut i8) -> String {
    let err_string = if ptr.is_null() {
        String::new()
    } else {
        let err_cstr = CStr::from_ptr(ptr);
        let err_string = err_cstr.to_str().unwrap().to_string();
        LLVMDisposeMessage(ptr);
        err_string
    };
    err_string
}

unsafe fn choose_entry_point(
    functions: impl Iterator<Item = LLVMValueRef>,
    name: Option<&str>,
) -> Result<String, String> {
    fn get_name(f: LLVMValueRef) -> Result<String, String> {
        let mut len = 0;
        unsafe {
            let name = LLVMGetValueName2(f, &mut len).cast();
            let x = str::from_utf8(slice::from_raw_parts(name, len)).map_err(|e| e.to_string())?;
            Ok(x.to_owned())
        }
    }
    let mut entry_points = functions.filter(|f| {
        is_entry_point(*f) && name.iter().all(|n| get_name(*f) == Ok((*n).to_string()))
    });

    let entry_point = entry_points
        .next()
        .ok_or_else(|| "No matching entry point found.".to_owned())?;

    if entry_points.next().is_some() {
        Err("Multiple matching entry points found.".to_owned())
    } else {
        get_name(entry_point)
    }
}

fn get_functions(module: LLVMModuleRef) -> Vec<LLVMValueRef> {
    let mut functions = Vec::new();
    unsafe {
        let mut function = LLVMGetFirstFunction(module);
        while !function.is_null() {
            functions.push(function);
            function = LLVMGetNextFunction(function);
        }
    }
    functions
}

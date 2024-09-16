// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::{
    core::{LLVMContextSetDiagnosticHandler, LLVMGetDiagInfoDescription, LLVMGetDiagInfoSeverity},
    prelude::{LLVMContextRef, LLVMDiagnosticInfoRef},
    LLVMDiagnosticSeverity,
};

pub unsafe fn set_diagnostic_handler(context: LLVMContextRef, output_ptr: *mut core::ffi::c_void) {
    unsafe { LLVMContextSetDiagnosticHandler(context, Some(diagnostic_handler), output_ptr) };
}

pub(crate) extern "C" fn diagnostic_handler(
    diagnostic_info: LLVMDiagnosticInfoRef,
    output: *mut ::core::ffi::c_void,
) {
    unsafe {
        let severity = LLVMGetDiagInfoSeverity(diagnostic_info);
        if severity == LLVMDiagnosticSeverity::LLVMDSError {
            let c_char_output = output
                .cast::<*mut ::core::ffi::c_void>()
                .cast::<*mut ::core::ffi::c_char>();
            *c_char_output = LLVMGetDiagInfoDescription(diagnostic_info);
        }
    }
}

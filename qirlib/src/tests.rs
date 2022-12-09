// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    values,
    wrappers::{Builder, Message},
};
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAppendBasicBlockInContext, LLVMBuildRetVoid, LLVMContextCreate,
        LLVMCreateBuilderInContext, LLVMModuleCreateWithNameInContext, LLVMPositionBuilderAtEnd,
        LLVMPrintModuleToString,
    },
    prelude::*,
};
use normalize_line_endings::normalized;
use std::{
    env,
    ffi::{CStr, CString},
    fs,
    path::PathBuf,
    ptr::null_mut,
};

const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";

/// Compares generated IR against reference files in the "resources/tests" folder. If changes
/// to code generation break the tests:
///
/// 1. Run the tests with the `PYQIR_TEST_SAVE_REFERENCES` environment variable set to
///    regenerate the reference files.
/// 2. Review the changes and make sure they look reasonable.
/// 3. Unset the environment variable and run the tests again to confirm that they pass.
pub(crate) fn assert_reference_ir(
    id: &str,
    required_num_qubits: u64,
    required_num_results: u64,
    build: impl Fn(LLVMBuilderRef),
) {
    let (prefix, name) = split_id(id);
    let c_name = &CString::new(name).unwrap();
    let actual_ir = build_ir(c_name, required_num_qubits, required_num_results, build).unwrap();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources");
    path.push("tests");
    prefix.iter().for_each(|p| path.push(p));
    path.push(name);
    path.set_extension("ll");

    if env::var(PYQIR_TEST_SAVE_REFERENCES).is_ok() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, actual_ir.to_bytes()).unwrap();
        panic!(
            "Saved reference IR. Run again without the {} environment variable.",
            PYQIR_TEST_SAVE_REFERENCES
        )
    } else {
        let contents = fs::read_to_string(&path).unwrap();
        let expected_ir: String = normalized(contents.chars()).collect();
        assert_eq!(expected_ir, actual_ir.to_str().unwrap());
    }
}

fn build_ir(
    name: &CStr,
    required_num_qubits: u64,
    required_num_results: u64,
    build: impl Fn(LLVMBuilderRef),
) -> Result<Message, Message> {
    unsafe {
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(name.as_ptr(), context);
        let entry_point = values::entry_point(
            module,
            CStr::from_bytes_with_nul_unchecked(b"main\0"),
            required_num_qubits,
            required_num_results,
        );

        let builder = Builder::new(LLVMCreateBuilderInContext(context));
        LLVMPositionBuilderAtEnd(
            *builder,
            LLVMAppendBasicBlockInContext(context, entry_point, b"\0".as_ptr().cast()),
        );
        build(*builder);
        LLVMBuildRetVoid(*builder);

        let action = LLVMVerifierFailureAction::LLVMReturnStatusAction;
        let mut error = null_mut();
        if LLVMVerifyModule(module, action, &mut error) == 0 {
            Ok(Message::new(LLVMPrintModuleToString(module)))
        } else {
            Err(Message::new(error))
        }
    }
}

fn split_id(id: &str) -> (Vec<&str>, &str) {
    let mut parts: Vec<_> = id.split('/').collect();
    let name = parts.pop().expect("Empty string.");
    (parts, name)
}

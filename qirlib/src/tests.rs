// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::values;
use libc::c_char;
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAppendBasicBlockInContext, LLVMBuildRetVoid, LLVMContextCreate, LLVMContextDispose,
        LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMDisposeMessage,
        LLVMModuleCreateWithNameInContext, LLVMPositionBuilderAtEnd, LLVMPrintModuleToString,
    },
    prelude::*,
};
use normalize_line_endings::normalized;
use std::{
    env,
    ffi::{CStr, CString},
    fmt::{self, Debug, Formatter},
    fs,
    ops::Deref,
    path::PathBuf,
    ptr,
};

const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";
pub(crate) struct Context(LLVMContextRef);

impl Context {
    pub(crate) unsafe fn new(context: LLVMContextRef) -> Self {
        Self(context)
    }
}

impl Deref for Context {
    type Target = LLVMContextRef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.0);
        }
    }
}

pub(crate) struct Message(*mut c_char);

impl Message {
    pub(crate) unsafe fn new(message: *mut c_char) -> Self {
        Self(message)
    }
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", unsafe { CStr::from_ptr(self.0) })
    }
}

impl Deref for Message {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0) }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.0);
        }
    }
}

pub(crate) struct Builder(LLVMBuilderRef);

impl Builder {
    pub(crate) unsafe fn new(builder: LLVMBuilderRef) -> Self {
        Self(builder)
    }
}

impl Deref for Builder {
    type Target = LLVMBuilderRef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.0);
        }
    }
}

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
        let mut error = ptr::null_mut();
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

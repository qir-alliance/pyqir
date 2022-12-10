// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::values;
use const_str::{cstr, raw_cstr};
use libc::c_char;
#[allow(clippy::wildcard_imports)]
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::*,
    prelude::*,
    LLVMBuilder, LLVMContext,
};
use normalize_line_endings::normalized;
use std::{
    env,
    ffi::{CStr, CString},
    fmt::{self, Debug, Formatter},
    fs,
    ops::Deref,
    path::PathBuf,
    ptr::{self, NonNull},
};

const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";

pub(crate) struct Context(NonNull<LLVMContext>);

impl Context {
    pub(crate) unsafe fn new(context: NonNull<LLVMContext>) -> Self {
        Self(context)
    }
}

impl Deref for Context {
    type Target = NonNull<LLVMContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.0.as_ptr());
        }
    }
}

pub(crate) struct Message(NonNull<c_char>);

impl Message {
    pub(crate) unsafe fn new(message: NonNull<c_char>) -> Self {
        Self(message)
    }
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", unsafe { CStr::from_ptr(self.0.as_ptr()) })
    }
}

impl Deref for Message {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0.as_ptr()) }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.0.as_ptr());
        }
    }
}

pub(crate) struct Builder(NonNull<LLVMBuilder>);

impl Builder {
    pub(crate) unsafe fn new(builder: NonNull<LLVMBuilder>) -> Self {
        Self(builder)
    }
}

impl Deref for Builder {
    type Target = NonNull<LLVMBuilder>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.0.as_ptr());
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
            cstr!("main"),
            required_num_qubits,
            required_num_results,
        );

        let builder = LLVMCreateBuilderInContext(context);
        let builder = Builder::new(NonNull::new(builder).unwrap());
        LLVMPositionBuilderAtEnd(
            builder.as_ptr(),
            LLVMAppendBasicBlockInContext(context, entry_point, raw_cstr!("")),
        );
        build(builder.as_ptr());
        LLVMBuildRetVoid(builder.as_ptr());

        let action = LLVMVerifierFailureAction::LLVMReturnStatusAction;
        let mut error = ptr::null_mut();
        if LLVMVerifyModule(module, action, &mut error) == 0 {
            let ir = LLVMPrintModuleToString(module);
            Ok(Message::new(NonNull::new(ir).unwrap()))
        } else {
            Err(Message::new(NonNull::new(error).unwrap()))
        }
    }
}

fn split_id(id: &str) -> (Vec<&str>, &str) {
    let mut parts: Vec<_> = id.split('/').collect();
    let name = parts.pop().expect("Empty string.");
    (parts, name)
}

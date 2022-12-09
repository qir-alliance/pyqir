use libc::c_char;
use llvm_sys::{
    core::{LLVMContextDispose, LLVMDisposeBuilder, LLVMDisposeMessage},
    prelude::*,
};
use std::ops::Deref;
use std::{
    ffi::CStr,
    fmt::{self, Debug, Formatter},
};

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

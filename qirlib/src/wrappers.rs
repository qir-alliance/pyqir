use llvm_sys::{core::LLVMContextDispose, prelude::*};
use std::ops::Deref;

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

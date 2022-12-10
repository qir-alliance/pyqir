// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use libc::c_char;
use llvm_sys::{
    core::{LLVMDisposeMemoryBuffer, LLVMDisposeMessage},
    prelude::*,
    LLVMMemoryBuffer,
};
use std::{ffi::CStr, ops::Deref, ptr::NonNull};

pub(crate) struct Message(NonNull<c_char>);

impl Message {
    pub(crate) unsafe fn new(message: NonNull<c_char>) -> Self {
        Self(message)
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
        unsafe { LLVMDisposeMessage(self.0.as_ptr()) }
    }
}

pub(crate) struct MemoryBuffer(NonNull<LLVMMemoryBuffer>);

impl MemoryBuffer {
    pub(crate) unsafe fn new(buffer: NonNull<LLVMMemoryBuffer>) -> Self {
        Self(buffer)
    }

    pub(crate) fn as_ptr(&self) -> LLVMMemoryBufferRef {
        self.0.as_ptr()
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMemoryBuffer(self.0.as_ptr());
        }
    }
}

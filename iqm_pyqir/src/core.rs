// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use llvm_sys::core::LLVMContextCreate;
#[allow(deprecated)]
use llvm_sys::{
    core::{LLVMContextDispose, LLVMDisposeMemoryBuffer, LLVMDisposeMessage},
    prelude::*,
    LLVMContext, LLVMMemoryBuffer,
};
use pyo3::prelude::*;
use std::{
    ffi::{c_char, CStr},
    ops::Deref,
    ptr::NonNull,
};

/// The context owns global state needed by most LLVM objects.
#[pyclass(unsendable)]
#[derive(Eq, PartialEq)]
pub(crate) struct Context(NonNull<LLVMContext>);

#[pymethods]
impl Context {
    #[new]
    pub(crate) fn new() -> Self {
        Self(NonNull::new(unsafe { LLVMContextCreate() }).unwrap())
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

pub(crate) struct MemoryBuffer(NonNull<LLVMMemoryBuffer>);

impl MemoryBuffer {
    pub(crate) unsafe fn from_raw(buffer: LLVMMemoryBufferRef) -> Self {
        Self(NonNull::new(buffer).expect("Memory buffer is null."))
    }
}

impl Deref for MemoryBuffer {
    type Target = NonNull<LLVMMemoryBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMemoryBuffer(self.0.as_ptr());
        }
    }
}

pub(crate) struct Message(NonNull<c_char>);

impl Message {
    pub(crate) unsafe fn from_raw(message: *mut c_char) -> Self {
        Self(NonNull::new(message).expect("Message is null."))
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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types;
use const_str::raw_cstr;
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, prelude::*, LLVMLinkage};
use std::{ffi::CString, ptr::NonNull};

#[derive(Clone, Copy)]
pub(crate) enum Functor {
    Body,
    Adjoint,
}

pub(crate) unsafe fn build_call(
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    args: &mut [LLVMValueRef],
) -> LLVMValueRef {
    #[allow(deprecated)]
    LLVMBuildCall(
        builder,
        function,
        args.as_mut_ptr(),
        args.len().try_into().unwrap(),
        raw_cstr!(""),
    )
}

pub(crate) unsafe fn builder_module(builder: LLVMBuilderRef) -> LLVMModuleRef {
    NonNull::new(LLVMGetInsertBlock(builder))
        .and_then(|b| NonNull::new(LLVMGetBasicBlockParent(b.as_ptr())))
        .and_then(|v| NonNull::new(LLVMGetGlobalParent(v.as_ptr())))
        .expect("The builder's position has not been set.")
        .as_ptr()
}

pub(crate) unsafe fn no_param(module: LLVMModuleRef, name: &str, functor: Functor) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut []);
    declare_qis(module, name, functor, ty)
}

pub(crate) unsafe fn simple_gate(
    module: LLVMModuleRef,
    name: &str,
    functor: Functor,
) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [types::qubit(context)]);
    declare_qis(module, name, functor, ty)
}

pub(crate) unsafe fn two_qubit_gate(
    module: LLVMModuleRef,
    name: &str,
    functor: Functor,
) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let qubit = types::qubit(context);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [qubit, qubit]);
    declare_qis(module, name, functor, ty)
}

pub(crate) unsafe fn controlled_gate(module: LLVMModuleRef, name: &str) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let qubit = types::qubit(context);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [qubit, qubit]);
    declare_qis(module, name, Functor::Body, ty)
}

pub(crate) unsafe fn doubly_controlled_gate(module: LLVMModuleRef, name: &str) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let qubit = types::qubit(context);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [qubit, qubit, qubit]);
    declare_qis(module, name, Functor::Body, ty)
}

pub(crate) unsafe fn rotation_gate(module: LLVMModuleRef, name: &str) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMVoidTypeInContext(context),
        &mut [LLVMDoubleTypeInContext(context), types::qubit(context)],
    );
    declare_qis(module, name, Functor::Body, ty)
}

pub(crate) unsafe fn function_type(ret: LLVMTypeRef, params: &mut [LLVMTypeRef]) -> LLVMTypeRef {
    LLVMFunctionType(
        ret,
        params.as_mut_ptr(),
        params.len().try_into().unwrap(),
        0,
    )
}

pub(crate) unsafe fn declare_qis(
    module: LLVMModuleRef,
    name: &str,
    functor: Functor,
    ty: LLVMTypeRef,
) -> LLVMValueRef {
    let suffix = match functor {
        Functor::Body => "body",
        Functor::Adjoint => "adj",
    };
    let name = format!("__quantum__qis__{name}__{suffix}");
    declare_external_function(module, name.as_str(), ty)
}

pub(crate) unsafe fn declare_external_function(
    module: LLVMModuleRef,
    name: &str,
    ty: LLVMTypeRef,
) -> LLVMValueRef {
    let name =
        CString::new(name).expect("Could not create declaration from name containing a null byte");
    let function = LLVMGetNamedFunction(module, name.as_ptr().cast());
    if function.is_null() {
        let function = LLVMAddFunction(module, name.as_ptr().cast(), ty);
        LLVMSetLinkage(function, LLVMLinkage::LLVMExternalLinkage);
        function
    } else {
        function
    }
}

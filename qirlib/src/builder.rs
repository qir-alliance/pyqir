// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use llvm_sys::{
    core::{
        LLVMAppendBasicBlockInContext, LLVMBuildBr, LLVMBuildCondBr, LLVMGetBasicBlockParent,
        LLVMGetInsertBlock, LLVMGetTypeContext, LLVMPositionBuilderAtEnd, LLVMTypeOf,
    },
    prelude::*,
};
use std::{convert::Infallible, ptr::NonNull};

pub unsafe fn build_if(
    builder: LLVMBuilderRef,
    cond: LLVMValueRef,
    build_true: impl FnOnce(),
    build_false: impl FnOnce(),
) {
    let always_ok: Result<(), Infallible> = Ok(());
    try_build_if(
        builder,
        cond,
        || {
            build_true();
            always_ok
        },
        || {
            build_false();
            always_ok
        },
    )
    .unwrap();
}

pub unsafe fn try_build_if<E>(
    builder: LLVMBuilderRef,
    cond: LLVMValueRef,
    build_true: impl FnOnce() -> Result<(), E>,
    build_false: impl FnOnce() -> Result<(), E>,
) -> Result<(), E> {
    let function = NonNull::new(LLVMGetInsertBlock(builder))
        .and_then(|b| NonNull::new(LLVMGetBasicBlockParent(b.as_ptr())))
        .expect("The builder's position has not been set.")
        .as_ptr();

    let context = LLVMGetTypeContext(LLVMTypeOf(function));
    let then_block = LLVMAppendBasicBlockInContext(context, function, b"then\0".as_ptr().cast());
    let else_block = LLVMAppendBasicBlockInContext(context, function, b"else\0".as_ptr().cast());
    LLVMBuildCondBr(builder, cond, then_block, else_block);
    let continue_block =
        LLVMAppendBasicBlockInContext(context, function, b"continue\0".as_ptr().cast());

    LLVMPositionBuilderAtEnd(builder, then_block);
    build_true()?;
    LLVMBuildBr(builder, continue_block);

    LLVMPositionBuilderAtEnd(builder, else_block);
    build_false()?;
    LLVMBuildBr(builder, continue_block);

    LLVMPositionBuilderAtEnd(builder, continue_block);
    Ok(())
}

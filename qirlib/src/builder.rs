// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{builder::Builder, values::IntValue, LLVMReference};
use llvm_sys::{core::LLVMBuildCondBr, prelude::*};
use std::convert::Infallible;

pub trait Ext {
    fn build_if(&self, cond: IntValue, build_true: impl FnOnce(), build_false: impl FnOnce());

    #[allow(clippy::missing_errors_doc)]
    fn try_build_if<E>(
        &self,
        cond: IntValue,
        build_true: impl FnOnce() -> Result<(), E>,
        build_false: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), E>;
}

impl Ext for Builder<'_> {
    fn build_if(&self, cond: IntValue, build_true: impl FnOnce(), build_false: impl FnOnce()) {
        unsafe { build_if_unchecked(self, cond.get_ref(), build_true, build_false) }
    }

    fn try_build_if<E>(
        &self,
        cond: IntValue,
        build_true: impl FnOnce() -> Result<(), E>,
        build_false: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), E> {
        unsafe { try_build_if_unchecked(self, cond.get_ref(), build_true, build_false) }
    }
}

pub(crate) unsafe fn build_if_unchecked(
    builder: &Builder,
    cond: LLVMValueRef,
    build_true: impl FnOnce(),
    build_false: impl FnOnce(),
) {
    let always_ok: Result<(), Infallible> = Ok(());
    try_build_if_unchecked(
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

pub(crate) unsafe fn try_build_if_unchecked<E>(
    builder: &Builder,
    cond: LLVMValueRef,
    build_true: impl FnOnce() -> Result<(), E>,
    build_false: impl FnOnce() -> Result<(), E>,
) -> Result<(), E> {
    let insert_block = builder.get_insert_block().unwrap();
    let context = insert_block.get_context();
    let function = insert_block.get_parent().unwrap();
    let then_block = context.append_basic_block(function, "then");
    let else_block = context.append_basic_block(function, "else");
    LLVMBuildCondBr(
        builder.get_ref(),
        cond,
        then_block.get_ref(),
        else_block.get_ref(),
    );

    let continue_block = context.append_basic_block(function, "continue");

    builder.position_at_end(then_block);
    build_true()?;
    builder.build_unconditional_branch(continue_block);

    builder.position_at_end(else_block);
    build_false()?;
    builder.build_unconditional_branch(continue_block);

    builder.position_at_end(continue_block);
    Ok(())
}

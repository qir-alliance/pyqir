// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{builder::Builder as BuilderBase, module::Module, values::IntValue, LLVMReference};
use llvm_sys::{core::LLVMBuildCondBr, prelude::*};
use std::{borrow::Borrow, convert::Infallible, ops::Deref};

// TODO: With LLVM, it's possible to get the module that a builder is positioned in using only the
// builder itself. But it's not possible with Inkwell, so we have to bundle the references together.
// See https://github.com/TheDan64/inkwell/issues/347
pub struct Builder<'ctx, 'a> {
    builder: OwnOrBorrow<'a, BuilderBase<'ctx>>,
    module: &'a Module<'ctx>,
}

impl<'ctx, 'a> Deref for Builder<'ctx, 'a> {
    type Target = BuilderBase<'ctx>;

    fn deref(&self) -> &Self::Target {
        match &self.builder {
            OwnOrBorrow::Owned(b) => b,
            OwnOrBorrow::Borrowed(b) => b,
        }
    }
}

impl<'ctx, 'a> Builder<'ctx, 'a> {
    pub fn new(module: &'a Module<'ctx>) -> Self {
        Self {
            builder: OwnOrBorrow::Owned(module.get_context().create_builder()),
            module,
        }
    }

    pub fn from(builder: &'a BuilderBase<'ctx>, module: &'a Module<'ctx>) -> Self {
        Self {
            builder: OwnOrBorrow::Borrowed(builder),
            module,
        }
    }

    #[must_use]
    pub fn module(&self) -> &Module<'ctx> {
        self.module.borrow()
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn build_if(&self, cond: IntValue, build_true: impl FnOnce(), build_false: impl FnOnce()) {
        unsafe { build_if_unchecked(self, cond.get_ref(), build_true, build_false) }
    }

    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    pub fn try_build_if<E>(
        &self,
        cond: IntValue,
        build_true: impl FnOnce() -> Result<(), E>,
        build_false: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), E> {
        unsafe { try_build_if_unchecked(self, cond.get_ref(), build_true, build_false) }
    }
}

enum OwnOrBorrow<'a, T> {
    Owned(T),
    Borrowed(&'a T),
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

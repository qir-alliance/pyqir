use crate::types;
use inkwell::{
    builder::Builder as BuilderBase,
    module::Module,
    values::{IntValue, PointerValue},
};
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

    #[must_use]
    pub fn build_qubit(&self, id: u64) -> PointerValue<'ctx> {
        let value = self.module().get_context().i64_type().const_int(id, false);
        self.build_int_to_ptr(value, types::qubit(self.module()), "")
    }

    #[must_use]
    pub fn build_result(&self, id: u64) -> PointerValue<'ctx> {
        let value = self.module().get_context().i64_type().const_int(id, false);
        self.build_int_to_ptr(value, types::result(self.module()), "")
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn build_if(
        &self,
        cond: IntValue,
        build_true: impl FnOnce(&Self),
        build_false: impl FnOnce(&Self),
    ) {
        let always_ok: Result<(), Infallible> = Ok(());
        self.try_build_if(
            cond,
            |builder| {
                build_true(builder);
                always_ok
            },
            |builder| {
                build_false(builder);
                always_ok
            },
        )
        .unwrap();
    }

    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    pub fn try_build_if<E>(
        &self,
        cond: IntValue,
        build_true: impl FnOnce(&Self) -> Result<(), E>,
        build_false: impl FnOnce(&Self) -> Result<(), E>,
    ) -> Result<(), E> {
        let insert_block = self.get_insert_block().unwrap();
        let context = insert_block.get_context();
        let function = insert_block.get_parent().unwrap();
        let then_block = context.append_basic_block(function, "then");
        let else_block = context.append_basic_block(function, "else");
        self.build_conditional_branch(cond, then_block, else_block);
        let continue_block = context.append_basic_block(function, "continue");

        self.position_at_end(then_block);
        build_true(self)?;
        self.build_unconditional_branch(continue_block);

        self.position_at_end(else_block);
        build_false(self)?;
        self.build_unconditional_branch(continue_block);

        self.position_at_end(continue_block);
        Ok(())
    }
}

enum OwnOrBorrow<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

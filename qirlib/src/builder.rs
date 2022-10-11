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
}

impl<'ctx, 'a> Builder<'ctx, 'a> {
    pub fn from(builder: &'a BuilderBase<'ctx>, module: &'a Module<'ctx>) -> Self {
        Self {
            builder: OwnOrBorrow::Borrowed(builder),
            module,
        }
    }
}

impl<'ctx, 'a> Builder<'ctx, 'a> {
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

#[cfg(test)]
mod tests {
    use crate::{qis::BuilderBasicQisExt, tests::assert_reference_ir};

    #[test]
    fn test_empty_if() -> Result<(), String> {
        assert_reference_ir("builder/test_empty_if", 1, 1, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_if_result(builder.build_result(0), |_| (), |_| ());
        })
    }

    #[test]
    fn test_if_then() -> Result<(), String> {
        assert_reference_ir("builder/test_if_then", 1, 1, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_if_result(
                builder.build_result(0),
                |builder| builder.build_x(builder.build_qubit(0)),
                |_| (),
            );
        })
    }

    #[test]
    fn test_if_else() -> Result<(), String> {
        assert_reference_ir("builder/test_if_else", 1, 1, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_if_result(
                builder.build_result(0),
                |_| (),
                |builder| builder.build_x(builder.build_qubit(0)),
            );
        })
    }

    #[test]
    fn test_if_then_continue() -> Result<(), String> {
        assert_reference_ir("builder/test_if_then_continue", 1, 1, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_if_result(
                builder.build_result(0),
                |builder| builder.build_x(builder.build_qubit(0)),
                |_| (),
            );
            builder.build_h(builder.build_qubit(0));
        })
    }

    #[test]
    fn test_if_else_continue() -> Result<(), String> {
        assert_reference_ir("builder/test_if_else_continue", 1, 1, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_if_result(
                builder.build_result(0),
                |_| (),
                |builder| builder.build_x(builder.build_qubit(0)),
            );
            builder.build_h(builder.build_qubit(0));
        })
    }

    #[test]
    fn test_if_then_else_continue() -> Result<(), String> {
        assert_reference_ir("builder/test_if_then_else_continue", 1, 1, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_if_result(
                builder.build_result(0),
                |builder| builder.build_x(builder.build_qubit(0)),
                |builder| builder.build_y(builder.build_qubit(0)),
            );
            builder.build_h(builder.build_qubit(0));
        })
    }

    #[test]
    fn test_if_then_then() -> Result<(), String> {
        assert_reference_ir("builder/test_if_then_then", 1, 2, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_mz(builder.build_qubit(0), builder.build_result(1));
            builder.build_if_result(
                builder.build_result(0),
                |builder| {
                    builder.build_if_result(
                        builder.build_result(1),
                        |builder| builder.build_x(builder.build_qubit(0)),
                        |_| (),
                    );
                },
                |_| (),
            );
        })
    }

    #[test]
    fn test_if_else_else() -> Result<(), String> {
        assert_reference_ir("builder/test_if_else_else", 1, 2, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_mz(builder.build_qubit(0), builder.build_result(1));
            builder.build_if_result(
                builder.build_result(0),
                |_| (),
                |builder| {
                    builder.build_if_result(
                        builder.build_result(1),
                        |_| (),
                        |builder| builder.build_x(builder.build_qubit(0)),
                    );
                },
            );
        })
    }

    #[test]
    fn test_if_then_else() -> Result<(), String> {
        assert_reference_ir("builder/test_if_then_else", 1, 2, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_mz(builder.build_qubit(0), builder.build_result(1));
            builder.build_if_result(
                builder.build_result(0),
                |builder| {
                    builder.build_if_result(
                        builder.build_result(1),
                        |_| (),
                        |builder| builder.build_x(builder.build_qubit(0)),
                    );
                },
                |_| (),
            );
        })
    }

    #[test]
    fn test_if_else_then() -> Result<(), String> {
        assert_reference_ir("builder/test_if_else_then", 1, 2, |builder| {
            builder.build_mz(builder.build_qubit(0), builder.build_result(0));
            builder.build_mz(builder.build_qubit(0), builder.build_result(1));
            builder.build_if_result(
                builder.build_result(0),
                |_| (),
                |builder| {
                    builder.build_if_result(
                        builder.build_result(1),
                        |builder| builder.build_x(builder.build_qubit(0)),
                        |_| (),
                    );
                },
            );
        })
    }

    #[test]
    fn test_allows_unmeasured_result_condition() -> Result<(), String> {
        assert_reference_ir(
            "builder/test_allows_unmeasured_result_condition",
            1,
            1,
            |builder| {
                builder.build_if_result(
                    builder.build_result(0),
                    |builder| builder.build_x(builder.build_qubit(0)),
                    |builder| builder.build_h(builder.build_qubit(0)),
                );
            },
        )
    }
}

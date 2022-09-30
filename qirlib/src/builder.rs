use crate::types;
use inkwell::{
    builder::Builder,
    module::Module,
    values::{IntValue, PointerValue},
};
use std::{borrow::Borrow, ops::Deref};

// TODO: With LLVM, it's possible to get the module that a builder is positioned in using only the
// builder itself. But it's not possible with Inkwell, so we have to bundle the references together.
// See https://github.com/TheDan64/inkwell/issues/347
#[allow(clippy::module_name_repetitions)]
pub struct ModuleBuilder<'ctx, 'm, B> {
    builder: B,
    module: &'m Module<'ctx>,
}

impl<'ctx, 'm, B: Borrow<Builder<'ctx>>> Deref for ModuleBuilder<'ctx, 'm, B> {
    type Target = Builder<'ctx>;

    fn deref(&self) -> &Self::Target {
        self.builder.borrow()
    }
}

impl<'ctx, 'm> ModuleBuilder<'ctx, 'm, Builder<'ctx>> {
    pub fn new(module: &'m Module<'ctx>) -> Self {
        Self {
            builder: module.get_context().create_builder(),
            module,
        }
    }
}

impl<'ctx, 'b, 'm> ModuleBuilder<'ctx, 'm, &'b Builder<'ctx>> {
    pub fn from(builder: &'b Builder<'ctx>, module: &'m Module<'ctx>) -> Self {
        Self { builder, module }
    }
}

impl<'ctx, 'm, B: Borrow<Builder<'ctx>>> ModuleBuilder<'ctx, 'm, B> {
    pub fn module(&self) -> &Module<'ctx> {
        self.module.borrow()
    }

    pub fn build_qubit(&self, id: u64) -> PointerValue<'ctx> {
        let value = self.module().get_context().i64_type().const_int(id, false);
        self.build_int_to_ptr(value, types::qubit(self.module()), "")
    }

    pub fn build_result(&self, id: u64) -> PointerValue<'ctx> {
        let value = self.module().get_context().i64_type().const_int(id, false);
        self.build_int_to_ptr(value, types::result(self.module()), "")
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait BuilderExt {
    fn build_if(&self, cond: IntValue, build_true: impl Fn(), build_false: impl Fn());

    #[allow(clippy::missing_errors_doc)]
    fn try_build_if<E>(
        &self,
        cond: IntValue,
        build_true: impl Fn() -> Result<(), E>,
        build_false: impl Fn() -> Result<(), E>,
    ) -> Result<(), E>;
}

impl<'ctx> BuilderExt for Builder<'ctx> {
    fn build_if(&self, cond: IntValue, build_true: impl Fn(), build_false: impl Fn()) {
        self.try_build_if::<()>(
            cond,
            || {
                build_true();
                Ok(())
            },
            || {
                build_false();
                Ok(())
            },
        )
        .unwrap();
    }

    fn try_build_if<E>(
        &self,
        cond: IntValue,
        build_true: impl Fn() -> Result<(), E>,
        build_false: impl Fn() -> Result<(), E>,
    ) -> Result<(), E> {
        let insert_block = self.get_insert_block().unwrap();
        let context = insert_block.get_context();
        let function = insert_block.get_parent().unwrap();

        let then_block = context.append_basic_block(function, "then");
        let else_block = context.append_basic_block(function, "else");
        self.build_conditional_branch(cond, then_block, else_block);

        let continue_block = context.append_basic_block(function, "continue");

        self.position_at_end(then_block);
        build_true()?;
        self.build_unconditional_branch(continue_block);

        self.position_at_end(else_block);
        build_false()?;
        self.build_unconditional_branch(continue_block);

        self.position_at_end(continue_block);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ModuleBuilder;
    use crate::{module, qis::BuilderBasicExt};
    use inkwell::{builder::Builder, context::Context};
    use normalize_line_endings::normalized;
    use std::{env, fs, path::PathBuf};

    #[test]
    fn test_empty_if() -> Result<(), String> {
        check_or_save_reference_ir("test_empty_if", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_if_result(builder.build_result(0).into(), || (), || ());
        })
    }

    #[test]
    fn test_if_then() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || builder.build_x(builder.build_qubit(0).into()),
                || (),
            );
        })
    }

    #[test]
    fn test_if_else() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || (),
                || builder.build_x(builder.build_qubit(0).into()),
            );
        })
    }

    #[test]
    fn test_if_then_continue() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_continue", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || builder.build_x(builder.build_qubit(0).into()),
                || (),
            );
            builder.build_h(builder.build_qubit(0).into());
        })
    }

    #[test]
    fn test_if_else_continue() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else_continue", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || (),
                || builder.build_x(builder.build_qubit(0).into()),
            );
            builder.build_h(builder.build_qubit(0).into());
        })
    }

    #[test]
    fn test_if_then_else_continue() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_else_continue", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || builder.build_x(builder.build_qubit(0).into()),
                || builder.build_y(builder.build_qubit(0).into()),
            );
            builder.build_h(builder.build_qubit(0).into());
        })
    }

    #[test]
    fn test_if_then_then() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_then", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(1).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || {
                    builder.build_if_result(
                        builder.build_result(1).into(),
                        || builder.build_x(builder.build_qubit(0).into()),
                        || (),
                    );
                },
                || (),
            );
        })
    }

    #[test]
    fn test_if_else_else() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else_else", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(1).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || (),
                || {
                    builder.build_if_result(
                        builder.build_result(1).into(),
                        || (),
                        || builder.build_x(builder.build_qubit(0).into()),
                    );
                },
            );
        })
    }

    #[test]
    fn test_if_then_else() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_else", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(1).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || {
                    builder.build_if_result(
                        builder.build_result(1).into(),
                        || (),
                        || builder.build_x(builder.build_qubit(0).into()),
                    );
                },
                || (),
            );
        })
    }

    #[test]
    fn test_if_else_then() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else_then", |builder| {
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(0).into(),
            );
            builder.build_mz(
                builder.build_qubit(0).into(),
                builder.build_result(1).into(),
            );
            builder.build_if_result(
                builder.build_result(0).into(),
                || (),
                || {
                    builder.build_if_result(
                        builder.build_result(1).into(),
                        || builder.build_x(builder.build_qubit(0).into()),
                        || (),
                    );
                },
            );
        })
    }

    #[test]
    fn test_allows_unmeasured_result_condition() -> Result<(), String> {
        check_or_save_reference_ir("test_allows_unmeasured_result_condition", |builder| {
            builder.build_if_result(
                builder.build_result(0).into(),
                || builder.build_x(builder.build_qubit(0).into()),
                || builder.build_h(builder.build_qubit(0).into()),
            );
        })
    }

    /// Compares generated IR against reference files in the "resources/tests" folder. If changes
    /// to code generation break the tests:
    ///
    /// 1. Run the tests with the `PYQIR_TEST_SAVE_REFERENCES` environment variable set to
    ///    regenerate the reference files.
    /// 2. Review the changes and make sure they look reasonable.
    /// 3. Unset the environment variable and run the tests again to confirm that they pass.
    fn check_or_save_reference_ir(
        name: &str,
        build: impl for<'ctx> Fn(&ModuleBuilder<'ctx, '_, Builder<'ctx>>),
    ) -> Result<(), String> {
        const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";
        let actual_ir = build_ir(name, build)?;

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources");
        path.push("tests");
        path.push(name);
        path.set_extension("ll");

        if env::var(PYQIR_TEST_SAVE_REFERENCES).is_ok() {
            fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::write(&path, actual_ir).map_err(|e| e.to_string())?;

            Err(format!(
                "Saved reference IR. Run again without the {} environment variable.",
                PYQIR_TEST_SAVE_REFERENCES
            ))
        } else {
            let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let expected_ir: String = normalized(contents.chars()).collect();
            assert_eq!(expected_ir, actual_ir);
            Ok(())
        }
    }

    fn build_ir(
        name: &str,
        build: impl for<'ctx> Fn(&ModuleBuilder<'ctx, '_, Builder<'ctx>>),
    ) -> Result<String, String> {
        let context = Context::create();
        let module = context.create_module(name);
        let builder = ModuleBuilder::new(&module);
        module::build_entry_point(&module, &builder);
        build(&builder);
        builder.build_return(None);
        module::run_basic_passes_on(builder.module());
        module.verify().map_err(|e| e.to_string())?;
        Ok(module.print_to_string().to_string())
    }
}

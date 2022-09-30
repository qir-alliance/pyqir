use crate::{module, qis};
use inkwell::{
    builder::Builder,
    module::Module,
    values::{BasicMetadataValueEnum, IntValue},
};
use std::ops::Deref;

// TODO: With LLVM, it's possible to get the module that a builder is positioned in using only the
// builder itself. But it's not possible with Inkwell, so we have to bundle the references together.
// See https://github.com/TheDan64/inkwell/issues/347
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy)]
pub struct BuilderRef<'ctx, 'a> {
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
}

impl<'ctx, 'a> Deref for BuilderRef<'ctx, 'a> {
    type Target = Builder<'ctx>;

    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl<'ctx, 'a> BuilderRef<'ctx, 'a> {
    pub fn new(builder: &'a Builder<'ctx>, module: &'a Module<'ctx>) -> Self {
        Self { builder, module }
    }

    pub(crate) fn module(&self) -> &'a Module<'ctx> {
        self.module
    }
}

pub fn init(builder: BuilderRef) {
    let module = builder.module();
    let context = module.get_context();
    let entry_point = module::create_entry_point(module);
    let entry = context.append_basic_block(entry_point, "entry");
    builder.position_at_end(entry);
}

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
pub fn if_then<E>(
    builder: &Builder,
    cond: IntValue,
    build_true: impl Fn() -> Result<(), E>,
    build_false: impl Fn() -> Result<(), E>,
) -> Result<(), E> {
    let insert_block = builder.get_insert_block().unwrap();
    let context = insert_block.get_context();
    let function = insert_block.get_parent().unwrap();

    let then_block = context.append_basic_block(function, "then");
    let else_block = context.append_basic_block(function, "else");
    builder.build_conditional_branch(cond, then_block, else_block);

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

#[allow(clippy::missing_errors_doc)]
pub fn if_result<'ctx, E>(
    builder: BuilderRef<'ctx, '_>,
    cond: BasicMetadataValueEnum<'ctx>,
    build_one: impl Fn() -> Result<(), E>,
    build_zero: impl Fn() -> Result<(), E>,
) -> Result<(), E> {
    let bool_cond = qis::call_read_result(builder, cond);
    if_then(&builder, bool_cond, build_one, build_zero)
}

#[cfg(test)]
mod tests {
    use crate::{
        build::{self, BuilderRef},
        passes::run_basic_passes_on,
        qis,
        types::{qubit_id, result_id},
    };
    use inkwell::context::Context;
    use normalize_line_endings::normalized;
    use std::{env, fs, path::PathBuf};

    #[test]
    fn test_empty_if() -> Result<(), String> {
        check_or_save_reference_ir("test_empty_if", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            build::if_result::<String>(builder, result_id(builder, 0).into(), || Ok(()), || Ok(()))
                .unwrap();
        })
    }

    #[test]
    fn test_if_then() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            build::if_result::<String>(
                builder,
                result_id(builder, 0).into(),
                || {
                    qis::call_x(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
                || Ok(()),
            )
            .unwrap();
        })
    }

    #[test]
    fn test_if_else() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            build::if_result::<String>(
                builder,
                result_id(builder, 0).into(),
                || Ok(()),
                || {
                    qis::call_x(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
            )
            .unwrap();
        })
    }

    #[test]
    fn test_if_then_continue() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_continue", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            build::if_result::<String>(
                builder,
                result_id(builder, 0).into(),
                || {
                    qis::call_x(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
                || Ok(()),
            )
            .unwrap();
            qis::call_h(builder, qubit_id(builder, 0).into());
        })
    }

    #[test]
    fn test_if_else_continue() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else_continue", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            build::if_result::<String>(
                builder,
                result_id(builder, 0).into(),
                || Ok(()),
                || {
                    qis::call_x(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
            )
            .unwrap();
            qis::call_h(builder, qubit_id(builder, 0).into());
        })
    }

    #[test]
    fn test_if_then_else_continue() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_else_continue", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            build::if_result::<String>(
                builder,
                result_id(builder, 0).into(),
                || {
                    qis::call_x(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
                || {
                    qis::call_y(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
            )
            .unwrap();
            qis::call_h(builder, qubit_id(builder, 0).into());
        })
    }

    #[test]
    fn test_if_then_then() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_then", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 1).into(),
            );
            build::if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || {
                    build::if_result::<()>(
                        builder,
                        result_id(builder, 1).into(),
                        || {
                            qis::call_x(builder, qubit_id(builder, 0).into());
                            Ok(())
                        },
                        || Ok(()),
                    )
                    .unwrap();
                    Ok(())
                },
                || Ok(()),
            )
            .unwrap();
        })
    }

    #[test]
    fn test_if_else_else() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else_else", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 1).into(),
            );
            build::if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || Ok(()),
                || {
                    build::if_result::<()>(
                        builder,
                        result_id(builder, 1).into(),
                        || Ok(()),
                        || {
                            qis::call_x(builder, qubit_id(builder, 0).into());
                            Ok(())
                        },
                    )
                    .unwrap();
                    Ok(())
                },
            )
            .unwrap();
        })
    }

    #[test]
    fn test_if_then_else() -> Result<(), String> {
        check_or_save_reference_ir("test_if_then_else", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 1).into(),
            );
            build::if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || {
                    build::if_result::<()>(
                        builder,
                        result_id(builder, 1).into(),
                        || Ok(()),
                        || {
                            qis::call_x(builder, qubit_id(builder, 0).into());
                            Ok(())
                        },
                    )
                    .unwrap();
                    Ok(())
                },
                || Ok(()),
            )
            .unwrap();
        })
    }

    #[test]
    fn test_if_else_then() -> Result<(), String> {
        check_or_save_reference_ir("test_if_else_then", |builder| {
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 0).into(),
            );
            qis::call_mz(
                builder,
                qubit_id(builder, 0).into(),
                result_id(builder, 1).into(),
            );
            build::if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || Ok(()),
                || {
                    build::if_result::<()>(
                        builder,
                        result_id(builder, 1).into(),
                        || {
                            qis::call_x(builder, qubit_id(builder, 0).into());
                            Ok(())
                        },
                        || Ok(()),
                    )
                    .unwrap();
                    Ok(())
                },
            )
            .unwrap();
        })
    }

    #[test]
    fn test_allows_unmeasured_result_condition() -> Result<(), String> {
        check_or_save_reference_ir("test_allows_unmeasured_result_condition", |builder| {
            build::if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || {
                    qis::call_x(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
                || {
                    qis::call_h(builder, qubit_id(builder, 0).into());
                    Ok(())
                },
            )
            .unwrap();
        })
    }

    /// Compares generated IR against reference files in the "resources/tests" folder. If changes
    /// to code generation break the tests:
    ///
    /// 1. Run the tests with the `PYQIR_TEST_SAVE_REFERENCES` environment variable set to
    ///    regenerate the reference files.
    /// 2. Review the changes and make sure they look reasonable.
    /// 3. Unset the environment variable and run the tests again to confirm that they pass.
    fn check_or_save_reference_ir(name: &str, build: impl Fn(BuilderRef)) -> Result<(), String> {
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

    fn build_ir(name: &str, build: impl Fn(BuilderRef)) -> Result<String, String> {
        let context = Context::create();
        let module = context.create_module(name);
        let builder = context.create_builder();
        let builder = BuilderRef::new(&builder, &module);
        build::init(builder);
        build(builder);
        builder.build_return(None);
        run_basic_passes_on(&module);
        module.verify().map_err(|e| e.to_string())?;
        Ok(module.print_to_string().to_string())
    }
}

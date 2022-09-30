// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests {
    use crate::{
        codegen::{
            qis,
            types::{qubit_id, result_id},
            BuilderRef,
        },
        generation::qir,
        passes::run_basic_passes_on,
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
            qir::build_if_result::<String>(
                builder,
                result_id(builder, 0).into(),
                || Ok(()),
                || Ok(()),
            )
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
            qir::build_if_result::<String>(
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
            qir::build_if_result::<String>(
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
            qir::build_if_result::<String>(
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
            qir::build_if_result::<String>(
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
            qir::build_if_result::<String>(
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
            qir::build_if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || {
                    qir::build_if_result::<()>(
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
            qir::build_if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || Ok(()),
                || {
                    qir::build_if_result::<()>(
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
            qir::build_if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || {
                    qir::build_if_result::<()>(
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
            qir::build_if_result::<()>(
                builder,
                result_id(builder, 0).into(),
                || Ok(()),
                || {
                    qir::build_if_result::<()>(
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
            qir::build_if_result::<()>(
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
        qir::init_module_builder(&module, &builder);
        build(BuilderRef::new(&builder, &module));
        builder.build_return(None);
        run_basic_passes_on(&module);
        module.verify().map_err(|e| e.to_string())?;
        Ok(module.print_to_string().to_string())
    }
}

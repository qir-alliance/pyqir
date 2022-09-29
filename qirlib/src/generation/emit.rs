// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::types,
    generation::{
        env::Environment,
        interop::{self, SemanticModel, Type},
        qir::{self, instructions},
    },
    passes::run_basic_passes_on,
};
use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{AnyTypeEnum, BasicType, BasicTypeEnum},
    values::FunctionValue,
};
use std::convert::{Into, TryFrom};

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn ir(model: &SemanticModel) -> Result<String, String> {
    let context = Context::create();
    let module = context.create_module(&model.name);
    build_entry_function(model, &module)?;
    run_basic_passes_on(&module);
    Ok(module.print_to_string().to_string())
}

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn bitcode(model: &SemanticModel) -> Result<Vec<u8>, String> {
    let context = Context::create();
    let module = context.create_module(&model.name);
    build_entry_function(model, &module)?;
    run_basic_passes_on(&module);
    Ok(module.write_bitcode_to_memory().as_slice().to_vec())
}

fn build_entry_function(model: &SemanticModel, module: &Module) -> Result<(), String> {
    add_external_functions(module, model.external_functions.iter());
    let entry_point = qir::create_entry_point(module);
    let context = module.get_context();
    let entry = context.append_basic_block(entry_point, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry);
    write_instructions(model, module, &builder, entry_point);
    builder.build_return(None);
    module.verify().map_err(|e| e.to_string())
}

fn add_external_functions<'a>(
    module: &Module,
    functions: impl Iterator<Item = &'a (String, interop::Type)>,
) {
    for (name, ty) in functions {
        let ty = get_type(module, ty).into_function_type();
        module.add_function(name, ty, Some(Linkage::External));
    }
}

fn get_type<'ctx>(module: &Module<'ctx>, ty: &Type) -> AnyTypeEnum<'ctx> {
    let context = module.get_context();
    match ty {
        Type::Void => context.void_type().into(),
        &Type::Int { width } => context.custom_width_int_type(width).into(),
        Type::Double => context.f64_type().into(),
        Type::Qubit => types::qubit(module).into(),
        Type::Result => types::result(module).into(),
        Type::Function { params, result } => {
            let params = params
                .iter()
                .map(|ty| {
                    BasicTypeEnum::try_from(get_type(module, ty))
                        .unwrap()
                        .into()
                })
                .collect::<Vec<_>>();

            match get_type(module, result) {
                AnyTypeEnum::VoidType(void) => void.fn_type(&params, false),
                result => BasicTypeEnum::try_from(result)
                    .expect("Invalid return type.")
                    .fn_type(&params, false),
            }
            .into()
        }
    }
}

fn write_instructions<'ctx>(
    model: &SemanticModel,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    entry_point: FunctionValue,
) {
    let mut env = Environment::new();
    for inst in &model.instructions {
        instructions::emit(module, builder, &mut env, inst, entry_point);
    }
}

#[cfg(test)]
mod tests {
    use crate::generation::{
        emit,
        interop::{
            BinaryKind, BinaryOp, Call, IfResult, Instruction, IntPredicate, Measured,
            SemanticModel, Single, Type, Value, Variable,
        },
    };
    use normalize_line_endings::normalized;
    use std::{env, fs, path::PathBuf};

    #[test]
    fn test_empty_if() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_empty_if".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![],
                    if_zero: vec![],
                }),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                    if_zero: vec![],
                }),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![],
                    if_zero: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                }),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_continue() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_continue".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                    if_zero: vec![],
                }),
                Instruction::H(Single::new(Value::Qubit(0))),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else_continue() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else_continue".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![],
                    if_zero: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                }),
                Instruction::H(Single::new(Value::Qubit(0))),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_else_continue() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_else_continue".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                    if_zero: vec![Instruction::Y(Single::new(Value::Qubit(0)))],
                }),
                Instruction::H(Single::new(Value::Qubit(0))),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_then() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_then".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(1))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result(1),
                        if_one: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                        if_zero: vec![],
                    })],
                    if_zero: vec![],
                }),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else_else() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else_else".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(1))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![],
                    if_zero: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result(1),
                        if_one: vec![],
                        if_zero: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                    })],
                }),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_else() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_else".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(1))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result(1),
                        if_one: vec![],
                        if_zero: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                    })],
                    if_zero: vec![],
                }),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else_then() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else_then".to_string(),
            external_functions: vec![],
            instructions: vec![
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(0))),
                Instruction::M(Measured::new(Value::Qubit(0), Value::Result(1))),
                Instruction::IfResult(IfResult {
                    cond: Value::Result(0),
                    if_one: vec![],
                    if_zero: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result(1),
                        if_one: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                        if_zero: vec![],
                    })],
                }),
            ],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_allows_unmeasured_result_condition() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_allows_unmeasured_result_condition".to_string(),
            external_functions: vec![],
            instructions: vec![Instruction::IfResult(IfResult {
                cond: Value::Result(0),
                if_one: vec![Instruction::X(Single::new(Value::Qubit(0)))],
                if_zero: vec![Instruction::H(Single::new(Value::Qubit(0)))],
            })],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_call_variable() -> Result<(), String> {
        let i64 = Type::Int { width: 64 };
        let x = Variable::new();

        check_or_save_reference_ir(&SemanticModel {
            name: "test_call_variable".to_string(),
            external_functions: vec![
                (
                    "foo".to_string(),
                    Type::Function {
                        params: vec![],
                        result: Box::new(i64.clone()),
                    },
                ),
                (
                    "bar".to_string(),
                    Type::Function {
                        params: vec![i64],
                        result: Box::new(Type::Void),
                    },
                ),
            ],
            instructions: vec![
                Instruction::Call(Call {
                    name: "foo".to_string(),
                    args: vec![],
                    result: Some(x),
                }),
                Instruction::Call(Call {
                    name: "bar".to_string(),
                    args: vec![Value::Variable(x)],
                    result: None,
                }),
            ],
        })
    }

    #[test]
    fn test_int_binary_operators() -> Result<(), String> {
        let mut instructions = vec![];
        let lhs = Variable::new();
        let rhs = lhs.next();

        for result in [lhs, rhs] {
            instructions.push(Instruction::Call(Call {
                name: "source".to_string(),
                args: vec![],
                result: Some(result),
            }));
        }

        let kinds = [
            BinaryKind::And,
            BinaryKind::Or,
            BinaryKind::Xor,
            BinaryKind::Add,
            BinaryKind::Sub,
            BinaryKind::Mul,
            BinaryKind::Shl,
            BinaryKind::LShr,
            BinaryKind::ICmp(IntPredicate::EQ),
            BinaryKind::ICmp(IntPredicate::NE),
            BinaryKind::ICmp(IntPredicate::UGT),
            BinaryKind::ICmp(IntPredicate::UGE),
            BinaryKind::ICmp(IntPredicate::ULT),
            BinaryKind::ICmp(IntPredicate::ULE),
            BinaryKind::ICmp(IntPredicate::SGT),
            BinaryKind::ICmp(IntPredicate::SGE),
            BinaryKind::ICmp(IntPredicate::SLT),
            BinaryKind::ICmp(IntPredicate::SLE),
        ];

        let mut result = rhs;
        for kind in kinds {
            let sink = if matches!(kind, BinaryKind::ICmp(_)) {
                result = result.next();
                "sink_i1".to_string()
            } else {
                result = result.next();
                "sink_i32".to_string()
            };

            instructions.push(Instruction::BinaryOp(BinaryOp {
                kind,
                lhs: Value::Variable(lhs),
                rhs: Value::Variable(rhs),
                result,
            }));

            instructions.push(Instruction::Call(Call {
                name: sink,
                args: vec![Value::Variable(result)],
                result: None,
            }));
        }

        check_or_save_reference_ir(&SemanticModel {
            name: "test_int_binary_operators".to_string(),
            external_functions: vec![
                (
                    "source".to_string(),
                    Type::Function {
                        params: vec![],
                        result: Box::new(Type::Int { width: 32 }),
                    },
                ),
                (
                    "sink_i1".to_string(),
                    Type::Function {
                        params: vec![Type::Int { width: 1 }],
                        result: Box::new(Type::Void),
                    },
                ),
                (
                    "sink_i32".to_string(),
                    Type::Function {
                        params: vec![Type::Int { width: 32 }],
                        result: Box::new(Type::Void),
                    },
                ),
            ],
            instructions,
        })
    }

    /// Compares generated IR against reference files in the "resources/tests" folder. If changes
    /// to code generation break the tests:
    ///
    /// 1. Run the tests with the `PYQIR_TEST_SAVE_REFERENCES` environment variable set to
    ///    regenerate the reference files.
    /// 2. Review the changes and make sure they look reasonable.
    /// 3. Unset the environment variable and run the tests again to confirm that they pass.
    fn check_or_save_reference_ir(model: &SemanticModel) -> Result<(), String> {
        const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources");
        path.push("tests");
        path.push(&model.name);
        path.set_extension("ll");

        let actual_ir: String = normalized(emit::ir(model)?.chars()).collect();

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
}

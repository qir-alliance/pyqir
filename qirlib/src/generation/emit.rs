// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::CodeGenerator,
    generation::{
        env::Environment,
        interop::{self, SemanticModel, Type},
        qir,
    },
    passes::run_basic_passes_on,
};
use inkwell::{
    attributes::AttributeLoc,
    context::Context,
    module::Linkage,
    types::{AnyTypeEnum, BasicType, BasicTypeEnum},
    values::{BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use std::{
    collections::HashMap,
    convert::{Into, TryFrom},
};

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn ir(model: &SemanticModel) -> Result<String, String> {
    let ctx = Context::create();
    let generator = populate_context(&ctx, model)?;
    run_basic_passes_on(&generator.module);
    Ok(generator.get_ir())
}

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn bitcode(model: &SemanticModel) -> Result<Vec<u8>, String> {
    let ctx = Context::create();
    let generator = populate_context(&ctx, model)?;
    run_basic_passes_on(&generator.module);
    Ok(generator.get_bitcode().as_slice().to_vec())
}

/// # Errors
///
/// Will return `Err` if
///  - module cannot be loaded.
///  - module fails verification that the current `Module` is valid.
pub fn populate_context<'a>(
    ctx: &'a Context,
    model: &'a SemanticModel,
) -> Result<CodeGenerator<'a>, String> {
    let module = ctx.create_module(&model.name);
    let generator = CodeGenerator::new(ctx, module)?;
    build_entry_function(&generator, model)?;
    Ok(generator)
}

fn build_entry_function(generator: &CodeGenerator, model: &SemanticModel) -> Result<(), String> {
    add_external_functions(generator, model.external_functions.iter());
    let entry_point = qir::create_entry_point(generator.context, &generator.module);

    if !model.qubits.is_empty() {
        let required_qubits = generator
            .context
            .create_string_attribute("requiredQubits", &model.qubits.len().to_string());
        entry_point.add_attribute(AttributeLoc::Function, required_qubits);
    }

    let num_results: u64 = model.registers.iter().map(|f| f.size).sum();
    if num_results > 0 {
        let required_results = generator
            .context
            .create_string_attribute("requiredResults", &num_results.to_string());
        entry_point.add_attribute(AttributeLoc::Function, required_results);
    }

    let entry = generator.context.append_basic_block(entry_point, "entry");
    generator.builder.position_at_end(entry);

    let mut env = Environment::new(
        write_qubits(model, generator),
        write_registers(model, generator),
        HashMap::new(),
    );

    write_instructions(model, generator, &mut env, entry_point);
    generator.builder.build_return(None);
    generator.module.verify().map_err(|e| e.to_string())
}

fn add_external_functions<'a>(
    generator: &CodeGenerator,
    functions: impl Iterator<Item = &'a (String, interop::Type)>,
) {
    for (name, ty) in functions {
        let ty = get_type(generator, ty).into_function_type();
        generator
            .module
            .add_function(name, ty, Some(Linkage::External));
    }
}

fn get_type<'ctx>(generator: &CodeGenerator<'ctx>, ty: &Type) -> AnyTypeEnum<'ctx> {
    match ty {
        Type::Void => generator.context.void_type().into(),
        &Type::Int { width } => generator.context.custom_width_int_type(width).into(),
        Type::Double => generator.context.f64_type().into(),
        Type::Qubit => generator
            .qubit_type()
            .ptr_type(AddressSpace::Generic)
            .into(),
        Type::Result => generator
            .result_type()
            .ptr_type(AddressSpace::Generic)
            .into(),
        Type::Function { params, result } => {
            let params = params
                .iter()
                .map(|ty| {
                    BasicTypeEnum::try_from(get_type(generator, ty))
                        .unwrap()
                        .into()
                })
                .collect::<Vec<_>>();

            match get_type(generator, result) {
                AnyTypeEnum::VoidType(void) => void.fn_type(&params, false),
                result => BasicTypeEnum::try_from(result)
                    .expect("Invalid return type.")
                    .fn_type(&params, false),
            }
            .into()
        }
    }
}

fn write_qubits<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, BasicValueEnum<'ctx>> {
    let mut qubits = HashMap::new();
    for (id, qubit) in model.qubits.iter().enumerate() {
        let indexed_name = format!("{}{}", &qubit.name[..], qubit.index);
        let int_value = generator.usize_to_i64(id).into_int_value();
        let qubit_ptr_type = generator.qubit_type().ptr_type(AddressSpace::Generic);

        let intptr = generator
            .builder
            .build_int_to_ptr(int_value, qubit_ptr_type, &indexed_name);
        qubits.insert(indexed_name, intptr.into());
    }

    qubits
}

fn write_registers<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, BasicValueEnum<'ctx>> {
    let mut registers = HashMap::new();
    let mut id = 0;
    let number_of_registers = model.registers.len() as u64;
    if number_of_registers > 0 {
        for register in &model.registers {
            for index in 0..register.size {
                let indexed_name = format!("{}{}", register.name, index);
                let intptr = create_result_static_ptr(&indexed_name, generator, id);
                registers.insert(indexed_name, intptr.into());
                id += 1;
            }
        }
    }

    registers
}

fn create_result_static_ptr<'ctx>(
    indexed_name: &str,
    generator: &CodeGenerator<'ctx>,
    id: usize,
) -> PointerValue<'ctx> {
    let int_value = generator.usize_to_i64(id).into_int_value();
    let result_ptr_type = generator.result_type().ptr_type(AddressSpace::Generic);
    generator
        .builder
        .build_int_to_ptr(int_value, result_ptr_type, indexed_name)
}

fn write_instructions<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    entry_point: FunctionValue,
) {
    for inst in &model.instructions {
        qir::instructions::emit(generator, env, inst, entry_point);
    }
}

#[cfg(test)]
mod allocation_tests {
    use crate::generation::{
        emit,
        interop::{
            ClassicalRegister, Instruction, Measured, QuantumRegister, SemanticModel, Single, Value,
        },
    };

    #[test]
    fn when_no_qubits_or_registers_declared_then_only_entypoint_attribute_is_emitted(
    ) -> Result<(), String> {
        let model = SemanticModel {
            name: "test".to_owned(),
            registers: vec![],
            qubits: vec![],
            instructions: vec![],
            external_functions: vec![],
        };
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" }"));
        Ok(())
    }

    #[test]
    fn when_no_registers_declared_then_attribute_is_omitted() -> Result<(), String> {
        let model = SemanticModel {
            name: "test".to_owned(),
            registers: vec![],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::H(Single::new(Value::Qubit("q0".to_string())))],
            external_functions: vec![],
        };
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" \"requiredQubits\"=\"1\" }"));
        Ok(())
    }

    #[test]
    fn when_no_qubits_declared_then_attribute_is_omitted() -> Result<(), String> {
        let model = SemanticModel {
            name: "test".to_owned(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![],
            instructions: vec![],
            external_functions: vec![],
        };
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" \"requiredResults\"=\"1\" }"));
        Ok(())
    }

    #[test]
    fn when_qubits_and_registers_declared_then_required_attribute_are_emitted() -> Result<(), String>
    {
        let model = SemanticModel {
            name: "test".to_owned(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::M(Measured::new(
                Value::Qubit("q0".to_string()),
                Value::Result("r0".to_string()),
            ))],
            external_functions: vec![],
        };
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains(
            "attributes #0 = { \"EntryPoint\" \"requiredQubits\"=\"1\" \"requiredResults\"=\"1\" }"
        ));
        Ok(())
    }

    #[test]
    fn required_results_attribute_sums_registers_correctly() -> Result<(), String> {
        let model = SemanticModel {
            name: "test".to_owned(),
            registers: vec![
                ClassicalRegister::new("r".to_string(), 1),
                ClassicalRegister::new("r".to_string(), 3),
                ClassicalRegister::new("r".to_string(), 4),
            ],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::M(Measured::new(
                Value::Qubit("q0".to_string()),
                Value::Result("r0".to_string()),
            ))],
            external_functions: vec![],
        };
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains(
            "attributes #0 = { \"EntryPoint\" \"requiredQubits\"=\"1\" \"requiredResults\"=\"8\" }"
        ));
        Ok(())
    }
}

/// These tests compare generated IR against reference files in the "resources/tests" folder. If
/// changes to code generation break the tests:
///
/// 1. Run the tests with the `PYQIR_TEST_SAVE_REFERENCES` environment variable set to regenerate
///    the reference files.
/// 2. Review the changes and make sure they look reasonable.
/// 3. Unset the environment variable and run the tests again to confirm that they pass.
#[cfg(test)]
mod ir_tests {
    use super::test_utils::check_or_save_reference_ir;
    use crate::generation::interop::{
        BinaryKind, BinaryOp, Call, ClassicalRegister, IfResult, Instruction, IntPredicate,
        Measured, QuantumRegister, SemanticModel, Single, Type, Value, Variable,
    };

    #[test]
    fn test_empty_if() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_empty_if".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![],
                    if_zero: vec![],
                }),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                    if_zero: vec![],
                }),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![],
                    if_zero: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                }),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_continue() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_continue".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                    if_zero: vec![],
                }),
                Instruction::H(Single::new(Value::Qubit("q0".to_string()))),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else_continue() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else_continue".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![],
                    if_zero: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                }),
                Instruction::H(Single::new(Value::Qubit("q0".to_string()))),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_else_continue() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_else_continue".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                    if_zero: vec![Instruction::Y(Single::new(Value::Qubit("q0".to_string())))],
                }),
                Instruction::H(Single::new(Value::Qubit("q0".to_string()))),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_then() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_then".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 2)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r1".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result("r1".to_string()),
                        if_one: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                        if_zero: vec![],
                    })],
                    if_zero: vec![],
                }),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else_else() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else_else".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 2)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r1".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![],
                    if_zero: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result("r1".to_string()),
                        if_one: vec![],
                        if_zero: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                    })],
                }),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_then_else() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then_else".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 2)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r1".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result("r1".to_string()),
                        if_one: vec![],
                        if_zero: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                    })],
                    if_zero: vec![],
                }),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_if_else_then() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_else_then".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 2)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r0".to_string()),
                )),
                Instruction::M(Measured::new(
                    Value::Qubit("q0".to_string()),
                    Value::Result("r1".to_string()),
                )),
                Instruction::IfResult(IfResult {
                    cond: Value::Result("r0".to_string()),
                    if_one: vec![],
                    if_zero: vec![Instruction::IfResult(IfResult {
                        cond: Value::Result("r1".to_string()),
                        if_one: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                        if_zero: vec![],
                    })],
                }),
            ],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_allows_unmeasured_result_condition() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_allows_unmeasured_result_condition".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::IfResult(IfResult {
                cond: Value::Result("r0".to_string()),
                if_one: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                if_zero: vec![Instruction::H(Single::new(Value::Qubit("q0".to_string())))],
            })],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    #[should_panic(expected = "Result r1 not found.")]
    fn test_panics_if_result_not_found() {
        let model = SemanticModel {
            name: "test_panics_if_result_not_found".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::IfResult(IfResult {
                cond: Value::Result("r1".to_string()),
                if_one: vec![Instruction::X(Single::new(Value::Qubit("q0".to_string())))],
                if_zero: vec![Instruction::H(Single::new(Value::Qubit("q0".to_string())))],
            })],
            external_functions: vec![],
        };

        check_or_save_reference_ir(&model).unwrap();
    }

    #[test]
    fn test_call_variable() -> Result<(), String> {
        let i64 = Type::Int { width: 64 };
        let x = Variable::new();

        check_or_save_reference_ir(&SemanticModel {
            name: "test_call_variable".to_string(),
            registers: vec![],
            qubits: vec![],
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
            registers: vec![],
            qubits: vec![],
            instructions,
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
        })
    }
}

#[cfg(test)]
mod test_utils {
    use crate::generation::{emit, interop::SemanticModel};
    use normalize_line_endings::normalized;
    use std::{env, fs, path::PathBuf};

    const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";

    pub(crate) fn check_or_save_reference_ir(model: &SemanticModel) -> Result<(), String> {
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

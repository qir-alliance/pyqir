// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{interop::SemanticModel, qir};
use inkwell::{
    attributes::AttributeLoc,
    context::Context,
    values::{BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use qirlib::{codegen::CodeGenerator, passes::run_basic_passes_on};
use std::collections::HashMap;

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub(crate) fn ir(model: &SemanticModel) -> Result<String, String> {
    let ctx = Context::create();
    let generator = populate_context(&ctx, model)?;
    run_basic_passes_on(&generator.module);
    Ok(generator.get_ir())
}

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub(crate) fn bitcode(model: &SemanticModel) -> Result<Vec<u8>, String> {
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

fn build_entry_function(
    generator: &CodeGenerator<'_>,
    model: &SemanticModel,
) -> Result<(), String> {
    let entry_point = qir::create_entry_point(generator.context, &generator.module);

    if model.static_alloc {
        let num_qubits = format!("{}", model.qubits.len());
        let required_qubits = generator
            .context
            .create_string_attribute("requiredQubits", &num_qubits);
        entry_point.add_attribute(AttributeLoc::Function, required_qubits);
    }

    let entry = generator.context.append_basic_block(entry_point, "entry");
    generator.builder.position_at_end(entry);

    let qubits = write_qubits(model, generator);

    let mut registers = write_registers(model);

    write_instructions(model, generator, &qubits, &mut registers, entry_point);

    if !model.static_alloc {
        free_qubits(generator, &qubits);
    }

    generator.builder.build_return(None);

    generator.module.verify().map_err(|e| e.to_string())
}

fn free_qubits<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
) {
    for (_, value) in qubits.iter() {
        generator.emit_release_qubit(value);
    }
}

fn write_qubits<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, BasicValueEnum<'ctx>> {
    if model.static_alloc {
        let mut qubits: HashMap<String, BasicValueEnum<'ctx>> = HashMap::new();
        for (id, qubit) in model.qubits.iter().enumerate() {
            let indexed_name = format!("{}{}", &qubit.name[..], qubit.index);
            let int_value = generator.usize_to_i64(id).into_int_value();
            let qubit_ptr_type = generator.qubit_type().ptr_type(AddressSpace::Generic);

            let intptr =
                generator
                    .builder
                    .build_int_to_ptr(int_value, qubit_ptr_type, &indexed_name);
            qubits.insert(indexed_name, intptr.into());
        }
        qubits
    } else {
        let qubits = model
            .qubits
            .iter()
            .map(|reg| {
                let indexed_name = format!("{}{}", &reg.name[..], reg.index);
                let value = generator.emit_allocate_qubit(indexed_name.as_str());
                (indexed_name, value)
            })
            .collect();

        qubits
    }
}

fn write_registers<'ctx>(model: &SemanticModel) -> HashMap<String, Option<PointerValue<'ctx>>> {
    let mut registers = HashMap::new();
    let number_of_registers = model.registers.len() as u64;
    if number_of_registers > 0 {
        for register in &model.registers {
            for index in 0..register.size {
                let name = format!("{}{}", register.name, index);
                registers.insert(name, None);
            }
        }
    }
    registers
}

fn write_instructions<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    entry_point: FunctionValue,
) {
    for inst in &model.instructions {
        qir::instructions::emit(generator, inst, qubits, registers, entry_point);
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
mod tests {
    use crate::{
        emit,
        interop::{
            ClassicalRegister, If, Instruction, Measured, QuantumRegister, SemanticModel, Single,
        },
    };
    use normalize_line_endings::normalized;
    use std::{env, fs, path::PathBuf};

    const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";

    #[test]
    fn test_if_then() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_if_then".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                    else_insts: vec![],
                }),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![],
                    else_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                }),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                    else_insts: vec![],
                }),
                Instruction::H(Single::new("q0".to_string())),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![],
                    else_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                }),
                Instruction::H(Single::new("q0".to_string())),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                    else_insts: vec![Instruction::Y(Single::new("q0".to_string()))],
                }),
                Instruction::H(Single::new("q0".to_string())),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::M(Measured::new("q0".to_string(), "r1".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![Instruction::If(If {
                        condition: "r1".to_string(),
                        then_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                        else_insts: vec![],
                    })],
                    else_insts: vec![],
                }),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::M(Measured::new("q0".to_string(), "r1".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![],
                    else_insts: vec![Instruction::If(If {
                        condition: "r1".to_string(),
                        then_insts: vec![],
                        else_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                    })],
                }),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::M(Measured::new("q0".to_string(), "r1".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![Instruction::If(If {
                        condition: "r1".to_string(),
                        then_insts: vec![],
                        else_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                    })],
                    else_insts: vec![],
                }),
            ],
            static_alloc: true,
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
                Instruction::M(Measured::new("q0".to_string(), "r0".to_string())),
                Instruction::M(Measured::new("q0".to_string(), "r1".to_string())),
                Instruction::If(If {
                    condition: "r0".to_string(),
                    then_insts: vec![],
                    else_insts: vec![Instruction::If(If {
                        condition: "r1".to_string(),
                        then_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                        else_insts: vec![],
                    })],
                }),
            ],
            static_alloc: true,
        };

        check_or_save_reference_ir(&model)
    }

    #[test]
    fn test_results_default_to_zero_if_not_measured() -> Result<(), String> {
        let model = SemanticModel {
            name: "test_results_default_to_zero_if_not_measured".to_string(),
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::If(If {
                condition: "r0".to_string(),
                then_insts: vec![Instruction::X(Single::new("q0".to_string()))],
                else_insts: vec![Instruction::H(Single::new("q0".to_string()))],
            })],
            static_alloc: true,
        };

        check_or_save_reference_ir(&model)
    }

    fn check_or_save_reference_ir(model: &SemanticModel) -> Result<(), String> {
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

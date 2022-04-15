// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::CodeGenerator,
    generation::{
        interop::{self, ReturnType, SemanticModel, ValueType},
        qir,
    },
    passes::run_basic_passes_on,
};
use inkwell::{
    attributes::AttributeLoc,
    context::Context,
    module::Linkage,
    types::{BasicType, BasicTypeEnum, FunctionType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use std::{collections::HashMap, convert::Into};

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
    let generator = CodeGenerator::new(
        ctx,
        module,
        model.use_static_qubit_alloc,
        model.use_static_result_alloc,
    )?;
    build_entry_function(&generator, model)?;
    Ok(generator)
}

fn build_entry_function(generator: &CodeGenerator, model: &SemanticModel) -> Result<(), String> {
    add_external_functions(generator, model.external_functions.iter());
    let entry_point = qir::create_entry_point(generator.context, &generator.module);

    if generator.use_static_qubit_alloc {
        let num_qubits = format!("{}", model.qubits.len());
        let required_qubits = generator
            .context
            .create_string_attribute("requiredQubits", &num_qubits);
        entry_point.add_attribute(AttributeLoc::Function, required_qubits);
    }

    if generator.use_static_result_alloc {
        let num_results: u64 = model.registers.iter().map(|f| f.size).sum();
        let num_result_string = format!("{}", num_results);
        let required_results = generator
            .context
            .create_string_attribute("requiredResults", &num_result_string);
        entry_point.add_attribute(AttributeLoc::Function, required_results);
    }

    let entry = generator.context.append_basic_block(entry_point, "entry");
    generator.builder.position_at_end(entry);

    let qubits = write_qubits(model, generator);
    let mut registers = write_registers(model, generator);
    write_instructions(model, generator, &qubits, &mut registers, entry_point);

    if !model.use_static_qubit_alloc {
        free_qubits(generator, &qubits);
    }

    generator.builder.build_return(None);
    generator.module.verify().map_err(|e| e.to_string())
}

fn add_external_functions<'a>(
    generator: &CodeGenerator,
    functions: impl Iterator<Item = (&'a String, &'a interop::FunctionType)>,
) {
    for (name, ty) in functions {
        let ty = get_function_type(generator, ty);
        generator
            .module
            .add_function(name, ty, Some(Linkage::External));
    }
}

fn get_function_type<'ctx>(
    generator: &CodeGenerator<'ctx>,
    ty: &interop::FunctionType,
) -> FunctionType<'ctx> {
    let param_types: Vec<_> = ty
        .param_types
        .iter()
        .map(|ty| get_basic_type(generator, ty).into())
        .collect();

    let param_types = param_types.as_slice();
    match ty.return_type {
        ReturnType::Void => generator.context.void_type().fn_type(param_types, false),
        ReturnType::Value(ty) => get_basic_type(generator, &ty).fn_type(param_types, false),
    }
}

fn get_basic_type<'ctx>(generator: &CodeGenerator<'ctx>, ty: &ValueType) -> BasicTypeEnum<'ctx> {
    match ty {
        ValueType::Integer { width } => {
            BasicTypeEnum::IntType(generator.context.custom_width_int_type(*width))
        }
        ValueType::Double => BasicTypeEnum::FloatType(generator.context.f64_type()),
        ValueType::Qubit => {
            BasicTypeEnum::PointerType(generator.qubit_type().ptr_type(AddressSpace::Generic))
        }
        ValueType::Result => {
            BasicTypeEnum::PointerType(generator.result_type().ptr_type(AddressSpace::Generic))
        }
    }
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
    if generator.use_static_qubit_alloc {
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

fn write_registers<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, Option<PointerValue<'ctx>>> {
    if generator.use_static_result_alloc {
        let mut registers: HashMap<String, Option<PointerValue<'ctx>>> = HashMap::new();
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
    } else {
        let mut registers: HashMap<String, Option<PointerValue<'ctx>>> = HashMap::new();
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
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    entry_point: FunctionValue,
) {
    for inst in &model.instructions {
        qir::instructions::emit(generator, inst, qubits, registers, entry_point);
    }
}

#[cfg(test)]
mod result_alloc_tests {
    use crate::generation::{
        emit,
        interop::{
            ClassicalRegister, Instruction, Measured, QuantumRegister, SemanticModel, Single,
        },
    };
    use std::collections::HashMap;

    fn get_model(
        name: String,
        use_static_qubit_alloc: bool,
        use_static_result_alloc: bool,
    ) -> SemanticModel {
        SemanticModel {
            name,
            registers: vec![ClassicalRegister::new("r".to_string(), 1)],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::M(Measured::new(
                "q0".to_string(),
                "r0".to_string(),
            ))],
            use_static_qubit_alloc,
            use_static_result_alloc,
            external_functions: HashMap::new(),
        }
    }

    #[test]
    fn when_dynamic_qubit_and_dynamic_result_alloc_is_used_then_only_entypoint_attribute_is_emitted(
    ) -> Result<(), String> {
        let model = get_model("test".to_owned(), false, false);
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" }"));
        Ok(())
    }

    #[test]
    fn when_static_qubit_alloc_is_used_then_required_attribute_is_emitted() -> Result<(), String> {
        let model = get_model("test".to_owned(), true, false);
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" \"requiredQubits\"=\"1\" }"));
        Ok(())
    }

    #[test]
    fn when_static_result_alloc_is_used_then_required_attribute_is_emitted() -> Result<(), String> {
        let model = get_model("test".to_owned(), false, true);
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" \"requiredResults\"=\"1\" }"));
        Ok(())
    }

    #[test]
    fn when_static_qubit_and_static_result_alloc_is_used_then_both_required_attribute_are_emitted(
    ) -> Result<(), String> {
        let model = get_model("test".to_owned(), true, true);
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains(
            "attributes #0 = { \"EntryPoint\" \"requiredQubits\"=\"1\" \"requiredResults\"=\"1\" }"
        ));
        Ok(())
    }

    #[test]
    fn when_static_result_alloc_is_used_then_emitted_attribute_sums_registers_correctly() -> Result<(), String>
    {
        let model = SemanticModel {
            name: "test".to_owned(),
            registers: vec![
                ClassicalRegister::new("r".to_string(), 1),
                ClassicalRegister::new("r".to_string(), 3),
                ClassicalRegister::new("r".to_string(), 4),
            ],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::M(Measured::new(
                "q0".to_string(),
                "r0".to_string(),
            ))],
            use_static_qubit_alloc: false,
            use_static_result_alloc: true,
            external_functions: HashMap::new(),
        };
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" \"requiredResults\"=\"8\" }"));
        Ok(())
    }

    #[test]
    fn when_static_result_alloc_is_used_and_no_registers_declared_then_emitted_attribute_sums_correctly(
    ) -> Result<(), String> {
        let model = SemanticModel {
            name: "test".to_owned(),
            registers: vec![],
            qubits: vec![QuantumRegister::new("q".to_string(), 0)],
            instructions: vec![Instruction::H(Single::new("q0".to_string()))],
            use_static_qubit_alloc: false,
            use_static_result_alloc: true,
            external_functions: HashMap::new(),
        };
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("attributes #0 = { \"EntryPoint\" \"requiredResults\"=\"0\" }"));
        Ok(())
    }

    #[test]
    fn when_dynamic_result_alloc_is_used_then_m_body_is_emitted() -> Result<(), String> {
        let model = get_model("test".to_owned(), false, false);
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("declare %Result* @__quantum__qis__m__body(%Qubit*)"));
        Ok(())
    }

    #[test]
    fn when_static_result_alloc_is_used_then_mz_body_is_emitted() -> Result<(), String> {
        let model = get_model("test".to_owned(), false, true);
        let actual_ir: String = emit::ir(&model)?;
        assert!(actual_ir.contains("declare void @__quantum__qis__mz__body(%Qubit*, %Result*)"));
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
mod if_tests {
    use crate::generation::{
        emit,
        interop::{
            ClassicalRegister, If, Instruction, Measured, QuantumRegister, SemanticModel, Single,
        },
    };
    use normalize_line_endings::normalized;
    use std::{collections::HashMap, env, fs, path::PathBuf};

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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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
            use_static_qubit_alloc: true,
            use_static_result_alloc: false,
            external_functions: HashMap::new(),
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

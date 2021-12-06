// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{interop::SemanticModel, qir};
use inkwell::values::BasicValueEnum;
use qirlib::passes::run_basic_passes_on;
use qirlib::{codegen::CodeGenerator, module};
use std::collections::HashMap;

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn write_model_to_file(model: &SemanticModel, file_name: &str) -> Result<(), String> {
    let ctx = inkwell::context::Context::create();
    let generator = populate_context(&ctx, model)?;
    run_basic_passes_on(&generator.module);
    generator.emit_ir(file_name)?;

    Ok(())
}

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn get_ir_string(model: &SemanticModel) -> Result<String, String> {
    let ctx = inkwell::context::Context::create();
    let generator = populate_context(&ctx, model)?;
    run_basic_passes_on(&generator.module);
    let ir = generator.get_ir_string();

    Ok(ir)
}

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn get_bitcode_base64_string(model: &SemanticModel) -> Result<String, String> {
    let ctx = inkwell::context::Context::create();
    let generator = populate_context(&ctx, model)?;
    run_basic_passes_on(&generator.module);

    let b64 = generator.get_bitcode_base64_string();

    Ok(b64)
}

/// # Errors
///
/// Will return `Err` if
///  - module cannot be loaded.
///  - module fails verification that the current `Module` is valid.
pub fn populate_context<'a>(
    ctx: &'a inkwell::context::Context,
    model: &'a SemanticModel,
) -> Result<CodeGenerator<'a>, String> {
    let module = module::load_template(&model.name, ctx)?;
    let generator = CodeGenerator::new(ctx, module)?;
    build_entry_function(&generator, model)?;
    Ok(generator)
}

fn build_entry_function(
    generator: &CodeGenerator<'_>,
    model: &SemanticModel,
) -> Result<(), String> {
    let entrypoint = qir::get_entry_function(&generator.module);

    let entry = generator.context.append_basic_block(entrypoint, "entry");
    generator.builder.position_at_end(entry);

    let qubits = write_qubits(model, generator);

    let registers = write_registers(model, generator);

    write_instructions(model, generator, &qubits, &registers);

    free_qubits(generator, &qubits);

    let output = registers.get("results").unwrap();
    generator.builder.build_return(Some(&output.0));

    generator.module.verify().map_err(|e| e.to_string())
}

fn free_qubits<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
) {
    for (_, value) in qubits.iter() {
        qir::qubits::emit_release(generator, value);
    }
}

fn write_qubits<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, BasicValueEnum<'ctx>> {
    let qubits = model
        .qubits
        .iter()
        .map(|reg| {
            let indexed_name = format!("{}{}", &reg.name[..], reg.index);
            let value = qir::qubits::emit_allocate(generator, indexed_name.as_str());
            (indexed_name, value)
        })
        .collect();

    qubits
}

fn write_registers<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)> {
    let mut registers = HashMap::new();
    let number_of_registers = model.registers.len() as u64;
    if number_of_registers > 0 {
        let results =
            qir::array1d::emit_array_allocate1d(generator, 8, number_of_registers, "results");
        registers.insert(String::from("results"), (results, None));
        let mut sub_results = vec![];
        for reg in &model.registers {
            let (sub_result, entries) =
                qir::array1d::emit_array_1d(generator, reg.name.as_str(), reg.size);
            sub_results.push(sub_result);
            registers.insert(reg.name.clone(), (sub_result, None));
            for (index, _) in entries {
                registers.insert(format!("{}{}", reg.name, index), (sub_result, Some(index)));
            }
        }
        qir::array1d::set_elements(generator, &results, &sub_results, "results");
    } else {
        let results = qir::array1d::emit_empty_result_array_allocate1d(generator, "results");
        registers.insert(String::from("results"), (results, None));
    }
    registers
}

fn write_instructions<'ctx>(
    model: &SemanticModel,
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    for inst in &model.instructions {
        qir::instructions::emit(generator, inst, qubits, registers);
    }
}

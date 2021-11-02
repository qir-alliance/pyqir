// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use inkwell::values::BasicValueEnum;
use qirlib::context::{Context, ContextType};

use crate::{interop::SemanticModel, qir};

pub fn write_model_to_file(model: &SemanticModel, file_name: &str) -> Result<(), String> {
    let ctx = inkwell::context::Context::create();
    let context = populate_context(&ctx, &model)?;

    context.emit_ir(file_name)?;

    Ok(())
}

pub fn get_ir_string(model: &SemanticModel) -> Result<String, String> {
    let ctx = inkwell::context::Context::create();
    let context = populate_context(&ctx, &model)?;

    let ir = context.get_ir_string();

    Ok(ir)
}

pub fn get_bitcode_base64_string(model: &SemanticModel) -> Result<String, String> {
    let ctx = inkwell::context::Context::create();
    let context = populate_context(&ctx, &model)?;

    let b64 = context.get_bitcode_base64_string();

    Ok(b64)
}


pub fn populate_context<'a>(
    ctx: &'a inkwell::context::Context,
    model: &'a SemanticModel,
) -> Result<Context<'a>, String> {
    let context_type = ContextType::Template(&model.name);
    match Context::new(&ctx, context_type) {
        Err(err) => {
            let message = err.to_string();
            return Err(message);
        }
        Ok(context) => {
            build_entry_function(&context, model)?;
            Ok(context)
        }
    }
}

fn build_entry_function(context: &Context<'_>, model: &SemanticModel) -> Result<(), String> {
    let entrypoint = qir::get_entry_function(context);

    let entry = context.context.append_basic_block(entrypoint, "entry");
    context.builder.position_at_end(entry);

    let qubits = write_qubits(&model, context);

    let registers = write_registers(&model, context);

    write_instructions(&model, context, &qubits, &registers);

    free_qubits(context, &qubits);

    let output = registers.get("results").unwrap();
    context.builder.build_return(Some(&output.0));

    if let Err(err) = context.module.verify() {
        let message = err.to_string();
        return Err(message);
    }
    Ok(())
}

fn free_qubits<'ctx>(context: &Context<'ctx>, qubits: &HashMap<String, BasicValueEnum<'ctx>>) {
    for (_, value) in qubits.iter() {
        qir::qubits::emit_release(context, value);
    }
}

fn write_qubits<'ctx>(
    model: &SemanticModel,
    context: &Context<'ctx>,
) -> HashMap<String, BasicValueEnum<'ctx>> {
    let qubits = model
        .qubits
        .iter()
        .map(|reg| {
            let indexed_name = format!("{}{}", &reg.name[..], reg.index);
            let value = qir::qubits::emit_allocate(&context, indexed_name.as_str());
            (indexed_name, value)
        })
        .collect();

    qubits
}

fn write_registers<'ctx>(
    model: &SemanticModel,
    context: &Context<'ctx>,
) -> HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)> {
    let mut registers = HashMap::new();
    let number_of_registers = model.registers.len() as u64;
    if number_of_registers > 0 {
        let results =
            qir::array1d::emit_array_allocate1d(&context, 8, number_of_registers, "results");
        registers.insert(String::from("results"), (results, None));
        let mut sub_results = vec![];
        for reg in model.registers.iter() {
            let (sub_result, entries) =
                qir::array1d::emit_array_1d(context, reg.name.as_str(), reg.size.clone());
            sub_results.push(sub_result);
            registers.insert(reg.name.clone(), (sub_result, None));
            for (index, _) in entries {
                registers.insert(format!("{}{}", reg.name, index), (sub_result, Some(index)));
            }
        }
        qir::array1d::set_elements(&context, &results, sub_results, "results");
        registers
    } else {
        let results = qir::array1d::emit_empty_result_array_allocate1d(&context, "results");
        registers.insert(String::from("results"), (results, None));
        registers
    }
}

fn write_instructions<'ctx>(
    model: &SemanticModel,
    context: &Context<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    for inst in model.instructions.iter() {
        qir::instructions::emit(context, inst, qubits, registers);
    }
}

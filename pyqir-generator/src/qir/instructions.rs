// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::result;
use crate::interop::{If, Instruction};
use inkwell::{
    module::{Linkage, Module},
    types::BasicMetadataTypeEnum,
    values::{AnyValue, BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use qirlib::codegen::CodeGenerator;
use std::collections::HashMap;

/// # Panics
///
/// Panics if the qubit name doesn't exist
fn get_qubit<'ctx>(
    name: &str,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
) -> BasicValueEnum<'ctx> {
    *qubits
        .get(name)
        .unwrap_or_else(|| panic!("Qubit {} not found.", name))
}

fn measure<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubit: &str,
    target: &str,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
) {
    let find_qubit = |name| get_qubit(name, qubits);

    // measure the qubit and save the result to a temporary value
    let new_value = generator.emit_call_with_return(
        generator.qis_m_body(),
        &[find_qubit(qubit).into()],
        target,
    );

    registers.insert(target.to_owned(), Some(new_value.into_pointer_value()));
}

fn controlled<'ctx>(
    generator: &CodeGenerator<'ctx>,
    intrinsic: FunctionValue<'ctx>,
    control: BasicValueEnum<'ctx>,
    qubit: BasicValueEnum<'ctx>,
) {
    generator.emit_void_call(intrinsic, &[control.into(), qubit.into()]);
}

pub(crate) fn emit<'ctx>(
    generator: &CodeGenerator<'ctx>,
    inst: &Instruction,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    entry_point: FunctionValue,
) {
    let find_qubit = |name| get_qubit(name, qubits);
    match inst {
        Instruction::Cx(inst) => {
            let control = find_qubit(&inst.control);
            let qubit = find_qubit(&inst.target);
            controlled(generator, generator.qis_cnot_body(), control, qubit);
        }
        Instruction::Cz(inst) => {
            let control = find_qubit(&inst.control);
            let qubit = find_qubit(&inst.target);
            controlled(generator, generator.qis_cz_body(), control, qubit);
        }
        Instruction::H(inst) => {
            generator.emit_void_call(generator.qis_h_body(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::M(inst) => {
            measure(generator, &inst.qubit, &inst.target, qubits, registers);
        }
        Instruction::Reset(inst) => {
            generator.emit_void_call(
                generator.qis_reset_body(),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::Rx(inst) => {
            generator.emit_void_call(
                generator.qis_rx_body(),
                &[
                    generator.f64_to_f64(inst.theta),
                    find_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::Ry(inst) => {
            generator.emit_void_call(
                generator.qis_ry_body(),
                &[
                    generator.f64_to_f64(inst.theta),
                    find_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::Rz(inst) => {
            generator.emit_void_call(
                generator.qis_rz_body(),
                &[
                    generator.f64_to_f64(inst.theta),
                    find_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::S(inst) => {
            generator.emit_void_call(generator.qis_s_body(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::SAdj(inst) => {
            generator.emit_void_call(generator.qis_s_adj(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::T(inst) => {
            generator.emit_void_call(generator.qis_t_body(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::TAdj(inst) => {
            generator.emit_void_call(generator.qis_t_adj(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::X(inst) => {
            generator.emit_void_call(generator.qis_x_body(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::Y(inst) => {
            generator.emit_void_call(generator.qis_y_body(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::Z(inst) => {
            generator.emit_void_call(generator.qis_z_body(), &[find_qubit(&inst.qubit).into()]);
        }
        Instruction::If(if_inst) => emit_if(generator, registers, qubits, entry_point, if_inst),
        Instruction::Call(call) => {
            let types: Vec<BasicMetadataTypeEnum<'ctx>> = call
                .args
                .iter()
                .map(|args| type_name_to_basicmetadatatypeenum(generator, args.type_name.as_str()))
                .collect();
            let args: Vec<BasicMetadataValueEnum<'ctx>> = call
                .args
                .iter()
                .map(|args| match args.type_name.as_str() {
                    "Qubit" => find_qubit(&args.value).into(),
                    "Result" => {
                        if let Some(register) = registers[&args.value] {
                            register.into()
                        } else {
                            panic!("Register {} must be measured prior to use.", args.value);
                        }
                    }
                    "f64" => generator.f64_to_f64(args.value.parse::<f64>().unwrap()),
                    "i64" => generator.i64_to_i64(args.value.parse::<i64>().unwrap()),
                    "i8" => generator.bool_to_i1(args.value.parse::<bool>().unwrap()),
                    unknown => panic!("Unknown parameter type for extern declaration {}", unknown),
                })
                .collect();
            let function = get_extern_function_declaration(
                generator.context,
                &generator.module,
                &call.name,
                &types[..],
            );
            generator.emit_void_call(function, &args[..]);
        }
    }
}

fn emit_if<'ctx>(
    generator: &CodeGenerator<'ctx>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    entry_point: FunctionValue,
    if_inst: &If,
) {
    // Panic if an undeclared result name is referenced, and default to zero if the result has been
    // declared but not yet measured.
    let result = registers
        .get(&if_inst.condition)
        .unwrap_or_else(|| panic!("Result {} not found.", &if_inst.condition))
        .unwrap_or_else(|| result::get_zero(generator));

    let condition = result::equal(generator, result, result::get_one(generator));
    let then_block = generator.context.append_basic_block(entry_point, "then");
    let else_block = generator.context.append_basic_block(entry_point, "else");
    generator
        .builder
        .build_conditional_branch(condition, then_block, else_block);

    let continue_block = generator
        .context
        .append_basic_block(entry_point, "continue");

    let mut emit_block = |block, insts| {
        generator.builder.position_at_end(block);
        for inst in insts {
            emit(generator, inst, qubits, registers, entry_point);
        }

        generator.builder.build_unconditional_branch(continue_block);
    };

    emit_block(then_block, &if_inst.then_insts);
    emit_block(else_block, &if_inst.else_insts);
    generator.builder.position_at_end(continue_block);
}

fn get_extern_function_declaration<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    function_name: &str,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionValue<'ctx> {
    if let Some(function) = get_function(module, function_name) {
        function
    } else {
        let void_type = context.void_type();
        let fn_type = void_type.fn_type(param_types, false);
        let function = module.add_function(function_name, fn_type, Some(Linkage::External));
        log::debug!(
            "{}-> has been defined",
            function.print_to_string().to_string()
        );
        function
    }
}

pub(crate) fn get_function<'ctx>(
    module: &Module<'ctx>,
    function_name: &str,
) -> Option<FunctionValue<'ctx>> {
    let defined_function = module.get_function(function_name);
    match defined_function {
        None => {
            log::debug!(
                "{} global function was not defined in the module",
                function_name
            );
            None
        }
        Some(value) => Some(value),
    }
}

fn type_name_to_basicmetadatatypeenum<'ctx>(
    generator: &CodeGenerator<'ctx>,
    type_name: &str,
) -> BasicMetadataTypeEnum<'ctx> {
    match type_name {
        "Qubit" => generator
            .qubit_type()
            .ptr_type(AddressSpace::Generic)
            .into(),
        "Result" => generator
            .result_type()
            .ptr_type(AddressSpace::Generic)
            .into(),
        "f64" => generator.double_type().into(),
        "i64" => generator.int64_type().into(),
        "i8" => generator.bool_type().into(),
        unknown => panic!("Unknown parameter type for extern declaration {}", unknown),
    }
}

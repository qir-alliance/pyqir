// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    interop::{If, Instruction},
    qir::{array1d, basic_values, calls},
};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use qirlib::codegen::CodeGenerator;
use std::collections::HashMap;

/// # Panics
///
/// Panics if the qubit name doesn't exist
fn get_qubit<'ctx>(
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    *qubits
        .get(name)
        .unwrap_or_else(|| panic!("Qubit {} not found.", name))
}

/// # Panics
///
/// Panics if the register name doesn't exist
fn get_register<'ctx>(
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
    name: &str,
) -> (BasicValueEnum<'ctx>, Option<u64>) {
    registers
        .get(name)
        .unwrap_or_else(|| panic!("Register {} not found.", name))
        .to_owned()
}

fn get_register_result<'a>(
    generator: &CodeGenerator<'a>,
    registers: &HashMap<String, (BasicValueEnum<'a>, Option<u64>)>,
    name: &str,
) -> PointerValue<'a> {
    let (register, index) = get_register(registers, name);
    let element =
        array1d::get_bitcast_result_pointer_array_element(generator, index.unwrap(), &register, "")
            .into_pointer_value();

    generator
        .builder
        .build_load(element, "")
        .into_pointer_value()
}

fn measure<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubit: &str,
    target: &str,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    let find_qubit = |name| get_qubit(qubits, name);
    let find_register = |name| get_register(registers, name);

    // measure the qubit and save the result to a temporary value
    let result = calls::emit_call_with_return(
        &generator.builder,
        generator
            .intrinsics
            .m
            .expect("m must be defined in the template"),
        &[find_qubit(qubit).into()],
        "measurement",
    );

    // find the parent register and offset for the given target
    let (register, index) = find_register(target);

    // get the bitcast pointer to the target location
    let bitcast_indexed_target_register = array1d::get_bitcast_result_pointer_array_element(
        generator,
        index.unwrap(),
        &register,
        target,
    );

    // get the existing value from that location and decrement its ref count as its
    // being replaced with the measurement.
    let existing_value = generator.builder.build_load(
        bitcast_indexed_target_register.into_pointer_value(),
        "existing_value",
    );
    let minus_one = basic_values::i64_to_i32(generator, -1);
    generator.builder.build_call(
        generator.runtime_library.result_update_reference_count,
        &[existing_value.into(), minus_one],
        "",
    );

    // increase the ref count of the new value and store it in the target register
    let one = basic_values::i64_to_i32(generator, 1);
    generator.builder.build_call(
        generator.runtime_library.result_update_reference_count,
        &[result.into(), one],
        "",
    );
    let _ = generator
        .builder
        .build_store(bitcast_indexed_target_register.into_pointer_value(), result);
}

fn controlled<'ctx>(
    generator: &CodeGenerator<'ctx>,
    intrinsic: FunctionValue<'ctx>,
    control: BasicValueEnum<'ctx>,
    qubit: BasicValueEnum<'ctx>,
) {
    calls::emit_void_call(
        &generator.builder,
        intrinsic,
        &[control.into(), qubit.into()],
    );

    let minus_one = basic_values::i64_to_i32(generator, -1);
    generator.builder.build_call(
        generator.runtime_library.array_update_reference_count,
        &[control.into(), minus_one],
        "",
    );
}

#[allow(clippy::too_many_lines)]
pub(crate) fn emit<'ctx>(
    generator: &CodeGenerator<'ctx>,
    inst: &Instruction,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    let intrinsics = &generator.intrinsics;
    let find_qubit = |name| get_qubit(qubits, name);
    let ctl = |value| array1d::create_ctl_wrapper(generator, value);

    match inst {
        Instruction::Cx(inst) => {
            let control = ctl(&find_qubit(&inst.control));
            let qubit = find_qubit(&inst.target);
            controlled(
                generator,
                intrinsics
                    .x_ctl
                    .expect("x_ctl must be defined in the template"),
                control,
                qubit,
            );
        }
        Instruction::Cz(inst) => {
            let control = ctl(&find_qubit(&inst.control));
            let qubit = find_qubit(&inst.target);
            controlled(
                generator,
                intrinsics
                    .z_ctl
                    .expect("z_ctl must be defined in the template"),
                control,
                qubit,
            );
        }
        Instruction::H(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.h.expect("h must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::M(inst) => measure(generator, &inst.qubit, &inst.target, qubits, registers),
        Instruction::Reset(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics
                .reset
                .expect("reset must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::Rx(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.r_x.expect("r_x must be defined in the template"),
            &[
                basic_values::f64_to_f64(generator, inst.theta),
                find_qubit(&inst.qubit).into(),
            ],
        ),
        Instruction::Ry(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.r_y.expect("r_y must be defined in the template"),
            &[
                basic_values::f64_to_f64(generator, inst.theta),
                find_qubit(&inst.qubit).into(),
            ],
        ),
        Instruction::Rz(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.r_z.expect("r_z must be defined in the template"),
            &[
                basic_values::f64_to_f64(generator, inst.theta),
                find_qubit(&inst.qubit).into(),
            ],
        ),
        Instruction::S(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.s.expect("s must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::SAdj(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics
                .s_adj
                .expect("s_adj must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::T(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.t.expect("t must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::TAdj(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics
                .t_adj
                .expect("t_adj must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::X(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.x.expect("x must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::Y(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.y.expect("y must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::Z(inst) => calls::emit_void_call(
            &generator.builder,
            intrinsics.z.expect("z must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::DumpMachine => calls::emit_void_call(
            &generator.builder,
            intrinsics
                .dumpmachine
                .expect("dumpmachine must be defined before use"),
            &[basic_values::i8_null_ptr(generator)],
        ),
        Instruction::If(if_inst) => emit_if(generator, registers, qubits, if_inst),
    }
}

fn emit_if<'a>(
    generator: &CodeGenerator<'a>,
    registers: &HashMap<String, (BasicValueEnum<'a>, Option<u64>)>,
    qubits: &HashMap<String, BasicValueEnum<'a>>,
    if_inst: &If,
) {
    // TODO: Refactor this with qir::get_entry_function.
    let entry_name = "QuantumApplication__Run__body";
    let entry = generator.module.get_function(entry_name).unwrap();

    // The reference count doesn't need to be updated because the result is only used for this
    // condition and won't outlive the array.
    let result = get_register_result(generator, registers, &if_inst.condition);
    let condition = result_equals(generator, result, get_result_one(generator));

    let then_block = generator.context.append_basic_block(entry, "then");
    let else_block = generator.context.append_basic_block(entry, "else");
    generator
        .builder
        .build_conditional_branch(condition, then_block, else_block);

    let continue_block = generator.context.append_basic_block(entry, "continue");
    let emit_block = |block, insts| {
        generator.builder.position_at_end(block);
        for inst in insts {
            emit(generator, inst, qubits, registers);
        }
        generator.builder.build_unconditional_branch(continue_block);
    };

    emit_block(then_block, &if_inst.true_insts);
    emit_block(else_block, &if_inst.false_insts);
    generator.builder.position_at_end(continue_block);
}

fn get_result_one<'a>(generator: &CodeGenerator<'a>) -> PointerValue<'a> {
    calls::emit_call_with_return(
        &generator.builder,
        generator.runtime_library.result_get_one,
        &[],
        "one",
    )
    .into_pointer_value()
}

fn result_equals<'a>(
    generator: &CodeGenerator<'a>,
    x: PointerValue<'a>,
    y: PointerValue<'a>,
) -> IntValue<'a> {
    calls::emit_call_with_return(
        &generator.builder,
        generator.runtime_library.result_equal,
        &[
            BasicMetadataValueEnum::PointerValue(x),
            BasicMetadataValueEnum::PointerValue(y),
        ],
        "",
    )
    .into_int_value()
}

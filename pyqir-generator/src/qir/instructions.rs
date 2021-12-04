// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::interop::Instruction;

use super::{
    array1d::{self, create_ctl_wrapper},
    basic_values, calls,
};
use inkwell::values::{BasicValueEnum, FunctionValue};
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

/// # Panics
///
/// Panics if the register name doesn't exist
fn get_register<'ctx>(
    name: &str,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) -> (BasicValueEnum<'ctx>, Option<u64>) {
    registers
        .get(name)
        .unwrap_or_else(|| panic!("Register {} not found.", name))
        .to_owned()
}

fn measure<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubit: &str,
    target: &str,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    let find_qubit = |name| get_qubit(name, qubits);
    let find_register = |name| get_register(name, registers);

    // measure the qubit and save the result to a temporary value
    let result = calls::emit_call_with_return(
        generator,
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
    calls::emit_void_call(generator, intrinsic, &[control.into(), qubit.into()]);
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
    let find_qubit = |name| get_qubit(name, qubits);
    let ctl = |value| create_ctl_wrapper(generator, value);
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
            generator,
            intrinsics.h.expect("h must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::M(inst) => {
            measure(generator, &inst.qubit, &inst.target, qubits, registers);
        }
        Instruction::Reset(inst) => calls::emit_void_call(
            generator,
            intrinsics
                .reset
                .expect("reset must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::Rx(inst) => calls::emit_void_call(
            generator,
            intrinsics.r_x.expect("r_x must be defined in the template"),
            &[
                basic_values::f64_to_f64(generator, inst.theta),
                find_qubit(&inst.qubit).into(),
            ],
        ),
        Instruction::Ry(inst) => calls::emit_void_call(
            generator,
            intrinsics.r_y.expect("r_y must be defined in the template"),
            &[
                basic_values::f64_to_f64(generator, inst.theta),
                find_qubit(&inst.qubit).into(),
            ],
        ),
        Instruction::Rz(inst) => calls::emit_void_call(
            generator,
            intrinsics.r_z.expect("r_z must be defined in the template"),
            &[
                basic_values::f64_to_f64(generator, inst.theta),
                find_qubit(&inst.qubit).into(),
            ],
        ),
        Instruction::S(inst) => calls::emit_void_call(
            generator,
            intrinsics.s.expect("s must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::SAdj(inst) => calls::emit_void_call(
            generator,
            intrinsics
                .s_adj
                .expect("s_adj must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::T(inst) => calls::emit_void_call(
            generator,
            intrinsics.t.expect("t must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::TAdj(inst) => calls::emit_void_call(
            generator,
            intrinsics
                .t_adj
                .expect("t_adj must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::X(inst) => calls::emit_void_call(
            generator,
            intrinsics.x.expect("x must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::Y(inst) => calls::emit_void_call(
            generator,
            intrinsics.y.expect("y must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::Z(inst) => calls::emit_void_call(
            generator,
            intrinsics.z.expect("z must be defined in the template"),
            &[find_qubit(&inst.qubit).into()],
        ),
        Instruction::DumpMachine => calls::emit_void_call(
            generator,
            intrinsics
                .dumpmachine
                .expect("dumpmachine must be defined before use"),
            &[basic_values::i8_null_ptr(generator)],
        ),
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::interop::Instruction;

use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use qirlib::codegen::calls::Calls;
use qirlib::codegen::ext::{BasicValues, Intrinsics};
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
        generator
            .qis_m_body()
            .expect("m must be defined in the template"),
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

#[allow(clippy::too_many_lines)]
pub(crate) fn emit<'ctx>(
    generator: &CodeGenerator<'ctx>,
    inst: &Instruction,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
) {
    let find_qubit = |name| get_qubit(name, qubits);
    match inst {
        Instruction::Cx(inst) => {
            let control = find_qubit(&inst.control);
            let qubit = find_qubit(&inst.target);
            controlled(
                generator,
                generator
                    .qis_cnot_body()
                    .expect("qis_cnot_body must be defined in the template"),
                control,
                qubit,
            );
        }
        Instruction::Cz(inst) => {
            let control = find_qubit(&inst.control);
            let qubit = find_qubit(&inst.target);
            controlled(
                generator,
                generator
                    .qis_cz_body()
                    .expect("qis_cz_body must be defined in the template"),
                control,
                qubit,
            );
        }
        Instruction::H(inst) => {
            generator.emit_void_call(
                generator
                    .qis_h_body()
                    .expect("qis_h_body must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::M(inst) => {
            measure(generator, &inst.qubit, &inst.target, qubits, registers);
        }
        Instruction::Reset(inst) => {
            generator.emit_void_call(
                generator
                    .qis_reset_body()
                    .expect("qis_reset_body must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::Rx(inst) => {
            generator.emit_void_call(
                generator
                    .qis_rx_body()
                    .expect("qis_rx_body must be defined in the template"),
                &[
                    generator.f64_to_f64(inst.theta),
                    find_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::Ry(inst) => {
            generator.emit_void_call(
                generator
                    .qis_ry_body()
                    .expect("qis_ry_body must be defined in the template"),
                &[
                    generator.f64_to_f64(inst.theta),
                    find_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::Rz(inst) => {
            generator.emit_void_call(
                generator
                    .qis_rz_body()
                    .expect("qis_rz_body must be defined in the template"),
                &[
                    generator.f64_to_f64(inst.theta),
                    find_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::S(inst) => {
            generator.emit_void_call(
                generator
                    .qis_s_body()
                    .expect("qis_s_body must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::SAdj(inst) => {
            generator.emit_void_call(
                generator
                    .qis_s_adj()
                    .expect("qis_s_adj must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::T(inst) => {
            generator.emit_void_call(
                generator
                    .qis_t_body()
                    .expect("qis_t_body must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::TAdj(inst) => {
            generator.emit_void_call(
                generator
                    .qis_t_adj()
                    .expect("qis_t_adj must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::X(inst) => {
            generator.emit_void_call(
                generator
                    .qis_x_body()
                    .expect("qis_x_body must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::Y(inst) => {
            generator.emit_void_call(
                generator
                    .qis_y_body()
                    .expect("qis_y_body must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
        Instruction::Z(inst) => {
            generator.emit_void_call(
                generator
                    .qis_z_body()
                    .expect("qis_z_body must be defined in the template"),
                &[find_qubit(&inst.qubit).into()],
            );
        }
    }
}

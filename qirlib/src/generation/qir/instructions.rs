// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::CodeGenerator,
    generation::{
        interop::{Call, If, Instruction, Value},
        qir::result,
    },
};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;

/// # Panics
///
/// Panics if the qubit name doesn't exist
fn get_qubit<'ctx>(
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    *qubits
        .get(name)
        .unwrap_or_else(|| panic!("Qubit {} not found.", name))
}

/// Gets the most recent value of a result name. Defaults to zero if the result has been declared
/// but not yet measured.
///
/// # Panics
///
/// Panics if the result name has not been declared.
fn get_result<'ctx>(
    generator: &CodeGenerator<'ctx>,
    results: &HashMap<String, Option<PointerValue<'ctx>>>,
    name: &str,
) -> PointerValue<'ctx> {
    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    if generator.use_static_result_alloc {
        results
            .get(name)
            .unwrap_or_else(|| panic!("Result {} not found.", name))
            .unwrap_or_else(|| get_result(generator, results, "__unused__"))
    } else {
        results
            .get(name)
            .unwrap_or_else(|| panic!("Result {} not found.", name))
            .unwrap_or_else(|| result::get_zero(generator))
    }
}

fn measure<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubit: &str,
    target: &str,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    results: &mut HashMap<String, Option<PointerValue<'ctx>>>,
) {
    if generator.use_static_result_alloc {
        // measure the qubit and save the result to a temporary value
        generator.emit_void_call(
            generator.qis_mz_body(),
            &[
                get_qubit(qubits, qubit).into(),
                get_result(generator, results, target).into(),
            ],
        );
    } else {
        // measure the qubit and save the result to a temporary value
        let new_value = generator.emit_call_with_return(
            generator.qis_m_body(),
            &[get_qubit(qubits, qubit).into()],
            target,
        );
        results.insert(target.to_owned(), Some(new_value.into_pointer_value()));
    }
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
    results: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    entry_point: FunctionValue,
) {
    let get_qubit = |name| get_qubit(qubits, name);

    match inst {
        Instruction::Cx(inst) => {
            let control = get_qubit(&inst.control);
            let qubit = get_qubit(&inst.target);
            controlled(generator, generator.qis_cnot_body(), control, qubit);
        }
        Instruction::Cz(inst) => {
            let control = get_qubit(&inst.control);
            let qubit = get_qubit(&inst.target);
            controlled(generator, generator.qis_cz_body(), control, qubit);
        }
        Instruction::H(inst) => {
            generator.emit_void_call(generator.qis_h_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::M(inst) => {
            measure(generator, &inst.qubit, &inst.target, qubits, results);
        }
        Instruction::Reset(inst) => {
            generator.emit_void_call(generator.qis_reset_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::Rx(inst) => {
            generator.emit_void_call(
                generator.qis_rx_body(),
                &[
                    generator.f64_to_f64(inst.theta),
                    get_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::Ry(inst) => {
            generator.emit_void_call(
                generator.qis_ry_body(),
                &[
                    generator.f64_to_f64(inst.theta),
                    get_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::Rz(inst) => {
            generator.emit_void_call(
                generator.qis_rz_body(),
                &[
                    generator.f64_to_f64(inst.theta),
                    get_qubit(&inst.qubit).into(),
                ],
            );
        }
        Instruction::S(inst) => {
            generator.emit_void_call(generator.qis_s_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::SAdj(inst) => {
            generator.emit_void_call(generator.qis_s_adj(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::T(inst) => {
            generator.emit_void_call(generator.qis_t_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::TAdj(inst) => {
            generator.emit_void_call(generator.qis_t_adj(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::X(inst) => {
            generator.emit_void_call(generator.qis_x_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::Y(inst) => {
            generator.emit_void_call(generator.qis_y_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::Z(inst) => {
            generator.emit_void_call(generator.qis_z_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::Call(call) => emit_call(generator, qubits, results, call),
        Instruction::If(if_) => emit_if(generator, qubits, results, entry_point, if_),
    }
}

fn emit_call<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    results: &HashMap<String, Option<PointerValue<'ctx>>>,
    call: &Call,
) {
    let args: Vec<_> = call
        .args
        .iter()
        .map(|value| match value {
            Value::Integer(value) => generator
                .context
                .custom_width_int_type(value.width())
                .const_int(value.value(), false)
                .into(),
            Value::Double(value) => generator.f64_to_f64(*value),
            Value::Qubit(name) => get_qubit(qubits, name).into(),
            Value::Result(name) => get_result(generator, results, name).into(),
        })
        .collect();

    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    let function = generator
        .module
        .get_function(&call.name)
        .unwrap_or_else(|| panic!("Function {} not found.", &call.name));

    generator.emit_void_call(function, args.as_slice());
}

fn emit_if<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    results: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    entry_point: FunctionValue,
    if_: &If,
) {
    let result = get_result(generator, results, &if_.condition);

    let condition = if generator.use_static_result_alloc {
        result::read_result(generator, result)
    } else {
        result::equal(generator, result, result::get_one(generator))
    };

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
            emit(generator, inst, qubits, results, entry_point);
        }

        generator.builder.build_unconditional_branch(continue_block);
    };

    emit_block(then_block, &if_.then_insts);
    emit_block(else_block, &if_.else_insts);
    generator.builder.position_at_end(continue_block);
}

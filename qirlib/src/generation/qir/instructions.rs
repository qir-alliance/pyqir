// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::CodeGenerator,
    generation::{
        emit::Environment,
        interop::{Call, If, Instruction, Value},
        qir::result,
    },
};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
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
        // error if the key isn't found
        // error if static result wasn't initialized
        results
            .get(name)
            .unwrap_or_else(|| panic!("Result {} not found.", name))
            .unwrap_or_else(|| panic!("Result {} not initialized.", name))
    } else {
        // error if the key isn't found
        // return 0 if result is accessed prior to read.
        results
            .get(name)
            .unwrap_or_else(|| panic!("Result {} not found.", name))
            .unwrap_or_else(|| result::get_zero(generator))
    }
}

fn get_value<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &Environment<'ctx>,
    value: &Value,
) -> BasicMetadataValueEnum<'ctx> {
    match value {
        Value::Integer(i) => generator
            .context
            .custom_width_int_type(i.width())
            .const_int(i.value(), false)
            .into(),
        Value::Double(d) => generator.f64_to_f64(*d),
        Value::Qubit(q) => get_qubit(&env.qubits, q).into(),
        Value::Result(r) => get_result(generator, &env.results, r).into(),
        Value::Variable(v) => (*env
            .variables
            .get(v)
            .unwrap_or_else(|| panic!("Variable {:?} not found.", v)))
        .into(),
    }
}

fn measure<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    qubit: &str,
    target: &str,
) {
    if generator.use_static_result_alloc {
        // measure the qubit and save the result to a temporary value
        generator.emit_void_call(
            generator.qis_mz_body(),
            &[
                get_qubit(&env.qubits, qubit).into(),
                get_result(generator, &env.results, target).into(),
            ],
        );
    } else {
        // measure the qubit and save the result to a temporary value
        let new_value = generator.emit_call_with_return(
            generator.qis_m_body(),
            &[get_qubit(&env.qubits, qubit).into()],
            target,
        );
        env.results
            .insert(target.to_owned(), Some(new_value.into_pointer_value()));
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
    env: &mut Environment<'ctx>,
    inst: &Instruction,
    entry_point: FunctionValue,
) {
    let get_qubit = |name| get_qubit(&env.qubits, name);

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
        Instruction::M(inst) => measure(generator, env, &inst.qubit, &inst.target),
        Instruction::Reset(inst) => {
            generator.emit_void_call(generator.qis_reset_body(), &[get_qubit(&inst.qubit).into()]);
        }
        Instruction::Rx(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_qubit(&inst.qubit).into();
            generator.emit_void_call(generator.qis_rx_body(), &[theta, qubit]);
        }
        Instruction::Ry(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_qubit(&inst.qubit).into();
            generator.emit_void_call(generator.qis_ry_body(), &[theta, qubit]);
        }
        Instruction::Rz(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_qubit(&inst.qubit).into();
            generator.emit_void_call(generator.qis_rz_body(), &[theta, qubit]);
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
        Instruction::Call(call) => emit_call(generator, env, call),
        Instruction::If(if_) => emit_if(generator, env, entry_point, if_),
    }
}

fn emit_call<'ctx>(generator: &CodeGenerator<'ctx>, env: &mut Environment<'ctx>, call: &Call) {
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
            Value::Qubit(name) => get_qubit(&env.qubits, name).into(),
            Value::Result(name) => get_result(generator, &env.results, name).into(),
            Value::Variable(variable) => (*env.variables.get(variable).unwrap()).into(),
        })
        .collect();

    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    let function = generator
        .module
        .get_function(&call.name)
        .unwrap_or_else(|| panic!("Function {} not found.", &call.name));

    match call.result {
        None => {
            generator.emit_void_call(function, args.as_slice());
        }
        Some(variable) => {
            let value = generator.emit_call_with_return(function, args.as_slice(), "");
            env.variables.insert(variable, value);
        }
    }
}

fn emit_if<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    entry_point: FunctionValue,
    if_: &If,
) {
    let result = get_result(generator, &env.results, &if_.condition);

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
            emit(generator, env, inst, entry_point);
        }

        generator.builder.build_unconditional_branch(continue_block);
    };

    emit_block(then_block, &if_.then_insts);
    emit_block(else_block, &if_.else_insts);
    generator.builder.position_at_end(continue_block);
}

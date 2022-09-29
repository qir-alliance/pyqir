// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::{qis, types, CodeGenerator},
    generation::{
        env::Environment,
        interop::{BinaryKind, BinaryOp, Call, If, IfResult, Instruction, IntPredicate, Value},
    },
};
use inkwell::values::{BasicMetadataValueEnum, FunctionValue, IntValue};

pub(crate) fn emit<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    inst: &Instruction,
    entry_point: FunctionValue,
) {
    match inst {
        Instruction::Cx(inst) => {
            let control = get_value(generator, env, &inst.control);
            let qubit = get_value(generator, env, &inst.target);
            qis::call_cnot(generator.module(), generator.builder(), control, qubit);
        }
        Instruction::Cz(inst) => {
            let control = get_value(generator, env, &inst.control);
            let qubit = get_value(generator, env, &inst.target);
            qis::call_cz(generator.module(), generator.builder(), control, qubit);
        }
        Instruction::H(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_h(generator.module(), generator.builder(), qubit);
        }
        Instruction::M(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            let target = get_value(generator, env, &inst.target);
            qis::call_mz(generator.module(), generator.builder(), qubit, target);
        }
        Instruction::Reset(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_reset(generator.module(), generator.builder(), qubit);
        }
        Instruction::Rx(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_rx(generator.module(), generator.builder(), theta, qubit);
        }
        Instruction::Ry(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_ry(generator.module(), generator.builder(), theta, qubit);
        }
        Instruction::Rz(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_rz(generator.module(), generator.builder(), theta, qubit);
        }
        Instruction::S(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_s(generator.module(), generator.builder(), qubit);
        }
        Instruction::SAdj(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_s_adj(generator.module(), generator.builder(), qubit);
        }
        Instruction::T(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_t(generator.module(), generator.builder(), qubit);
        }
        Instruction::TAdj(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_t_adj(generator.module(), generator.builder(), qubit);
        }
        Instruction::X(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_x(generator.module(), generator.builder(), qubit);
        }
        Instruction::Y(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_y(generator.module(), generator.builder(), qubit);
        }
        Instruction::Z(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            qis::call_z(generator.module(), generator.builder(), qubit);
        }
        Instruction::BinaryOp(op) => emit_binary_op(generator, env, op),
        Instruction::Call(call) => emit_call(generator, env, call),
        Instruction::If(if_bool) => emit_if_bool(generator, env, entry_point, if_bool),
        Instruction::IfResult(if_result) => emit_if_result(generator, env, entry_point, if_result),
    }
}

fn emit_binary_op<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    op: &BinaryOp,
) {
    let lhs = get_value(generator, env, &op.lhs).into_int_value();
    let rhs = get_value(generator, env, &op.rhs).into_int_value();
    let result = match op.kind {
        BinaryKind::And => generator.builder().build_and(lhs, rhs, ""),
        BinaryKind::Or => generator.builder().build_or(lhs, rhs, ""),
        BinaryKind::Xor => generator.builder().build_xor(lhs, rhs, ""),
        BinaryKind::Add => generator.builder().build_int_add(lhs, rhs, ""),
        BinaryKind::Sub => generator.builder().build_int_sub(lhs, rhs, ""),
        BinaryKind::Mul => generator.builder().build_int_mul(lhs, rhs, ""),
        BinaryKind::Shl => generator.builder().build_left_shift(lhs, rhs, ""),
        BinaryKind::LShr => generator.builder().build_right_shift(lhs, rhs, false, ""),
        BinaryKind::ICmp(pred) => {
            generator
                .builder()
                .build_int_compare(to_inkwell_predicate(pred), lhs, rhs, "")
        }
    };
    env.set_variable(op.result, result.into()).unwrap();
}

fn to_inkwell_predicate(pred: IntPredicate) -> inkwell::IntPredicate {
    match pred {
        IntPredicate::EQ => inkwell::IntPredicate::EQ,
        IntPredicate::NE => inkwell::IntPredicate::NE,
        IntPredicate::UGT => inkwell::IntPredicate::UGT,
        IntPredicate::UGE => inkwell::IntPredicate::UGE,
        IntPredicate::ULT => inkwell::IntPredicate::ULT,
        IntPredicate::ULE => inkwell::IntPredicate::ULE,
        IntPredicate::SGT => inkwell::IntPredicate::SGT,
        IntPredicate::SGE => inkwell::IntPredicate::SGE,
        IntPredicate::SLT => inkwell::IntPredicate::SLT,
        IntPredicate::SLE => inkwell::IntPredicate::SLE,
    }
}

fn emit_call<'ctx>(generator: &CodeGenerator<'ctx>, env: &mut Environment<'ctx>, call: &Call) {
    let args: Vec<_> = call
        .args
        .iter()
        .map(|value| get_value(generator, env, value))
        .collect();

    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    let function = generator
        .module()
        .get_function(&call.name)
        .unwrap_or_else(|| panic!("Function {} not found.", &call.name));

    match call.result {
        None => {
            generator.builder().build_call(function, &args, "");
        }
        Some(var) => {
            let call = generator.builder().build_call(function, &args, "");
            let value = call.try_as_basic_value().left().unwrap();
            env.set_variable(var, value).unwrap();
        }
    }
}

fn emit_if_bool<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    entry_point: FunctionValue,
    if_bool: &If,
) {
    emit_if(
        generator,
        env,
        entry_point,
        get_value(generator, env, &if_bool.cond).into_int_value(),
        &if_bool.if_true,
        &if_bool.if_false,
    );
}

fn emit_if_result<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    entry_point: FunctionValue,
    if_result: &IfResult,
) {
    let result_cond = get_value(generator, env, &if_result.cond);
    let bool_cond = qis::call_read_result(generator.module(), generator.builder(), result_cond);
    emit_if(
        generator,
        env,
        entry_point,
        bool_cond,
        &if_result.if_one,
        &if_result.if_zero,
    );
}

fn emit_if<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    entry_point: FunctionValue,
    cond: IntValue<'ctx>,
    then_insts: &[Instruction],
    else_insts: &[Instruction],
) {
    let then_block = generator.context().append_basic_block(entry_point, "then");
    let else_block = generator.context().append_basic_block(entry_point, "else");

    generator
        .builder()
        .build_conditional_branch(cond, then_block, else_block);

    let continue_block = generator
        .context()
        .append_basic_block(entry_point, "continue");

    let mut emit_block = |block, insts| {
        generator.builder().position_at_end(block);

        for inst in insts {
            emit(generator, env, inst, entry_point);
        }

        generator
            .builder()
            .build_unconditional_branch(continue_block);
    };

    emit_block(then_block, then_insts);
    emit_block(else_block, else_insts);
    generator.builder().position_at_end(continue_block);
}

fn get_value<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &Environment<'ctx>,
    value: &Value,
) -> BasicMetadataValueEnum<'ctx> {
    match value {
        Value::Int(i) => generator
            .context()
            .custom_width_int_type(i.width())
            .const_int(i.value(), false)
            .into(),
        &Value::Double(d) => generator.context().f64_type().const_float(d).into(),
        &Value::Qubit(id) => types::qubit_id(generator.module(), generator.builder(), id).into(),
        &Value::Result(id) => types::result_id(generator.module(), generator.builder(), id).into(),
        &Value::Variable(v) => env
            .variable(v)
            .unwrap_or_else(|| panic!("Variable {:?} not found.", v))
            .into(),
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::{qis, types, BuilderRef},
    generation::{
        env::Environment,
        interop::{BinaryKind, BinaryOp, Call, If, IfResult, Instruction, IntPredicate, Value},
    },
};
use inkwell::values::{BasicMetadataValueEnum, IntValue};

pub(crate) fn emit<'ctx>(
    builder: BuilderRef<'ctx, '_>,
    env: &mut Environment<'ctx>,
    inst: &Instruction,
) {
    match inst {
        Instruction::Cx(inst) => {
            let control = get_value(builder, env, &inst.control);
            let qubit = get_value(builder, env, &inst.target);
            qis::call_cnot(builder, control, qubit);
        }
        Instruction::Cz(inst) => {
            let control = get_value(builder, env, &inst.control);
            let qubit = get_value(builder, env, &inst.target);
            qis::call_cz(builder, control, qubit);
        }
        Instruction::H(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_h(builder, qubit);
        }
        Instruction::M(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            let target = get_value(builder, env, &inst.target);
            qis::call_mz(builder, qubit, target);
        }
        Instruction::Reset(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_reset(builder, qubit);
        }
        Instruction::Rx(inst) => {
            let theta = get_value(builder, env, &inst.theta);
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_rx(builder, theta, qubit);
        }
        Instruction::Ry(inst) => {
            let theta = get_value(builder, env, &inst.theta);
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_ry(builder, theta, qubit);
        }
        Instruction::Rz(inst) => {
            let theta = get_value(builder, env, &inst.theta);
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_rz(builder, theta, qubit);
        }
        Instruction::S(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_s(builder, qubit);
        }
        Instruction::SAdj(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_s_adj(builder, qubit);
        }
        Instruction::T(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_t(builder, qubit);
        }
        Instruction::TAdj(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_t_adj(builder, qubit);
        }
        Instruction::X(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_x(builder, qubit);
        }
        Instruction::Y(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_y(builder, qubit);
        }
        Instruction::Z(inst) => {
            let qubit = get_value(builder, env, &inst.qubit);
            qis::call_z(builder, qubit);
        }
        Instruction::BinaryOp(op) => emit_binary_op(builder, env, op),
        Instruction::Call(call) => emit_call(builder, env, call),
        Instruction::If(if_bool) => emit_if_bool(builder, env, if_bool),
        Instruction::IfResult(if_result) => {
            emit_if_result(builder, env, if_result);
        }
    }
}

fn emit_binary_op<'ctx>(builder: BuilderRef<'ctx, '_>, env: &mut Environment<'ctx>, op: &BinaryOp) {
    let lhs = get_value(builder, env, &op.lhs).into_int_value();
    let rhs = get_value(builder, env, &op.rhs).into_int_value();
    let result = match op.kind {
        BinaryKind::And => builder.build_and(lhs, rhs, ""),
        BinaryKind::Or => builder.build_or(lhs, rhs, ""),
        BinaryKind::Xor => builder.build_xor(lhs, rhs, ""),
        BinaryKind::Add => builder.build_int_add(lhs, rhs, ""),
        BinaryKind::Sub => builder.build_int_sub(lhs, rhs, ""),
        BinaryKind::Mul => builder.build_int_mul(lhs, rhs, ""),
        BinaryKind::Shl => builder.build_left_shift(lhs, rhs, ""),
        BinaryKind::LShr => builder.build_right_shift(lhs, rhs, false, ""),
        BinaryKind::ICmp(pred) => {
            builder.build_int_compare(to_inkwell_predicate(pred), lhs, rhs, "")
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

fn emit_call<'ctx>(builder: BuilderRef<'ctx, '_>, env: &mut Environment<'ctx>, call: &Call) {
    let args: Vec<_> = call
        .args
        .iter()
        .map(|value| get_value(builder, env, value))
        .collect();

    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    let function = builder
        .module()
        .get_function(&call.name)
        .unwrap_or_else(|| panic!("Function {} not found.", &call.name));

    match call.result {
        None => {
            builder.build_call(function, &args, "");
        }
        Some(var) => {
            let call = builder.build_call(function, &args, "");
            let value = call.try_as_basic_value().left().unwrap();
            env.set_variable(var, value).unwrap();
        }
    }
}

fn emit_if_bool<'ctx>(builder: BuilderRef<'ctx, '_>, env: &mut Environment<'ctx>, if_bool: &If) {
    emit_if(
        builder,
        env,
        get_value(builder, env, &if_bool.cond).into_int_value(),
        &if_bool.if_true,
        &if_bool.if_false,
    );
}

fn emit_if_result<'ctx>(
    builder: BuilderRef<'ctx, '_>,
    env: &mut Environment<'ctx>,
    if_result: &IfResult,
) {
    let result_cond = get_value(builder, env, &if_result.cond);
    let bool_cond = qis::call_read_result(builder, result_cond);
    emit_if(
        builder,
        env,
        bool_cond,
        &if_result.if_one,
        &if_result.if_zero,
    );
}

fn emit_if<'ctx>(
    builder: BuilderRef<'ctx, '_>,
    env: &mut Environment<'ctx>,
    cond: IntValue<'ctx>,
    then_insts: &[Instruction],
    else_insts: &[Instruction],
) {
    let context = builder.module().get_context();
    let function = builder.get_insert_block().unwrap().get_parent().unwrap();
    let then_block = context.append_basic_block(function, "then");
    let else_block = context.append_basic_block(function, "else");
    builder.build_conditional_branch(cond, then_block, else_block);

    let continue_block = context.append_basic_block(function, "continue");
    let mut emit_block = |block, insts| {
        builder.position_at_end(block);
        for inst in insts {
            emit(builder, env, inst);
        }
        builder.build_unconditional_branch(continue_block);
    };

    emit_block(then_block, then_insts);
    emit_block(else_block, else_insts);
    builder.position_at_end(continue_block);
}

fn get_value<'ctx>(
    builder: BuilderRef<'ctx, '_>,
    env: &Environment<'ctx>,
    value: &Value,
) -> BasicMetadataValueEnum<'ctx> {
    match value {
        Value::Int(i) => builder
            .module()
            .get_context()
            .custom_width_int_type(i.width())
            .const_int(i.value(), false)
            .into(),
        &Value::Double(d) => builder
            .module()
            .get_context()
            .f64_type()
            .const_float(d)
            .into(),
        &Value::Qubit(id) => types::qubit_id(builder, id).into(),
        &Value::Result(id) => types::result_id(builder, id).into(),
        &Value::Variable(v) => env
            .variable(v)
            .unwrap_or_else(|| panic!("Variable {:?} not found.", v))
            .into(),
    }
}

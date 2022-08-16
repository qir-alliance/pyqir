// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    codegen::CodeGenerator,
    generation::{
        env::Environment,
        interop::{
            BinaryKind, BinaryOp, Call, If, IfResult, Instruction, IntPredicate, Measured, Value,
        },
    },
};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue};

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
            generator.emit_void_call(generator.qis_cnot_body(), &[control, qubit]);
        }
        Instruction::Cz(inst) => {
            let control = get_value(generator, env, &inst.control);
            let qubit = get_value(generator, env, &inst.target);
            generator.emit_void_call(generator.qis_cz_body(), &[control, qubit]);
        }
        Instruction::H(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_h_body(), &[qubit]);
        }
        Instruction::M(inst) => emit_measured(generator, env, inst),
        Instruction::Reset(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_reset_body(), &[qubit]);
        }
        Instruction::Rx(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_rx_body(), &[theta, qubit]);
        }
        Instruction::Ry(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_ry_body(), &[theta, qubit]);
        }
        Instruction::Rz(inst) => {
            let theta = get_value(generator, env, &inst.theta);
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_rz_body(), &[theta, qubit]);
        }
        Instruction::S(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_s_body(), &[qubit]);
        }
        Instruction::SAdj(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_s_adj(), &[qubit]);
        }
        Instruction::T(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_t_body(), &[qubit]);
        }
        Instruction::TAdj(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_t_adj(), &[qubit]);
        }
        Instruction::X(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_x_body(), &[qubit]);
        }
        Instruction::Y(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_y_body(), &[qubit]);
        }
        Instruction::Z(inst) => {
            let qubit = get_value(generator, env, &inst.qubit);
            generator.emit_void_call(generator.qis_z_body(), &[qubit]);
        }
        Instruction::BinaryOp(op) => emit_binary_op(generator, env, op),
        Instruction::Call(call) => emit_call(generator, env, call),
        Instruction::If(if_) => emit_if_bool(generator, env, entry_point, if_),
        Instruction::IfResult(if_result) => emit_if_result(generator, env, entry_point, if_result),
    }
}

fn emit_measured<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    measured: &Measured,
) {
    let qubit = get_value(generator, env, &measured.qubit);
    generator.emit_void_call(
        generator.qis_mz_body(),
        &[qubit, get_result(env, &measured.target).into()],
    );
}

fn emit_binary_op<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    op: &BinaryOp,
) {
    let lhs = get_value(generator, env, &op.lhs).into_int_value();
    let rhs = get_value(generator, env, &op.rhs).into_int_value();
    let result = match op.kind {
        BinaryKind::And => generator.builder.build_and(lhs, rhs, ""),
        BinaryKind::Or => generator.builder.build_or(lhs, rhs, ""),
        BinaryKind::Xor => generator.builder.build_xor(lhs, rhs, ""),
        BinaryKind::Add => generator.builder.build_int_add(lhs, rhs, ""),
        BinaryKind::Sub => generator.builder.build_int_sub(lhs, rhs, ""),
        BinaryKind::Mul => generator.builder.build_int_mul(lhs, rhs, ""),
        BinaryKind::Shl => generator.builder.build_left_shift(lhs, rhs, ""),
        BinaryKind::LShr => generator.builder.build_right_shift(lhs, rhs, false, ""),
        BinaryKind::ICmp(pred) => {
            generator
                .builder
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
        .module
        .get_function(&call.name)
        .unwrap_or_else(|| panic!("Function {} not found.", &call.name));

    match call.result {
        None => {
            generator.emit_void_call(function, args.as_slice());
        }
        Some(var) => {
            let value = generator.emit_call_with_return(function, args.as_slice(), "");
            env.set_variable(var, value).unwrap();
        }
    }
}

fn emit_if_bool<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    entry_point: FunctionValue,
    if_: &If,
) {
    emit_if(
        generator,
        env,
        entry_point,
        get_value(generator, env, &if_.cond).into_int_value(),
        &if_.if_true,
        &if_.if_false,
    );
}

fn emit_if_result<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &mut Environment<'ctx>,
    entry_point: FunctionValue,
    if_result: &IfResult,
) {
    let result = get_result(env, &if_result.cond);
    let cond = read_result(generator, result.into());

    emit_if(
        generator,
        env,
        entry_point,
        cond,
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
    let then_block = generator.context.append_basic_block(entry_point, "then");
    let else_block = generator.context.append_basic_block(entry_point, "else");

    generator
        .builder
        .build_conditional_branch(cond, then_block, else_block);

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

    emit_block(then_block, then_insts);
    emit_block(else_block, else_insts);
    generator.builder.position_at_end(continue_block);
}

fn get_value<'ctx>(
    generator: &CodeGenerator<'ctx>,
    env: &Environment<'ctx>,
    value: &Value,
) -> BasicMetadataValueEnum<'ctx> {
    match value {
        Value::Int(i) => generator
            .context
            .custom_width_int_type(i.width())
            .const_int(i.value(), false)
            .into(),
        &Value::Double(d) => generator.f64_to_f64(d),
        Value::Qubit(q) => get_qubit(env, q).into(),
        Value::Result(r) => get_result(env, r).into(),
        &Value::Variable(v) => env
            .variable(v)
            .unwrap_or_else(|| panic!("Variable {:?} not found.", v))
            .into(),
    }
}

/// # Panics
///
/// Panics if the qubit name doesn't exist
fn get_qubit<'ctx>(env: &Environment<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    env.qubit(name)
        .unwrap_or_else(|| panic!("Qubit {} not found.", name))
}

/// Gets the most recent value of a result name. Defaults to zero if the result has been declared
/// but not yet measured.
///
/// # Panics
///
/// Panics if the result name has not been declared.
fn get_result<'ctx>(env: &Environment<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
    // TODO: Panicking can be unfriendly to Python clients.
    // See: https://github.com/qir-alliance/pyqir/issues/31
    env.result(name)
        .unwrap_or_else(|| panic!("Result {} not found.", name))
}

fn read_result<'ctx>(
    generator: &CodeGenerator<'ctx>,
    result: BasicMetadataValueEnum<'ctx>,
) -> IntValue<'ctx> {
    generator
        .emit_call_with_return(generator.qis_read_result(), &[result], "equal")
        .into_int_value()
}

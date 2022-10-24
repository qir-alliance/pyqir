// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types;
use inkwell::{
    attributes::AttributeLoc,
    types::{ArrayType, BasicTypeEnum},
    values::{AnyValueEnum, BasicValueEnum, FunctionValue, InstructionOpcode},
};

#[must_use]
pub fn qubit_id(value: AnyValueEnum) -> Option<u64> {
    if types::is_qubit(value.get_type()) {
        pointer_to_int(value)
    } else {
        None
    }
}

#[must_use]
pub fn result_id(value: AnyValueEnum) -> Option<u64> {
    if types::is_result(value.get_type()) {
        pointer_to_int(value)
    } else {
        None
    }
}

#[must_use]
pub fn is_entry_point(function: FunctionValue) -> bool {
    function
        .get_string_attribute(AttributeLoc::Function, "EntryPoint")
        .is_some()
}

#[must_use]
pub fn is_interop_friendly(function: FunctionValue) -> bool {
    function
        .get_string_attribute(AttributeLoc::Function, "InteropFriendly")
        .is_some()
}

#[must_use]
pub fn required_num_qubits(function: FunctionValue) -> Option<u64> {
    let attribute = function.get_string_attribute(AttributeLoc::Function, "requiredQubits")?;
    attribute.get_string_value().to_str().ok()?.parse().ok()
}

#[must_use]
pub fn required_num_results(function: FunctionValue) -> Option<u64> {
    let attribute = function.get_string_attribute(AttributeLoc::Function, "requiredResults")?;
    attribute.get_string_value().to_str().ok()?.parse().ok()
}

#[must_use]
pub fn global_byte_string_value_name(value: AnyValueEnum) -> Option<String> {
    let instruction = match value {
        AnyValueEnum::PointerValue(p) => p.as_instruction(),
        AnyValueEnum::InstructionValue(i) => Some(i),
        _ => None,
    }
    .filter(|i| i.get_opcode() == InstructionOpcode::GetElementPtr)?;

    match instruction.get_operand(0)?.left()? {
        BasicValueEnum::ArrayValue(array) if is_byte_array(array.get_type()) => Some(
            array
                .get_name()
                .to_str()
                .expect("Name is not valid UTF-8.")
                .to_string(),
        ),
        _ => None,
    }
}

fn pointer_to_int(value: AnyValueEnum) -> Option<u64> {
    match value {
        AnyValueEnum::PointerValue(p) => {
            let context = p.get_type().get_context();
            p.const_to_int(context.i64_type())
                .get_zero_extended_constant()
        }
        _ => None,
    }
}

fn is_byte_array(ty: ArrayType) -> bool {
    match ty.get_element_type() {
        BasicTypeEnum::IntType(i) => i.get_bit_width() == 8,
        _ => false,
    }
}

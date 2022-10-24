// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types;
use inkwell::{
    attributes::AttributeLoc,
    values::{AnyValueEnum, FunctionValue},
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

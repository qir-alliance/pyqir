// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types;
use core::slice;
use inkwell::{
    attributes::AttributeLoc,
    context::ContextRef,
    types::{AnyTypeEnum, PointerType},
    values::{AnyValueEnum, FunctionValue, PointerValue},
    LLVMReference,
};
use llvm_sys::{
    core::{
        LLVMConstIntGetZExtValue, LLVMGetAsString, LLVMGetConstOpcode, LLVMGetInitializer,
        LLVMGetOperand, LLVMIsAConstantDataSequential, LLVMIsAConstantExpr,
    },
    LLVMOpcode,
};
use std::convert::TryFrom;

#[must_use]
pub fn qubit<'ctx>(context: &ContextRef<'ctx>, id: u64) -> PointerValue<'ctx> {
    context
        .i64_type()
        .const_int(id, false)
        .const_to_pointer(types::qubit(context))
}

#[must_use]
pub fn qubit_id(value: AnyValueEnum) -> Option<u64> {
    if types::is_qubit(value.get_type()) {
        pointer_to_int(value)
    } else {
        None
    }
}

#[must_use]
pub fn result<'ctx>(context: &ContextRef<'ctx>, id: u64) -> PointerValue<'ctx> {
    context
        .i64_type()
        .const_int(id, false)
        .const_to_pointer(types::result(context))
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
#[allow(clippy::missing_panics_doc)]
pub fn constant_bytes(value: AnyValueEnum) -> Option<&[u8]> {
    let pointer = match value {
        AnyValueEnum::PointerValue(p) if is_byte_string(p.get_type()) => Some(p),
        _ => None,
    }?;

    let expr = unsafe { LLVMIsAConstantExpr(pointer.get_ref()) };
    let opcode = unsafe { LLVMGetConstOpcode(expr) };
    if opcode != LLVMOpcode::LLVMGetElementPtr {
        return None;
    }

    let element = unsafe { LLVMGetOperand(expr, 0) };
    let offset = unsafe { LLVMConstIntGetZExtValue(LLVMGetOperand(expr, 1)) };
    let offset = usize::try_from(offset).expect("Pointer offset larger than usize.");
    let init = unsafe { LLVMIsAConstantDataSequential(LLVMGetInitializer(element)) };
    if init.is_null() {
        return None;
    }

    let mut len = 0;
    let data = unsafe { LLVMGetAsString(init, &mut len) };
    let data = unsafe { slice::from_raw_parts(data.cast(), len) };
    Some(&data[offset..])
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

fn is_byte_string(ty: PointerType) -> bool {
    match ty.get_element_type() {
        AnyTypeEnum::IntType(i) => i.get_bit_width() == 8,
        _ => false,
    }
}

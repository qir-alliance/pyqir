// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types;
use core::slice;
use inkwell::{
    attributes::AttributeLoc,
    context::ContextRef,
    module::{Linkage, Module},
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

pub fn entry_point<'ctx>(
    module: &Module<'ctx>,
    name: &str,
    required_num_qubits: u64,
    required_num_results: u64,
) -> FunctionValue<'ctx> {
    let context = module.get_context();
    let ty = context.void_type().fn_type(&[], false);
    let entry_point = module.add_function(name, ty, None);
    entry_point.add_attribute(
        AttributeLoc::Function,
        context.create_string_attribute("EntryPoint", ""),
    );
    add_num_attribute(entry_point, "requiredQubits", required_num_qubits);
    add_num_attribute(entry_point, "requiredResults", required_num_results);
    entry_point
}

#[must_use]
pub fn is_entry_point(function: FunctionValue) -> bool {
    function
        .get_string_attribute(AttributeLoc::Function, "EntryPoint")
        .is_some()
        || function
            .get_string_attribute(AttributeLoc::Function, "entry_point")
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
    if let Some(attribute) = function.get_string_attribute(AttributeLoc::Function, "requiredQubits")
    {
        attribute.get_string_value().to_str().ok()?.parse().ok()
    } else {
        let attribute =
            function.get_string_attribute(AttributeLoc::Function, "required_num_qubits")?;
        attribute.get_string_value().to_str().ok()?.parse().ok()
    }
}

#[must_use]
pub fn required_num_results(function: FunctionValue) -> Option<u64> {
    if let Some(attribute) =
        function.get_string_attribute(AttributeLoc::Function, "requiredResults")
    {
        attribute.get_string_value().to_str().ok()?.parse().ok()
    } else {
        let attribute =
            function.get_string_attribute(AttributeLoc::Function, "required_num_results")?;
        attribute.get_string_value().to_str().ok()?.parse().ok()
    }
}

#[must_use]
pub fn is_irreversible(function: FunctionValue) -> bool {
    function
        .get_string_attribute(AttributeLoc::Function, "irreversible")
        .is_some()
}

pub fn global_string<'ctx>(module: &Module<'ctx>, value: &[u8]) -> PointerValue<'ctx> {
    let context = module.get_context();
    let string = context.const_string(value, true);
    let size = string.get_type().get_size();
    let global = module.add_global(context.i8_type().array_type(size), None, "");
    global.set_linkage(Linkage::Internal);
    global.set_constant(true);
    global.set_initializer(&string);
    let zero = context.i32_type().const_zero();
    unsafe { global.as_pointer_value().const_gep(&[zero, zero]) }
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn extract_string(value: AnyValueEnum) -> Option<&[u8]> {
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

fn add_num_attribute(function: FunctionValue, key: &str, value: u64) {
    let context = function.get_type().get_context();
    let attribute = context.create_string_attribute(key, &value.to_string());
    function.add_attribute(AttributeLoc::Function, attribute);
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

#[cfg(test)]
mod tests {
    use crate::tests::assert_reference_ir;

    #[test]
    fn zero_required_qubits_results() -> Result<(), String> {
        assert_reference_ir("module/zero_required_qubits_results", 0, 0, |_| ())
    }

    #[test]
    fn one_required_qubit() -> Result<(), String> {
        assert_reference_ir("module/one_required_qubit", 1, 0, |_| ())
    }

    #[test]
    fn one_required_result() -> Result<(), String> {
        assert_reference_ir("module/one_required_result", 0, 1, |_| ())
    }

    #[test]
    fn many_required_qubits_results() -> Result<(), String> {
        assert_reference_ir("module/many_required_qubits_results", 5, 7, |_| ())
    }
}

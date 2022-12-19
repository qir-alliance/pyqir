// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types;
use const_str::raw_cstr;
use core::slice;
#[allow(clippy::wildcard_imports)]
use llvm_sys::{
    core::*, prelude::*, LLVMAttributeFunctionIndex, LLVMAttributeIndex, LLVMLinkage,
    LLVMOpaqueAttributeRef, LLVMOpcode, LLVMTypeKind, LLVMValueKind,
};
use std::{convert::TryFrom, ffi::CStr, ptr::NonNull, str};

pub unsafe fn qubit(context: LLVMContextRef, id: u64) -> LLVMValueRef {
    let i64 = LLVMInt64TypeInContext(context);
    let value = LLVMConstInt(i64, id, 0);
    LLVMConstIntToPtr(value, types::qubit(context))
}

pub unsafe fn qubit_id(value: LLVMValueRef) -> Option<u64> {
    if types::is_qubit(LLVMTypeOf(value)) {
        pointer_to_int(value)
    } else {
        None
    }
}

pub unsafe fn result(context: LLVMContextRef, id: u64) -> LLVMValueRef {
    let i64 = LLVMInt64TypeInContext(context);
    let value = LLVMConstInt(i64, id, 0);
    LLVMConstIntToPtr(value, types::result(context))
}

pub unsafe fn result_id(value: LLVMValueRef) -> Option<u64> {
    if types::is_result(LLVMTypeOf(value)) {
        pointer_to_int(value)
    } else {
        None
    }
}

pub unsafe fn entry_point(
    module: LLVMModuleRef,
    name: &CStr,
    required_num_qubits: u64,
    required_num_results: u64,
) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let void = LLVMVoidTypeInContext(context);
    let ty = LLVMFunctionType(void, [].as_mut_ptr(), 0, 0);
    let function = LLVMAddFunction(module, name.as_ptr(), ty);

    add_string_attribute(function, b"EntryPoint", b"");
    add_string_attribute(
        function,
        b"requiredQubits",
        required_num_qubits.to_string().as_bytes(),
    );
    add_string_attribute(
        function,
        b"requiredResults",
        required_num_results.to_string().as_bytes(),
    );

    function
}

pub unsafe fn is_entry_point(function: LLVMValueRef) -> bool {
    LLVMGetValueKind(function) == LLVMValueKind::LLVMFunctionValueKind
        && (get_string_attribute(function, LLVMAttributeFunctionIndex, b"EntryPoint").is_some()
            || get_string_attribute(function, LLVMAttributeFunctionIndex, b"entry_point").is_some())
}

pub unsafe fn is_irreversible(function: LLVMValueRef) -> bool {
    LLVMGetValueKind(function) == LLVMValueKind::LLVMFunctionValueKind
        && get_string_attribute(function, LLVMAttributeFunctionIndex, b"irreversible").is_some()
}

pub unsafe fn is_interop_friendly(function: LLVMValueRef) -> bool {
    LLVMGetValueKind(function) == LLVMValueKind::LLVMFunctionValueKind
        && get_string_attribute(function, LLVMAttributeFunctionIndex, b"InteropFriendly").is_some()
}

pub unsafe fn required_num_qubits(function: LLVMValueRef) -> Option<u64> {
    if LLVMGetValueKind(function) == LLVMValueKind::LLVMFunctionValueKind {
        let required_qubits =
            get_string_attribute(function, LLVMAttributeFunctionIndex, b"requiredQubits")?;
        let mut len = 0;
        let value = LLVMGetStringAttributeValue(required_qubits.as_ptr(), &mut len);
        let value = slice::from_raw_parts(value.cast(), len.try_into().unwrap());
        str::from_utf8(value).ok()?.parse().ok()
    } else {
        None
    }
}

pub unsafe fn required_num_results(function: LLVMValueRef) -> Option<u64> {
    if LLVMGetValueKind(function) == LLVMValueKind::LLVMFunctionValueKind {
        let required_qubits =
            get_string_attribute(function, LLVMAttributeFunctionIndex, b"requiredResults")?;
        let mut len = 0;
        let value = LLVMGetStringAttributeValue(required_qubits.as_ptr(), &mut len);
        let value = slice::from_raw_parts(value.cast(), len.try_into().unwrap());
        str::from_utf8(value).ok()?.parse().ok()
    } else {
        None
    }
}

pub unsafe fn global_string(module: LLVMModuleRef, value: &[u8]) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let string = LLVMConstStringInContext(
        context,
        value.as_ptr().cast(),
        value.len().try_into().unwrap(),
        0,
    );

    let len = LLVMGetArrayLength(LLVMTypeOf(string));
    let ty = LLVMArrayType(LLVMInt8TypeInContext(context), len);
    let global = LLVMAddGlobal(module, ty, raw_cstr!(""));
    LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);
    LLVMSetGlobalConstant(global, 1);
    LLVMSetInitializer(global, string);

    let zero = LLVMConstNull(LLVMInt32TypeInContext(context));
    let mut indices = [zero, zero];
    #[allow(deprecated)]
    LLVMConstGEP(
        global,
        indices.as_mut_ptr(),
        indices.len().try_into().unwrap(),
    )
}

pub unsafe fn extract_string(value: LLVMValueRef) -> Option<Vec<u8>> {
    if !is_byte_string(LLVMTypeOf(value)) {
        return None;
    }

    let expr = LLVMIsAConstantExpr(value);
    let opcode = LLVMGetConstOpcode(expr);
    if opcode != LLVMOpcode::LLVMGetElementPtr {
        return None;
    }

    let element = LLVMGetOperand(expr, 0);
    let offset = LLVMConstIntGetZExtValue(LLVMGetOperand(expr, 1));
    let offset = usize::try_from(offset).expect("Pointer offset larger than usize.");
    let init = LLVMIsAConstantDataSequential(LLVMGetInitializer(element));
    if init.is_null() {
        return None;
    }

    let mut len = 0;
    let data = LLVMGetAsString(init, &mut len);
    let data = slice::from_raw_parts(data.cast(), len);
    Some(data[offset..].to_vec())
}

unsafe fn add_string_attribute(function: LLVMValueRef, kind: &[u8], value: &[u8]) {
    let context = LLVMGetTypeContext(LLVMTypeOf(function));
    let attr = LLVMCreateStringAttribute(
        context,
        kind.as_ptr().cast(),
        kind.len().try_into().unwrap(),
        value.as_ptr().cast(),
        value.len().try_into().unwrap(),
    );
    LLVMAddAttributeAtIndex(function, LLVMAttributeFunctionIndex, attr);
}

unsafe fn get_string_attribute(
    function: LLVMValueRef,
    index: LLVMAttributeIndex,
    kind: &[u8],
) -> Option<NonNull<LLVMOpaqueAttributeRef>> {
    NonNull::new(LLVMGetStringAttributeAtIndex(
        function,
        index,
        kind.as_ptr().cast(),
        kind.len().try_into().unwrap(),
    ))
}

unsafe fn pointer_to_int(value: LLVMValueRef) -> Option<u64> {
    let ty = LLVMTypeOf(value);
    if LLVMGetTypeKind(ty) == LLVMTypeKind::LLVMPointerTypeKind && LLVMIsConstant(value) != 0 {
        let context = LLVMGetTypeContext(ty);
        let int = LLVMConstPtrToInt(value, LLVMInt64TypeInContext(context));
        Some(LLVMConstIntGetZExtValue(int))
    } else {
        None
    }
}

unsafe fn is_byte_string(ty: LLVMTypeRef) -> bool {
    if LLVMGetTypeKind(ty) == LLVMTypeKind::LLVMPointerTypeKind {
        let pointee = LLVMGetElementType(ty);
        LLVMGetTypeKind(pointee) == LLVMTypeKind::LLVMIntegerTypeKind
            && LLVMGetIntTypeWidth(pointee) == 8
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_reference_ir;

    #[test]
    fn zero_required_qubits_results() {
        assert_reference_ir("module/zero_required_qubits_results", 0, 0, |_| ());
    }

    #[test]
    fn one_required_qubit() {
        assert_reference_ir("module/one_required_qubit", 1, 0, |_| ());
    }

    #[test]
    fn one_required_result() {
        assert_reference_ir("module/one_required_result", 0, 1, |_| ());
    }

    #[test]
    fn many_required_qubits_results() {
        assert_reference_ir("module/many_required_qubits_results", 5, 7, |_| ());
    }
}

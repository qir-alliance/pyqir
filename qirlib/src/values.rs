// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use const_str::raw_cstr;
use core::slice;
#[allow(clippy::wildcard_imports)]
use llvm_sys::{
    core::*, prelude::*, LLVMAttributeFunctionIndex, LLVMAttributeIndex, LLVMLinkage,
    LLVMOpaqueAttributeRef, LLVMTypeKind, LLVMValueKind,
};
use std::{
    ffi::CStr,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
    str,
};

pub unsafe fn qubit(context: LLVMContextRef, id: u64) -> LLVMValueRef {
    let i64 = LLVMInt64TypeInContext(context);
    let value = LLVMConstInt(i64, id, 0);
    LLVMConstIntToPtr(value, LLVMPointerTypeInContext(context, 0))
}

pub unsafe fn ptr_id(value: LLVMValueRef) -> Option<u64> {
    if LLVMPointerTypeIsOpaque(LLVMTypeOf(value)) == 1 {
        pointer_to_int(value)
    } else {
        None
    }
}

pub unsafe fn result(context: LLVMContextRef, id: u64) -> LLVMValueRef {
    let i64 = LLVMInt64TypeInContext(context);
    let value = LLVMConstInt(i64, id, 0);
    LLVMConstIntToPtr(value, LLVMPointerTypeInContext(context, 0))
}

pub unsafe fn entry_point(
    module: LLVMModuleRef,
    name: &CStr,
    required_num_qubits: u64,
    required_num_results: u64,
    qir_profiles: &str,
    output_labeling_schema: &str,
) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let void = LLVMVoidTypeInContext(context);
    let ty = LLVMFunctionType(void, [].as_mut_ptr(), 0, 0);
    let function = LLVMAddFunction(module, name.as_ptr(), ty);

    add_string_attribute(function, b"entry_point", b"", LLVMAttributeFunctionIndex);
    add_string_attribute(
        function,
        b"required_num_qubits",
        required_num_qubits.to_string().as_bytes(),
        LLVMAttributeFunctionIndex,
    );
    add_string_attribute(
        function,
        b"required_num_results",
        required_num_results.to_string().as_bytes(),
        LLVMAttributeFunctionIndex,
    );

    add_string_attribute(
        function,
        b"qir_profiles",
        qir_profiles.as_bytes(),
        LLVMAttributeFunctionIndex,
    );

    add_string_attribute(
        function,
        b"output_labeling_schema",
        output_labeling_schema.as_bytes(),
        LLVMAttributeFunctionIndex,
    );

    function
}

pub unsafe fn is_entry_point(function: LLVMValueRef) -> bool {
    LLVMGetValueKind(function) == LLVMValueKind::LLVMFunctionValueKind
        && (get_string_attribute(function, LLVMAttributeFunctionIndex, b"entry_point").is_some()
            || get_string_attribute(function, LLVMAttributeFunctionIndex, b"EntryPoint").is_some())
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
            get_string_attribute(function, LLVMAttributeFunctionIndex, b"requiredQubits")
                .or_else(|| {
                    get_string_attribute(
                        function,
                        LLVMAttributeFunctionIndex,
                        b"required_num_qubits",
                    )
                })
                .or_else(|| {
                    get_string_attribute(
                        function,
                        LLVMAttributeFunctionIndex,
                        b"num_required_qubits",
                    )
                })?;

        let mut len = 0;
        let value = LLVMGetStringAttributeValue(required_qubits.as_ptr(), &raw mut len);
        let value = slice::from_raw_parts(value.cast(), len.try_into().unwrap());
        str::from_utf8(value).ok()?.parse().ok()
    } else {
        None
    }
}

pub unsafe fn required_num_results(function: LLVMValueRef) -> Option<u64> {
    if LLVMGetValueKind(function) == LLVMValueKind::LLVMFunctionValueKind {
        let required_results =
            get_string_attribute(function, LLVMAttributeFunctionIndex, b"requiredResults")
                .or_else(|| {
                    get_string_attribute(
                        function,
                        LLVMAttributeFunctionIndex,
                        b"required_num_results",
                    )
                })
                .or_else(|| {
                    get_string_attribute(
                        function,
                        LLVMAttributeFunctionIndex,
                        b"num_required_results",
                    )
                })?;
        let mut len = 0;
        let value = LLVMGetStringAttributeValue(required_results.as_ptr(), &raw mut len);
        let value = slice::from_raw_parts(value.cast(), len.try_into().unwrap());
        str::from_utf8(value).ok()?.parse().ok()
    } else {
        None
    }
}

#[cfg(feature = "llvm18-1")]
pub unsafe fn global_string(module: LLVMModuleRef, value: &[u8]) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let string = LLVMConstStringInContext(
        context,
        value.as_ptr().cast(),
        value.len().try_into().unwrap(),
        0,
    );

    let len = LLVMGetArrayLength2(LLVMTypeOf(string));
    let i8_ty = LLVMInt8TypeInContext(context);
    let ty = LLVMArrayType2(LLVMInt8TypeInContext(context), len);
    let global = LLVMAddGlobal(module, ty, raw_cstr!(""));
    LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);
    LLVMSetGlobalConstant(global, 1);
    LLVMSetInitializer(global, string);

    let zero = LLVMConstNull(LLVMInt32TypeInContext(context));
    let mut indices = [zero, zero];
    LLVMConstGEP2(
        i8_ty,
        global,
        indices.as_mut_ptr(),
        indices.len().try_into().unwrap(),
    )
}

#[cfg(any(feature = "llvm19-1", feature = "llvm20-1"))]
pub unsafe fn global_string(module: LLVMModuleRef, value: &[u8]) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let string = LLVMConstStringInContext2(context, value.as_ptr().cast(), value.len(), 0);

    let len = LLVMGetArrayLength2(LLVMTypeOf(string));
    let i8_ty = LLVMInt8TypeInContext(context);
    let ty = LLVMArrayType2(LLVMInt8TypeInContext(context), len);
    let global = LLVMAddGlobal(module, ty, raw_cstr!(""));
    LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);
    LLVMSetGlobalConstant(global, 1);
    LLVMSetInitializer(global, string);

    let zero = LLVMConstNull(LLVMInt32TypeInContext(context));
    let mut indices = [zero, zero];
    LLVMConstGEP2(
        i8_ty,
        global,
        indices.as_mut_ptr(),
        indices.len().try_into().unwrap(),
    )
}

pub unsafe fn extract_string(value: LLVMValueRef) -> Option<Vec<u8>> {
    if LLVMIsNull(value) != 0 {
        return None;
    }

    if LLVMIsGlobalConstant(value) == 0 {
        return None;
    }

    let element = LLVMGetOperand(value, 0);
    if LLVMIsAConstantAggregateZero(element) == element {
        return None;
    }
    let mut len = 0;
    let data = LLVMGetAsString(element, &raw mut len);
    let data = slice::from_raw_parts(data.cast(), len);
    Some(data[..].to_vec())
}

pub unsafe fn add_string_attribute(
    function: LLVMValueRef,
    key: &[u8],
    value: &[u8],
    index: LLVMAttributeIndex,
) {
    let context = LLVMGetTypeContext(LLVMTypeOf(function));
    let attr = LLVMCreateStringAttribute(
        context,
        key.as_ptr().cast(),
        key.len().try_into().unwrap(),
        value.as_ptr().cast(),
        value.len().try_into().unwrap(),
    );
    LLVMAddAttributeAtIndex(function, index, attr);
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

pub unsafe fn get_attribute_count(function: LLVMValueRef, index: LLVMAttributeIndex) -> usize {
    LLVMGetAttributeCountAtIndex(function, index)
        .try_into()
        .expect("Attribute count larger than usize.")
}

pub unsafe fn get_string_attribute_kind(attr: *mut LLVMOpaqueAttributeRef) -> String {
    let mut len = 0;
    let value = LLVMGetStringAttributeKind(attr, &raw mut len).cast();
    let value = slice::from_raw_parts(value, len.try_into().unwrap());
    str::from_utf8(value)
        .expect("Attribute kind is not valid UTF-8.")
        .to_string()
}

pub unsafe fn get_string_attribute_value(attr: *mut LLVMOpaqueAttributeRef) -> Option<String> {
    if LLVMIsStringAttribute(attr) == 0 {
        None
    } else {
        let mut len = 0;
        let value = LLVMGetStringAttributeValue(attr, &raw mut len).cast();
        let value = slice::from_raw_parts(value, len.try_into().unwrap());
        Some(
            str::from_utf8(value)
                .expect("Attribute kind is not valid UTF-8.")
                .to_string(),
        )
    }
}

pub unsafe fn get_attributes(
    function: LLVMValueRef,
    index: LLVMAttributeIndex,
) -> Vec<*mut LLVMOpaqueAttributeRef> {
    let count = get_attribute_count(function, index);
    if count == 0 {
        return Vec::new();
    }
    let attrs: Vec<MaybeUninit<*mut LLVMOpaqueAttributeRef>> = Vec::with_capacity(count);
    let mut attrs = ManuallyDrop::new(attrs);
    for _ in 0..count {
        attrs.push(MaybeUninit::uninit());
    }

    LLVMGetAttributesAtIndex(function, index, attrs.as_mut_ptr().cast());

    Vec::from_raw_parts(attrs.as_mut_ptr().cast(), attrs.len(), attrs.capacity())
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

#[cfg(test)]
mod string_attribute_tests {
    use std::ffi::CString;

    use llvm_sys::{
        core::{
            LLVMAddFunction, LLVMBuildRetVoid, LLVMContextCreate, LLVMContextDispose,
            LLVMCreateBuilderInContext, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
            LLVMVoidTypeInContext,
        },
        LLVMAttributeFunctionIndex, LLVMAttributeIndex, LLVMAttributeReturnIndex, LLVMContext,
        LLVMModule, LLVMValue,
    };

    use crate::values::get_attributes;

    use super::{add_string_attribute, get_attribute_count};

    fn setup_expect(
        setup: impl Fn(*mut LLVMContext, *mut LLVMModule, *mut LLVMValue),
        expect: impl Fn(*mut LLVMValue),
    ) {
        unsafe {
            let context = LLVMContextCreate();
            let module_name = CString::new("test_module").unwrap();
            let module = LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context);
            let function_name = CString::new("test_func").unwrap();
            let function = LLVMAddFunction(
                module,
                function_name.as_ptr(),
                LLVMVoidTypeInContext(context),
            );
            let builder = LLVMCreateBuilderInContext(context);
            LLVMBuildRetVoid(builder);
            setup(context, module, function);
            expect(function);
            LLVMDisposeModule(module);
            LLVMContextDispose(context);
        }
    }
    #[test]
    fn get_attribute_count_works_when_function_attrs_exist() {
        unsafe {
            setup_expect(
                |_, _, function| {
                    add_string_attribute(function, b"entry_point", b"", LLVMAttributeFunctionIndex);
                    add_string_attribute(
                        function,
                        b"required_num_qubits",
                        b"1",
                        LLVMAttributeFunctionIndex,
                    );
                    add_string_attribute(
                        function,
                        b"required_num_results",
                        b"2",
                        LLVMAttributeFunctionIndex,
                    );
                    add_string_attribute(
                        function,
                        b"qir_profiles",
                        b"test",
                        LLVMAttributeFunctionIndex,
                    );
                },
                |f| {
                    let count = get_attribute_count(f, LLVMAttributeFunctionIndex);
                    assert!(count == 4);
                },
            );
        }
    }
    #[test]
    fn attributes_with_kind_only_have_empty_string_values() {
        unsafe {
            setup_expect(
                |_, _, function| {
                    add_string_attribute(function, b"entry_point", b"", LLVMAttributeFunctionIndex);
                },
                |f| {
                    let count = get_attribute_count(f, LLVMAttributeFunctionIndex);
                    assert!(count == 1);
                    let attrs = get_attributes(f, LLVMAttributeFunctionIndex);
                    for attr in attrs {
                        if let Some(value) = super::get_string_attribute_value(attr) {
                            assert_eq!(value, "");
                        } else {
                            panic!("Should have a value");
                        }
                    }
                },
            );
        }
    }
    #[test]
    fn attributes_with_kind_only_have_key_matching_kind() {
        unsafe {
            setup_expect(
                |_, _, function| {
                    add_string_attribute(function, b"entry_point", b"", LLVMAttributeFunctionIndex);
                },
                |f| {
                    let count = get_attribute_count(f, LLVMAttributeFunctionIndex);
                    assert!(count == 1);
                    let attrs = get_attributes(f, LLVMAttributeFunctionIndex);
                    for attr in attrs {
                        assert_eq!(super::get_string_attribute_kind(attr), "entry_point");
                    }
                },
            );
        }
    }
    #[test]
    fn attributes_with_key_and_value_have_matching_kind_and_value() {
        unsafe {
            setup_expect(
                |_, _, function| {
                    add_string_attribute(
                        function,
                        b"qir_profiles",
                        b"test",
                        LLVMAttributeFunctionIndex,
                    );
                },
                |f| {
                    let count = get_attribute_count(f, LLVMAttributeFunctionIndex);
                    assert!(count == 1);
                    let attrs = get_attributes(f, LLVMAttributeFunctionIndex);
                    for attr in attrs {
                        assert_eq!(super::get_string_attribute_kind(attr), "qir_profiles");
                        assert!(super::get_string_attribute_value(attr).is_some());
                        assert_eq!(super::get_string_attribute_value(attr).unwrap(), "test");
                    }
                },
            );
        }
    }
    #[test]
    fn get_attribute_count_works_when_function_attrs_dont_exist() {
        unsafe {
            setup_expect(
                |_, _, _| {},
                |f| {
                    let count = get_attribute_count(f, LLVMAttributeFunctionIndex);
                    assert!(count == 0);
                },
            );
        }
    }
    #[test]
    fn get_attribute_count_works_when_return_attrs_dont_exist() {
        unsafe {
            setup_expect(
                |_, _, _| {},
                |f| {
                    let count = get_attribute_count(f, LLVMAttributeReturnIndex);
                    assert!(count == 0);
                },
            );
        }
    }
    #[test]
    fn get_attribute_count_works_when_param_attrs_dont_exist() {
        unsafe {
            setup_expect(
                |_, _, _| {},
                |f| {
                    const INVALID_PARAM_ID: LLVMAttributeIndex = 1;
                    let count = get_attribute_count(f, INVALID_PARAM_ID);
                    assert!(count == 0);
                },
            );
        }
    }
    #[test]
    fn iteration_works_when_function_attrs_dont_exist() {
        unsafe {
            setup_expect(
                |_, _, _| {},
                |f| {
                    let attrs = get_attributes(f, LLVMAttributeFunctionIndex);
                    assert!(
                        attrs.into_iter().next().is_none(),
                        "Should not have any attributes"
                    );
                },
            );
        }
    }
    #[test]
    fn iteration_works_when_return_attrs_dont_exist() {
        unsafe {
            setup_expect(
                |_, _, _| {},
                |f| {
                    let attrs = get_attributes(f, LLVMAttributeReturnIndex);
                    assert!(
                        attrs.into_iter().next().is_none(),
                        "Should not have any attributes"
                    );
                },
            );
        }
    }
    #[test]
    fn iteration_works_when_param_attrs_dont_exist() {
        unsafe {
            setup_expect(
                |_, _, _| {},
                |f| {
                    const INVALID_PARAM_ID: LLVMAttributeIndex = 1;
                    let attrs = get_attributes(f, INVALID_PARAM_ID);
                    assert!(
                        attrs.into_iter().next().is_none(),
                        "Should not have any attributes"
                    );
                },
            );
        }
    }
}

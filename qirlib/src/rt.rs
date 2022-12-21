// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    types,
    utils::{build_call, builder_module, declare_external_function, function_type},
};

use llvm_sys::{
    core::{
        LLVMGetModuleContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMPointerType,
        LLVMVoidTypeInContext,
    },
    prelude::*,
};

pub unsafe fn build_array_record_output(
    builder: LLVMBuilderRef,
    num_elements: LLVMValueRef,
    label: LLVMValueRef,
) {
    build_call(
        builder,
        array_record_output(builder_module(builder)),
        &mut [num_elements, label],
    );
}

pub unsafe fn build_initialize(builder: LLVMBuilderRef, data: LLVMValueRef) {
    unsafe {
        build_call(builder, initialize(builder_module(builder)), &mut [data]);
    }
}

pub unsafe fn build_result_record_output(
    builder: LLVMBuilderRef,
    result: LLVMValueRef,
    label: LLVMValueRef,
) {
    unsafe {
        build_call(
            builder,
            result_record_output(builder_module(builder)),
            &mut [result, label],
        );
    }
}

pub unsafe fn build_tuple_record_output(
    builder: LLVMBuilderRef,
    num_elements: LLVMValueRef,
    label: LLVMValueRef,
) {
    unsafe {
        build_call(
            builder,
            tuple_record_output(builder_module(builder)),
            &mut [num_elements, label],
        );
    }
}

unsafe fn array_record_output(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let param_type = LLVMInt64TypeInContext(context);
    let name = "array_record_output";
    record_output(module, name, param_type)
}

unsafe fn initialize(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let i8type = LLVMInt8TypeInContext(context);
    let i8p = LLVMPointerType(i8type, 0);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [i8p]);
    let name = "__quantum__rt__initialize";
    declare_external_function(module, name, ty)
}

unsafe fn result_record_output(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let param_type = types::result(context);
    let name = "result_record_output";
    record_output(module, name, param_type)
}

unsafe fn tuple_record_output(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let param_type = LLVMInt64TypeInContext(context);
    let name = "tuple_record_output";
    record_output(module, name, param_type)
}

unsafe fn record_output(
    module: LLVMModuleRef,
    name: &str,
    param_type: LLVMTypeRef,
) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let i8type = LLVMInt8TypeInContext(context);
    let i8p = LLVMPointerType(i8type, 0);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [param_type, i8p]);
    let name = format!("__quantum__rt__{name}");
    declare_external_function(module, &name, ty)
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use llvm_sys::{
        core::{
            LLVMBasicBlockAsValue, LLVMConstInt, LLVMConstPointerNull, LLVMGetInsertBlock,
            LLVMGetTypeContext, LLVMTypeOf,
        },
        LLVMContext,
    };

    use super::*;
    use crate::{tests::assert_reference_ir, values::result};

    unsafe fn builder_context(builder: LLVMBuilderRef) -> Option<NonNull<LLVMContext>> {
        let block = NonNull::new(LLVMGetInsertBlock(builder))?;
        NonNull::new(LLVMGetTypeContext(LLVMTypeOf(LLVMBasicBlockAsValue(
            block.as_ptr(),
        ))))
    }

    #[test]
    fn array_record_output() {
        assert_reference_ir("rt/array_record_output", 0, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            let i64_ty = LLVMInt64TypeInContext(context);

            let value = LLVMConstInt(i64_ty, 0, 0);

            let i8_ty = LLVMInt8TypeInContext(context);
            let i8_ptr_ty = LLVMPointerType(i8_ty, 0);
            let i8_null_ptr = LLVMConstPointerNull(i8_ptr_ty);

            build_array_record_output(builder, value, i8_null_ptr);
        });
    }

    #[test]
    fn initialize() {
        assert_reference_ir("rt/initialize", 0, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            let i8_ty = LLVMInt8TypeInContext(context);
            let i8_ptr_ty = LLVMPointerType(i8_ty, 0);
            let i8_null_ptr = LLVMConstPointerNull(i8_ptr_ty);
            build_initialize(builder, i8_null_ptr);
        });
    }

    #[test]
    fn result_record_output() {
        assert_reference_ir("rt/result_record_output", 0, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            let i8_ty = LLVMInt8TypeInContext(context);
            let i8_ptr_ty = LLVMPointerType(i8_ty, 0);
            let i8_null_ptr = LLVMConstPointerNull(i8_ptr_ty);
            build_result_record_output(builder, result(context, 0), i8_null_ptr);
        });
    }

    #[test]
    fn tuple_record_output() {
        assert_reference_ir("rt/tuple_record_output", 0, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            let i64_ty = LLVMInt64TypeInContext(context);

            let value = LLVMConstInt(i64_ty, 0, 0);
            let i8_ty = LLVMInt8TypeInContext(context);
            let i8_ptr_ty = LLVMPointerType(i8_ty, 0);
            let i8_null_ptr = LLVMConstPointerNull(i8_ptr_ty);
            build_tuple_record_output(builder, value, i8_null_ptr);
        });
    }
}

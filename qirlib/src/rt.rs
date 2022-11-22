// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    qis::{builder_module, function_type},
    types,
};
use inkwell::{
    builder::Builder,
    values::{IntValue, PointerValue},
    LLVMReference,
};
use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMGetModuleContext, LLVMGetNamedFunction, LLVMInt64TypeInContext,
        LLVMInt8TypeInContext, LLVMPointerType, LLVMSetLinkage, LLVMVoidTypeInContext,
    },
    prelude::*,
    LLVMLinkage,
};

use std::ffi::CString;

use crate::qis::build_call;

pub trait BuilderExt<'ctx> {
    fn build_tuple_record_output(&self, num_elements: IntValue, label: PointerValue);
    fn build_array_record_output(&self, num_elements: IntValue, label: PointerValue);
    fn build_result_record_output(&self, result: PointerValue, label: PointerValue);
}

impl<'ctx> BuilderExt<'ctx> for Builder<'ctx> {
    fn build_tuple_record_output(&self, num_elements: IntValue, label: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                tuple_record_output(builder_module(self.get_ref())),
                &mut [num_elements.get_ref(), label.get_ref()],
            );
        }
    }

    fn build_array_record_output(&self, num_elements: IntValue, label: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                array_record_output(builder_module(self.get_ref())),
                &mut [num_elements.get_ref(), label.get_ref()],
            );
        }
    }

    fn build_result_record_output(&self, result: PointerValue, label: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                result_record_output(builder_module(self.get_ref())),
                &mut [result.get_ref(), label.get_ref()],
            );
        }
    }
}

unsafe fn tuple_record_output(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let param_type = LLVMInt64TypeInContext(context);
    let name = "tuple_record_output";
    record_output(module, name, param_type)
}

unsafe fn array_record_output(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let param_type = LLVMInt64TypeInContext(context);
    let name = "array_record_output";
    record_output(module, name, param_type)
}

unsafe fn result_record_output(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let param_type = types::result_unchecked(context);
    let name = "result_record_output";
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
    let name = format!("__quantum__rt__{}", name);
    declare_bare(module, &name, ty)
}

unsafe fn declare_bare(module: LLVMModuleRef, name: &str, ty: LLVMTypeRef) -> LLVMValueRef {
    let name = CString::new(name).unwrap();
    let function = LLVMGetNamedFunction(module, name.as_ptr().cast());
    if function.is_null() {
        let function = LLVMAddFunction(module, name.as_ptr().cast(), ty);
        LLVMSetLinkage(function, LLVMLinkage::LLVMExternalLinkage);
        function
    } else {
        function
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::assert_reference_ir, values::result};

    #[test]
    fn tuple_record_output() -> Result<(), String> {
        assert_reference_ir("rt/tuple_record_output", 0, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            let value = context.i64_type().const_int(0, false);
            let null = context
                .custom_width_int_type(8)
                .ptr_type(inkwell::AddressSpace::Generic)
                .const_null();
            builder.build_tuple_record_output(value, null);
        })
    }

    #[test]
    fn array_record_output() -> Result<(), String> {
        assert_reference_ir("rt/array_record_output", 0, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            let value = context.i64_type().const_int(0, false);
            let null = context
                .custom_width_int_type(8)
                .ptr_type(inkwell::AddressSpace::Generic)
                .const_null();
            builder.build_array_record_output(value, null);
        })
    }

    #[test]
    fn result_record_output() -> Result<(), String> {
        assert_reference_ir("rt/result_record_output", 0, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            let null = context
                .custom_width_int_type(8)
                .ptr_type(inkwell::AddressSpace::Generic)
                .const_null();
            builder.build_result_record_output(result(&context, 0), null);
        })
    }
}

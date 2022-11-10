// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::{build_if_unchecked, try_build_if_unchecked},
    types,
};
use inkwell::{
    builder::Builder,
    values::{FloatValue, PointerValue},
    LLVMReference,
};
use libc::c_uint;
use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMBuildCall, LLVMDoubleTypeInContext, LLVMFunctionType,
        LLVMGetBasicBlockParent, LLVMGetGlobalParent, LLVMGetInsertBlock, LLVMGetModuleContext,
        LLVMGetNamedFunction, LLVMInt1TypeInContext, LLVMSetLinkage, LLVMVoidTypeInContext,
    },
    prelude::*,
    LLVMLinkage,
};
use std::{ffi::CString, ptr::NonNull};

pub trait BuilderExt<'ctx> {
    fn build_cx(&self, control: PointerValue, qubit: PointerValue);

    fn build_cz(&self, control: PointerValue, qubit: PointerValue);

    fn build_h(&self, qubit: PointerValue);

    fn build_s(&self, qubit: PointerValue);

    fn build_s_adj(&self, qubit: PointerValue);

    fn build_t(&self, qubit: PointerValue);

    fn build_t_adj(&self, qubit: PointerValue);

    fn build_x(&self, qubit: PointerValue);

    fn build_y(&self, qubit: PointerValue);

    fn build_z(&self, qubit: PointerValue);

    fn build_rx(&self, theta: FloatValue, qubit: PointerValue);

    fn build_ry(&self, theta: FloatValue, qubit: PointerValue);

    fn build_rz(&self, theta: FloatValue, qubit: PointerValue);

    fn build_reset(&self, qubit: PointerValue);

    fn build_mz(&self, qubit: PointerValue, result: PointerValue);

    fn build_if_result(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl FnOnce(),
        build_zero: impl FnOnce(),
    );

    #[allow(clippy::missing_errors_doc)]
    fn try_build_if_result<E>(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl FnOnce() -> Result<(), E>,
        build_zero: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), E>;
}

impl<'ctx> BuilderExt<'ctx> for Builder<'ctx> {
    fn build_cx(&self, control: PointerValue, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                controlled_gate(builder_module(self.get_ref()), "cnot"),
                &mut [control.get_ref(), qubit.get_ref()],
            );
        }
    }

    fn build_cz(&self, control: PointerValue, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                controlled_gate(builder_module(self.get_ref()), "cz"),
                &mut [control.get_ref(), qubit.get_ref()],
            );
        }
    }

    fn build_h(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "h", Functor::Body),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_s(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "s", Functor::Body),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_s_adj(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "s", Functor::Adjoint),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_t(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "t", Functor::Body),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_t_adj(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "t", Functor::Adjoint),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_x(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "x", Functor::Body),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_y(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "y", Functor::Body),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_z(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "z", Functor::Body),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_rx(&self, theta: FloatValue, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                rotation_gate(builder_module(self.get_ref()), "rx"),
                &mut [theta.get_ref(), qubit.get_ref()],
            );
        }
    }

    fn build_ry(&self, theta: FloatValue, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                rotation_gate(builder_module(self.get_ref()), "ry"),
                &mut [theta.get_ref(), qubit.get_ref()],
            );
        }
    }

    fn build_rz(&self, theta: FloatValue, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                rotation_gate(builder_module(self.get_ref()), "rz"),
                &mut [theta.get_ref(), qubit.get_ref()],
            );
        }
    }

    fn build_reset(&self, qubit: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                simple_gate(builder_module(self.get_ref()), "reset", Functor::Body),
                &mut [qubit.get_ref()],
            );
        }
    }

    fn build_mz(&self, qubit: PointerValue, result: PointerValue) {
        unsafe {
            build_call(
                self.get_ref(),
                mz(builder_module(self.get_ref())),
                &mut [qubit.get_ref(), result.get_ref()],
            );
        }
    }

    fn build_if_result(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl FnOnce(),
        build_zero: impl FnOnce(),
    ) {
        unsafe {
            let bool_cond = build_read_result(self.get_ref(), cond.get_ref());
            build_if_unchecked(self, bool_cond, build_one, build_zero);
        }
    }

    fn try_build_if_result<E>(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl FnOnce() -> Result<(), E>,
        build_zero: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), E> {
        unsafe {
            let bool_cond = build_read_result(self.get_ref(), cond.get_ref());
            try_build_if_unchecked(self, bool_cond, build_one, build_zero)
        }
    }
}

#[derive(Clone, Copy)]
enum Functor {
    Body,
    Adjoint,
}

unsafe fn build_read_result(builder: LLVMBuilderRef, result: LLVMValueRef) -> LLVMValueRef {
    build_call(builder, read_result(builder_module(builder)), &mut [result])
}

unsafe fn build_call(
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    args: &mut [LLVMValueRef],
) -> LLVMValueRef {
    LLVMBuildCall(
        builder,
        function,
        args.as_mut_ptr(),
        c_uint::try_from(args.len()).unwrap(),
        [0].as_ptr(),
    )
}

unsafe fn builder_module(builder: LLVMBuilderRef) -> LLVMModuleRef {
    NonNull::new(LLVMGetInsertBlock(builder))
        .and_then(|b| NonNull::new(LLVMGetBasicBlockParent(b.as_ptr())))
        .and_then(|v| NonNull::new(LLVMGetGlobalParent(v.as_ptr())))
        .expect("The builder's position has not been set.")
        .as_ptr()
}

unsafe fn simple_gate(module: LLVMModuleRef, name: &str, functor: Functor) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMVoidTypeInContext(context),
        &mut [types::qubit_unchecked(context)],
    );
    declare(module, name, functor, ty)
}

unsafe fn controlled_gate(module: LLVMModuleRef, name: &str) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let qubit = types::qubit_unchecked(context);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [qubit, qubit]);
    declare(module, name, Functor::Body, ty)
}

unsafe fn rotation_gate(module: LLVMModuleRef, name: &str) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMVoidTypeInContext(context),
        &mut [
            LLVMDoubleTypeInContext(context),
            types::qubit_unchecked(context),
        ],
    );
    declare(module, name, Functor::Body, ty)
}

unsafe fn mz(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMVoidTypeInContext(context),
        &mut [
            types::qubit_unchecked(context),
            types::result_unchecked(context),
        ],
    );
    declare(module, "mz", Functor::Body, ty)
}

unsafe fn read_result(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMInt1TypeInContext(context),
        &mut [types::result_unchecked(context)],
    );
    declare(module, "read_result", Functor::Body, ty)
}

unsafe fn declare(
    module: LLVMModuleRef,
    name: &str,
    functor: Functor,
    ty: LLVMTypeRef,
) -> LLVMValueRef {
    let suffix = match functor {
        Functor::Body => "body",
        Functor::Adjoint => "adj",
    };
    let name = CString::new(format!("__quantum__qis__{}__{}", name, suffix)).unwrap();
    let function = LLVMGetNamedFunction(module, name.as_ptr().cast());
    if function.is_null() {
        let function = LLVMAddFunction(module, name.as_ptr().cast(), ty);
        LLVMSetLinkage(function, LLVMLinkage::LLVMExternalLinkage);
        function
    } else {
        function
    }
}

unsafe fn function_type(ret: LLVMTypeRef, params: &mut [LLVMTypeRef]) -> LLVMTypeRef {
    LLVMFunctionType(
        ret,
        params.as_mut_ptr(),
        c_uint::try_from(params.len()).unwrap(),
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tests::assert_reference_ir,
        values::{qubit, result},
    };
    use inkwell::context::Context;

    #[test]
    #[should_panic(expected = "The builder's position has not been set.")]
    fn builder_not_positioned() {
        let context = Context::create();
        let builder = context.create_builder();
        let context = context.void_type().get_context();
        builder.build_x(qubit(&context, 0));
    }

    #[test]
    fn cx() -> Result<(), String> {
        assert_reference_ir("qis/cx", 2, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_cx(qubit(&context, 0), qubit(&context, 1));
        })
    }

    #[test]
    fn cz() -> Result<(), String> {
        assert_reference_ir("qis/cz", 2, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_cz(qubit(&context, 0), qubit(&context, 1));
        })
    }

    #[test]
    fn h() -> Result<(), String> {
        assert_reference_ir("qis/h", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_h(qubit(&context, 0));
        })
    }

    #[test]
    fn s() -> Result<(), String> {
        assert_reference_ir("qis/s", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_s(qubit(&context, 0));
        })
    }

    #[test]
    fn s_adj() -> Result<(), String> {
        assert_reference_ir("qis/s_adj", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_s_adj(qubit(&context, 0));
        })
    }

    #[test]
    fn t() -> Result<(), String> {
        assert_reference_ir("qis/t", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_t(qubit(&context, 0));
        })
    }

    #[test]
    fn t_adj() -> Result<(), String> {
        assert_reference_ir("qis/t_adj", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_t_adj(qubit(&context, 0));
        })
    }

    #[test]
    fn x() -> Result<(), String> {
        assert_reference_ir("qis/x", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_x(qubit(&context, 0));
        })
    }

    #[test]
    fn y() -> Result<(), String> {
        assert_reference_ir("qis/y", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_y(qubit(&context, 0));
        })
    }

    #[test]
    fn z() -> Result<(), String> {
        assert_reference_ir("qis/z", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_z(qubit(&context, 0));
        })
    }

    #[test]
    fn rx() -> Result<(), String> {
        assert_reference_ir("qis/rx", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            let double = context.f64_type();
            builder.build_rx(double.const_float(0.0), qubit(&context, 0));
        })
    }

    #[test]
    fn ry() -> Result<(), String> {
        assert_reference_ir("qis/ry", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            let double = context.f64_type();
            builder.build_ry(double.const_float(0.0), qubit(&context, 0));
        })
    }

    #[test]
    fn rz() -> Result<(), String> {
        assert_reference_ir("qis/rz", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            let double = context.f64_type();
            builder.build_rz(double.const_float(0.0), qubit(&context, 0));
        })
    }

    #[test]
    fn reset() -> Result<(), String> {
        assert_reference_ir("qis/reset", 1, 0, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_reset(qubit(&context, 0));
        })
    }

    #[test]
    fn mz() -> Result<(), String> {
        assert_reference_ir("qis/mz", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
        })
    }

    #[test]
    fn read_result() -> Result<(), String> {
        assert_reference_ir("qis/read_result", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context();
            build_read_result(builder.get_ref(), result(&context, 0).get_ref());
        })
    }

    #[test]
    fn empty_if() -> Result<(), String> {
        assert_reference_ir("qis/empty_if", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(result(&context, 0), || (), || ());
        })
    }

    #[test]
    fn if_then() -> Result<(), String> {
        assert_reference_ir("qis/if_then", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                || builder.build_x(qubit(&context, 0)),
                || (),
            );
        })
    }

    #[test]
    fn if_else() -> Result<(), String> {
        assert_reference_ir("qis/if_else", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                || (),
                || builder.build_x(qubit(&context, 0)),
            );
        })
    }

    #[test]
    fn if_then_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_then_continue", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                || builder.build_x(qubit(&context, 0)),
                || (),
            );
            builder.build_h(qubit(&context, 0));
        })
    }

    #[test]
    fn if_else_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_else_continue", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                || (),
                || builder.build_x(qubit(&context, 0)),
            );
            builder.build_h(qubit(&context, 0));
        })
    }

    #[test]
    fn if_then_else_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_then_else_continue", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                || builder.build_x(qubit(&context, 0)),
                || builder.build_y(qubit(&context, 0)),
            );
            builder.build_h(qubit(&context, 0));
        })
    }

    #[test]
    fn if_then_then() -> Result<(), String> {
        assert_reference_ir("qis/if_then_then", 1, 2, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                || {
                    builder.build_if_result(
                        result(&context, 1),
                        || builder.build_x(qubit(&context, 0)),
                        || (),
                    );
                },
                || (),
            );
        })
    }

    #[test]
    fn if_else_else() -> Result<(), String> {
        assert_reference_ir("qis/if_else_else", 1, 2, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                || (),
                || {
                    builder.build_if_result(
                        result(&context, 1),
                        || (),
                        || builder.build_x(qubit(&context, 0)),
                    );
                },
            );
        })
    }

    #[test]
    fn if_then_else() -> Result<(), String> {
        assert_reference_ir("qis/if_then_else", 1, 2, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                || {
                    builder.build_if_result(
                        result(&context, 1),
                        || (),
                        || builder.build_x(qubit(&context, 0)),
                    );
                },
                || (),
            );
        })
    }

    #[test]
    fn if_else_then() -> Result<(), String> {
        assert_reference_ir("qis/if_else_then", 1, 2, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                || (),
                || {
                    builder.build_if_result(
                        result(&context, 1),
                        || builder.build_x(qubit(&context, 0)),
                        || (),
                    );
                },
            );
        })
    }

    #[test]
    fn if_unmeasured_result() -> Result<(), String> {
        assert_reference_ir("qis/if_unmeasured_result", 1, 1, |builder| {
            let context = builder.get_insert_block().unwrap().get_context();
            builder.build_if_result(
                result(&context, 0),
                || builder.build_x(qubit(&context, 0)),
                || builder.build_h(qubit(&context, 0)),
            );
        })
    }
}

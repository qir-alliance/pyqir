// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::{build_if, try_build_if},
    types,
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

pub unsafe fn build_cx(builder: LLVMBuilderRef, control: LLVMValueRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        controlled_gate(builder_module(builder), "cnot"),
        &mut [control, qubit],
    );
}

pub unsafe fn build_cz(builder: LLVMBuilderRef, control: LLVMValueRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        controlled_gate(builder_module(builder), "cz"),
        &mut [control, qubit],
    );
}

pub unsafe fn build_h(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "h", Functor::Body),
        &mut [qubit],
    );
}

pub unsafe fn build_s(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "s", Functor::Body),
        &mut [qubit],
    );
}

pub unsafe fn build_s_adj(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "s", Functor::Adjoint),
        &mut [qubit],
    );
}

pub unsafe fn build_t(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "t", Functor::Body),
        &mut [qubit],
    );
}

pub unsafe fn build_t_adj(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "t", Functor::Adjoint),
        &mut [qubit],
    );
}

pub unsafe fn build_x(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "x", Functor::Body),
        &mut [qubit],
    );
}

pub unsafe fn build_y(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "y", Functor::Body),
        &mut [qubit],
    );
}

pub unsafe fn build_z(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "z", Functor::Body),
        &mut [qubit],
    );
}

pub unsafe fn build_rx(builder: LLVMBuilderRef, theta: LLVMValueRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        rotation_gate(builder_module(builder), "rx"),
        &mut [theta, qubit],
    );
}

pub unsafe fn build_ry(builder: LLVMBuilderRef, theta: LLVMValueRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        rotation_gate(builder_module(builder), "ry"),
        &mut [theta, qubit],
    );
}

pub unsafe fn build_rz(builder: LLVMBuilderRef, theta: LLVMValueRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        rotation_gate(builder_module(builder), "rz"),
        &mut [theta, qubit],
    );
}

pub unsafe fn build_reset(builder: LLVMBuilderRef, qubit: LLVMValueRef) {
    build_call(
        builder,
        simple_gate(builder_module(builder), "reset", Functor::Body),
        &mut [qubit],
    );
}

pub unsafe fn build_mz(builder: LLVMBuilderRef, qubit: LLVMValueRef, result: LLVMValueRef) {
    build_call(builder, mz(builder_module(builder)), &mut [qubit, result]);
}

pub unsafe fn build_if_result(
    builder: LLVMBuilderRef,
    cond: LLVMValueRef,
    build_one: impl FnOnce(),
    build_zero: impl FnOnce(),
) {
    let bool_cond = build_read_result(builder, cond);
    build_if(builder, bool_cond, build_one, build_zero);
}

pub unsafe fn try_build_if_result<E>(
    builder: LLVMBuilderRef,
    cond: LLVMValueRef,
    build_one: impl FnOnce() -> Result<(), E>,
    build_zero: impl FnOnce() -> Result<(), E>,
) -> Result<(), E> {
    let bool_cond = build_read_result(builder, cond);
    try_build_if(builder, bool_cond, build_one, build_zero)
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
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [types::qubit(context)]);
    declare(module, name, functor, ty)
}

unsafe fn controlled_gate(module: LLVMModuleRef, name: &str) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let qubit = types::qubit(context);
    let ty = function_type(LLVMVoidTypeInContext(context), &mut [qubit, qubit]);
    declare(module, name, Functor::Body, ty)
}

unsafe fn rotation_gate(module: LLVMModuleRef, name: &str) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMVoidTypeInContext(context),
        &mut [LLVMDoubleTypeInContext(context), types::qubit(context)],
    );
    declare(module, name, Functor::Body, ty)
}

unsafe fn mz(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMVoidTypeInContext(context),
        &mut [types::qubit(context), types::result(context)],
    );
    declare(module, "mz", Functor::Body, ty)
}

unsafe fn read_result(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMInt1TypeInContext(context),
        &mut [types::result(context)],
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
    use inkwell::{context::Context, LLVMReference};
    use llvm_sys_130::core::LLVMConstReal;

    #[test]
    #[should_panic(expected = "The builder's position has not been set.")]
    fn builder_not_positioned() {
        let context = Context::create();
        let builder = context.create_builder();
        let context = context.void_type().get_context();
        unsafe {
            build_x(builder.get_ref(), qubit(context.get_ref(), 0));
        }
    }

    #[test]
    fn cx() -> Result<(), String> {
        assert_reference_ir("qis/cx", 2, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_cx(builder.get_ref(), qubit(context, 0), qubit(context, 1));
        })
    }

    #[test]
    fn cz() -> Result<(), String> {
        assert_reference_ir("qis/cz", 2, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_cz(builder.get_ref(), qubit(context, 0), qubit(context, 1));
        })
    }

    #[test]
    fn h() -> Result<(), String> {
        assert_reference_ir("qis/h", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_h(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn s() -> Result<(), String> {
        assert_reference_ir("qis/s", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_s(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn s_adj() -> Result<(), String> {
        assert_reference_ir("qis/s_adj", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_s_adj(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn t() -> Result<(), String> {
        assert_reference_ir("qis/t", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_t(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn t_adj() -> Result<(), String> {
        assert_reference_ir("qis/t_adj", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_t_adj(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn x() -> Result<(), String> {
        assert_reference_ir("qis/x", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_x(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn y() -> Result<(), String> {
        assert_reference_ir("qis/y", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_y(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn z() -> Result<(), String> {
        assert_reference_ir("qis/z", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_z(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn rx() -> Result<(), String> {
        assert_reference_ir("qis/rx", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            let double = LLVMDoubleTypeInContext(context);
            build_rx(
                builder.get_ref(),
                LLVMConstReal(double, 0.0),
                qubit(context, 0),
            );
        })
    }

    #[test]
    fn ry() -> Result<(), String> {
        assert_reference_ir("qis/ry", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            let double = LLVMDoubleTypeInContext(context);
            build_ry(
                builder.get_ref(),
                LLVMConstReal(double, 0.0),
                qubit(context, 0),
            );
        })
    }

    #[test]
    fn rz() -> Result<(), String> {
        assert_reference_ir("qis/rz", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            let double = LLVMDoubleTypeInContext(context);
            build_rz(
                builder.get_ref(),
                LLVMConstReal(double, 0.0),
                qubit(context, 0),
            );
        })
    }

    #[test]
    fn reset() -> Result<(), String> {
        assert_reference_ir("qis/reset", 1, 0, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_reset(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn mz() -> Result<(), String> {
        assert_reference_ir("qis/mz", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
        })
    }

    #[test]
    fn read_result() -> Result<(), String> {
        assert_reference_ir("qis/read_result", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_read_result(builder.get_ref(), result(context, 0));
        })
    }

    #[test]
    fn empty_if() -> Result<(), String> {
        assert_reference_ir("qis/empty_if", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_if_result(builder.get_ref(), result(context, 0), || (), || ());
        })
    }

    #[test]
    fn if_then() -> Result<(), String> {
        assert_reference_ir("qis/if_then", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || build_x(builder.get_ref(), qubit(context, 0)),
                || (),
            );
        })
    }

    #[test]
    fn if_else() -> Result<(), String> {
        assert_reference_ir("qis/if_else", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || (),
                || build_x(builder.get_ref(), qubit(context, 0)),
            );
        })
    }

    #[test]
    fn if_then_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_then_continue", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || build_x(builder.get_ref(), qubit(context, 0)),
                || (),
            );
            build_h(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn if_else_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_else_continue", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || (),
                || build_x(builder.get_ref(), qubit(context, 0)),
            );
            build_h(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn if_then_else_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_then_else_continue", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || build_x(builder.get_ref(), qubit(context, 0)),
                || build_y(builder.get_ref(), qubit(context, 0)),
            );
            build_h(builder.get_ref(), qubit(context, 0));
        })
    }

    #[test]
    fn if_then_then() -> Result<(), String> {
        assert_reference_ir("qis/if_then_then", 1, 2, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 1));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || {
                    build_if_result(
                        builder.get_ref(),
                        result(context, 1),
                        || build_x(builder.get_ref(), qubit(context, 0)),
                        || (),
                    );
                },
                || (),
            );
        })
    }

    #[test]
    fn if_else_else() -> Result<(), String> {
        assert_reference_ir("qis/if_else_else", 1, 2, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 1));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || (),
                || {
                    build_if_result(
                        builder.get_ref(),
                        result(context, 1),
                        || (),
                        || build_x(builder.get_ref(), qubit(context, 0)),
                    );
                },
            );
        })
    }

    #[test]
    fn if_then_else() -> Result<(), String> {
        assert_reference_ir("qis/if_then_else", 1, 2, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 1));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || {
                    build_if_result(
                        builder.get_ref(),
                        result(context, 1),
                        || (),
                        || build_x(builder.get_ref(), qubit(context, 0)),
                    );
                },
                || (),
            );
        })
    }

    #[test]
    fn if_else_then() -> Result<(), String> {
        assert_reference_ir("qis/if_else_then", 1, 2, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 0));
            build_mz(builder.get_ref(), qubit(context, 0), result(context, 1));
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || (),
                || {
                    build_if_result(
                        builder.get_ref(),
                        result(context, 1),
                        || build_x(builder.get_ref(), qubit(context, 0)),
                        || (),
                    );
                },
            );
        })
    }

    #[test]
    fn if_unmeasured_result() -> Result<(), String> {
        assert_reference_ir("qis/if_unmeasured_result", 1, 1, |builder| unsafe {
            let context = builder.get_insert_block().unwrap().get_context().get_ref();
            build_if_result(
                builder.get_ref(),
                result(context, 0),
                || build_x(builder.get_ref(), qubit(context, 0)),
                || build_h(builder.get_ref(), qubit(context, 0)),
            );
        })
    }
}

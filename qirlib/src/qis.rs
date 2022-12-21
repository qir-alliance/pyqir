// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::{build_if, try_build_if},
    types,
    utils::{
        build_call, builder_module, controlled_gate, declare_qis, doubly_controlled_gate,
        function_type, no_param, rotation_gate, simple_gate, two_qubit_gate, Functor,
    },
};

#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, prelude::*};

pub unsafe fn build_barrier(builder: LLVMBuilderRef) {
    build_call(
        builder,
        no_param(builder_module(builder), "barrier", Functor::Body),
        &mut [],
    );
}

pub unsafe fn build_ccx(
    builder: LLVMBuilderRef,
    control1: LLVMValueRef,
    control2: LLVMValueRef,
    qubit: LLVMValueRef,
) {
    build_call(
        builder,
        doubly_controlled_gate(builder_module(builder), "ccx"),
        &mut [control1, control2, qubit],
    );
}

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

pub unsafe fn build_swap(builder: LLVMBuilderRef, qubit1: LLVMValueRef, qubit2: LLVMValueRef) {
    build_call(
        builder,
        two_qubit_gate(builder_module(builder), "swap", Functor::Body),
        &mut [qubit1, qubit2],
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

unsafe fn build_read_result(builder: LLVMBuilderRef, result: LLVMValueRef) -> LLVMValueRef {
    build_call(builder, read_result(builder_module(builder)), &mut [result])
}

unsafe fn mz(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMVoidTypeInContext(context),
        &mut [types::qubit(context), types::result(context)],
    );
    declare_qis(module, "mz", Functor::Body, ty)
}

unsafe fn read_result(module: LLVMModuleRef) -> LLVMValueRef {
    let context = LLVMGetModuleContext(module);
    let ty = function_type(
        LLVMInt1TypeInContext(context),
        &mut [types::result(context)],
    );
    declare_qis(module, "read_result", Functor::Body, ty)
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use super::*;
    use crate::{
        tests::{assert_reference_ir, Builder, Context},
        values::{qubit, result},
    };
    use llvm_sys::{
        core::{LLVMBasicBlockAsValue, LLVMConstReal, LLVMGetTypeContext, LLVMTypeOf},
        LLVMContext,
    };

    unsafe fn builder_context(builder: LLVMBuilderRef) -> Option<NonNull<LLVMContext>> {
        let block = NonNull::new(LLVMGetInsertBlock(builder))?;
        NonNull::new(LLVMGetTypeContext(LLVMTypeOf(LLVMBasicBlockAsValue(
            block.as_ptr(),
        ))))
    }

    #[test]
    #[should_panic(expected = "The builder's position has not been set.")]
    fn builder_not_positioned() {
        unsafe {
            let context = Context::new();
            let builder = Builder::new(&context);
            build_x(builder.as_ptr(), qubit(context.as_ptr(), 0));
        }
    }

    #[test]
    fn barrier() {
        assert_reference_ir("qis/barrier", 0, 0, |builder| unsafe {
            build_barrier(builder);
        });
    }

    #[test]
    fn ccx() {
        assert_reference_ir("qis/ccx", 3, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_ccx(
                builder,
                qubit(context, 0),
                qubit(context, 1),
                qubit(context, 2),
            );
        });
    }

    #[test]
    fn cx() {
        assert_reference_ir("qis/cx", 2, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_cx(builder, qubit(context, 0), qubit(context, 1));
        });
    }

    #[test]
    fn cz() {
        assert_reference_ir("qis/cz", 2, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_cz(builder, qubit(context, 0), qubit(context, 1));
        });
    }

    #[test]
    fn h() {
        assert_reference_ir("qis/h", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_h(builder, qubit(context, 0));
        });
    }

    #[test]
    fn s() {
        assert_reference_ir("qis/s", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_s(builder, qubit(context, 0));
        });
    }

    #[test]
    fn s_adj() {
        assert_reference_ir("qis/s_adj", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_s_adj(builder, qubit(context, 0));
        });
    }

    #[test]
    fn t() {
        assert_reference_ir("qis/t", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_t(builder, qubit(context, 0));
        });
    }

    #[test]
    fn t_adj() {
        assert_reference_ir("qis/t_adj", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_t_adj(builder, qubit(context, 0));
        });
    }

    #[test]
    fn x() {
        assert_reference_ir("qis/x", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_x(builder, qubit(context, 0));
        });
    }

    #[test]
    fn y() {
        assert_reference_ir("qis/y", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_y(builder, qubit(context, 0));
        });
    }

    #[test]
    fn z() {
        assert_reference_ir("qis/z", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_z(builder, qubit(context, 0));
        });
    }

    #[test]
    fn rx() {
        assert_reference_ir("qis/rx", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            let double = LLVMDoubleTypeInContext(context);
            build_rx(builder, LLVMConstReal(double, 0.0), qubit(context, 0));
        });
    }

    #[test]
    fn ry() {
        assert_reference_ir("qis/ry", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            let double = LLVMDoubleTypeInContext(context);
            build_ry(builder, LLVMConstReal(double, 0.0), qubit(context, 0));
        });
    }

    #[test]
    fn rz() {
        assert_reference_ir("qis/rz", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            let double = LLVMDoubleTypeInContext(context);
            build_rz(builder, LLVMConstReal(double, 0.0), qubit(context, 0));
        });
    }

    #[test]
    fn reset() {
        assert_reference_ir("qis/reset", 1, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_reset(builder, qubit(context, 0));
        });
    }

    #[test]
    fn mz() {
        assert_reference_ir("qis/mz", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
        });
    }

    #[test]
    fn read_result() {
        assert_reference_ir("qis/read_result", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_read_result(builder, result(context, 0));
        });
    }

    #[test]
    fn swap() {
        assert_reference_ir("qis/swap", 2, 0, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_swap(builder, qubit(context, 0), qubit(context, 1));
        });
    }

    #[test]
    fn empty_if() {
        assert_reference_ir("qis/empty_if", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_if_result(builder, result(context, 0), || (), || ());
        });
    }

    #[test]
    fn if_then() {
        assert_reference_ir("qis/if_then", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_if_result(
                builder,
                result(context, 0),
                || build_x(builder, qubit(context, 0)),
                || (),
            );
        });
    }

    #[test]
    fn if_else() {
        assert_reference_ir("qis/if_else", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_if_result(
                builder,
                result(context, 0),
                || (),
                || build_x(builder, qubit(context, 0)),
            );
        });
    }

    #[test]
    fn if_then_continue() {
        assert_reference_ir("qis/if_then_continue", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_if_result(
                builder,
                result(context, 0),
                || build_x(builder, qubit(context, 0)),
                || (),
            );
            build_h(builder, qubit(context, 0));
        });
    }

    #[test]
    fn if_else_continue() {
        assert_reference_ir("qis/if_else_continue", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_if_result(
                builder,
                result(context, 0),
                || (),
                || build_x(builder, qubit(context, 0)),
            );
            build_h(builder, qubit(context, 0));
        });
    }

    #[test]
    fn if_then_else_continue() {
        assert_reference_ir("qis/if_then_else_continue", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_if_result(
                builder,
                result(context, 0),
                || build_x(builder, qubit(context, 0)),
                || build_y(builder, qubit(context, 0)),
            );
            build_h(builder, qubit(context, 0));
        });
    }

    #[test]
    fn if_then_then() {
        assert_reference_ir("qis/if_then_then", 1, 2, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_mz(builder, qubit(context, 0), result(context, 1));
            build_if_result(
                builder,
                result(context, 0),
                || {
                    build_if_result(
                        builder,
                        result(context, 1),
                        || build_x(builder, qubit(context, 0)),
                        || (),
                    );
                },
                || (),
            );
        });
    }

    #[test]
    fn if_else_else() {
        assert_reference_ir("qis/if_else_else", 1, 2, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_mz(builder, qubit(context, 0), result(context, 1));
            build_if_result(
                builder,
                result(context, 0),
                || (),
                || {
                    build_if_result(
                        builder,
                        result(context, 1),
                        || (),
                        || build_x(builder, qubit(context, 0)),
                    );
                },
            );
        });
    }

    #[test]
    fn if_then_else() {
        assert_reference_ir("qis/if_then_else", 1, 2, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_mz(builder, qubit(context, 0), result(context, 1));
            build_if_result(
                builder,
                result(context, 0),
                || {
                    build_if_result(
                        builder,
                        result(context, 1),
                        || (),
                        || build_x(builder, qubit(context, 0)),
                    );
                },
                || (),
            );
        });
    }

    #[test]
    fn if_else_then() {
        assert_reference_ir("qis/if_else_then", 1, 2, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_mz(builder, qubit(context, 0), result(context, 0));
            build_mz(builder, qubit(context, 0), result(context, 1));
            build_if_result(
                builder,
                result(context, 0),
                || (),
                || {
                    build_if_result(
                        builder,
                        result(context, 1),
                        || build_x(builder, qubit(context, 0)),
                        || (),
                    );
                },
            );
        });
    }

    #[test]
    fn if_unmeasured_result() {
        assert_reference_ir("qis/if_unmeasured_result", 1, 1, |builder| unsafe {
            let context = builder_context(builder).unwrap().as_ptr();
            build_if_result(
                builder,
                result(context, 0),
                || build_x(builder, qubit(context, 0)),
                || build_h(builder, qubit(context, 0)),
            );
        });
    }
}

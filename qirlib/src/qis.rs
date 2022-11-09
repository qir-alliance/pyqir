// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{builder::Builder, types};
use inkwell::{
    module::{Linkage, Module},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{FloatValue, FunctionValue, IntValue, PointerValue},
};

pub trait BuilderBasicQisExt<'ctx> {
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
        build_one: impl FnOnce(&Self),
        build_zero: impl FnOnce(&Self),
    );

    #[allow(clippy::missing_errors_doc)]
    fn try_build_if_result<E>(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl FnOnce(&Self) -> Result<(), E>,
        build_zero: impl FnOnce(&Self) -> Result<(), E>,
    ) -> Result<(), E>;
}

impl<'ctx, 'a> BuilderBasicQisExt<'ctx> for Builder<'ctx, 'a> {
    fn build_cx(&self, control: PointerValue, qubit: PointerValue) {
        let function = controlled_gate(self.module(), "cnot");
        self.build_call(function, &[control.into(), qubit.into()], "");
    }

    fn build_cz(&self, control: PointerValue, qubit: PointerValue) {
        let function = controlled_gate(self.module(), "cz");
        self.build_call(function, &[control.into(), qubit.into()], "");
    }

    fn build_h(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "h", Functor::Body);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_s(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "s", Functor::Body);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_s_adj(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "s", Functor::Adjoint);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_t(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "t", Functor::Body);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_t_adj(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "t", Functor::Adjoint);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_x(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "x", Functor::Body);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_y(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "y", Functor::Body);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_z(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "z", Functor::Body);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_rx(&self, theta: FloatValue, qubit: PointerValue) {
        let function = rotation_gate(self.module(), "rx");
        self.build_call(function, &[theta.into(), qubit.into()], "");
    }

    fn build_ry(&self, theta: FloatValue, qubit: PointerValue) {
        let function = rotation_gate(self.module(), "ry");
        self.build_call(function, &[theta.into(), qubit.into()], "");
    }

    fn build_rz(&self, theta: FloatValue, qubit: PointerValue) {
        let function = rotation_gate(self.module(), "rz");
        self.build_call(function, &[theta.into(), qubit.into()], "");
    }

    fn build_reset(&self, qubit: PointerValue) {
        let function = simple_gate(self.module(), "reset", Functor::Body);
        self.build_call(function, &[qubit.into()], "");
    }

    fn build_mz(&self, qubit: PointerValue, result: PointerValue) {
        self.build_call(mz(self.module()), &[qubit.into(), result.into()], "");
    }

    fn build_if_result(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl FnOnce(&Self),
        build_zero: impl FnOnce(&Self),
    ) {
        let bool_cond = build_read_result(self, cond);
        self.build_if(bool_cond, build_one, build_zero);
    }

    fn try_build_if_result<E>(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl FnOnce(&Self) -> Result<(), E>,
        build_zero: impl FnOnce(&Self) -> Result<(), E>,
    ) -> Result<(), E> {
        let bool_cond = build_read_result(self, cond);
        self.try_build_if(bool_cond, build_one, build_zero)
    }
}

#[derive(Clone, Copy)]
enum Functor {
    Body,
    Adjoint,
}

fn build_read_result<'ctx>(
    builder: &Builder<'ctx, '_>,
    result: PointerValue<'ctx>,
) -> IntValue<'ctx> {
    builder
        .build_call(read_result(builder.module()), &[result.into()], "")
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value()
}

fn simple_gate<'ctx>(module: &Module<'ctx>, name: &str, functor: Functor) -> FunctionValue<'ctx> {
    let qubit_type = types::qubit(&module.get_context()).into();
    declare(module, name, functor, None, &[qubit_type])
}

fn controlled_gate<'ctx>(module: &Module<'ctx>, name: &str) -> FunctionValue<'ctx> {
    let qubit_type = types::qubit(&module.get_context()).into();
    declare(module, name, Functor::Body, None, &[qubit_type, qubit_type])
}

fn rotation_gate<'ctx>(module: &Module<'ctx>, name: &str) -> FunctionValue<'ctx> {
    let double_type = module.get_context().f64_type().into();
    let qubit_type = types::qubit(&module.get_context()).into();
    declare(
        module,
        name,
        Functor::Body,
        None,
        &[double_type, qubit_type],
    )
}

fn mz<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let context = module.get_context();
    let qubit_type = types::qubit(&context).into();
    let result_type = types::result(&context).into();
    declare(
        module,
        "mz",
        Functor::Body,
        None,
        &[qubit_type, result_type],
    )
}

fn read_result<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let bool_type = module.get_context().bool_type().into();
    let result_type = types::result(&module.get_context()).into();
    declare(
        module,
        "read_result",
        Functor::Body,
        Some(bool_type),
        &[result_type],
    )
}

fn declare<'ctx>(
    module: &Module<'ctx>,
    name: &str,
    functor: Functor,
    return_type: Option<BasicTypeEnum<'ctx>>,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionValue<'ctx> {
    let name = format!(
        "__quantum__qis__{}__{}",
        name,
        match functor {
            Functor::Body => "body",
            Functor::Adjoint => "adj",
        }
    );

    module.get_function(&name).unwrap_or_else(|| {
        log::debug!("{} global function was not defined in the module", name);
        let ty = match return_type {
            Some(ty) => ty.fn_type(param_types, false),
            None => module.get_context().void_type().fn_type(param_types, false),
        };
        module.add_function(&name, ty, Some(Linkage::External))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tests::assert_reference_ir,
        values::{qubit, result},
    };

    #[test]
    fn cx() -> Result<(), String> {
        assert_reference_ir("qis/cx", 2, 0, |builder| {
            let context = builder.module().get_context();
            builder.build_cx(qubit(&context, 0), qubit(&context, 1));
        })
    }

    #[test]
    fn cz() -> Result<(), String> {
        assert_reference_ir("qis/cz", 2, 0, |builder| {
            let context = builder.module().get_context();
            builder.build_cz(qubit(&context, 0), qubit(&context, 1));
        })
    }

    #[test]
    fn h() -> Result<(), String> {
        assert_reference_ir("qis/h", 1, 0, |builder| {
            builder.build_h(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn s() -> Result<(), String> {
        assert_reference_ir("qis/s", 1, 0, |builder| {
            builder.build_s(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn s_adj() -> Result<(), String> {
        assert_reference_ir("qis/s_adj", 1, 0, |builder| {
            builder.build_s_adj(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn t() -> Result<(), String> {
        assert_reference_ir("qis/t", 1, 0, |builder| {
            builder.build_t(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn t_adj() -> Result<(), String> {
        assert_reference_ir("qis/t_adj", 1, 0, |builder| {
            builder.build_t_adj(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn x() -> Result<(), String> {
        assert_reference_ir("qis/x", 1, 0, |builder| {
            builder.build_x(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn y() -> Result<(), String> {
        assert_reference_ir("qis/y", 1, 0, |builder| {
            builder.build_y(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn z() -> Result<(), String> {
        assert_reference_ir("qis/z", 1, 0, |builder| {
            builder.build_z(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn rx() -> Result<(), String> {
        assert_reference_ir("qis/rx", 1, 0, |builder| {
            let context = builder.module().get_context();
            let double_type = context.f64_type();
            builder.build_rx(double_type.const_float(0.0), qubit(&context, 0));
        })
    }

    #[test]
    fn ry() -> Result<(), String> {
        assert_reference_ir("qis/ry", 1, 0, |builder| {
            let context = builder.module().get_context();
            let double_type = context.f64_type();
            builder.build_ry(double_type.const_float(0.0), qubit(&context, 0));
        })
    }

    #[test]
    fn rz() -> Result<(), String> {
        assert_reference_ir("qis/rz", 1, 0, |builder| {
            let context = builder.module().get_context();
            let double_type = context.f64_type();
            builder.build_rz(double_type.const_float(0.0), qubit(&context, 0));
        })
    }

    #[test]
    fn reset() -> Result<(), String> {
        assert_reference_ir("qis/reset", 1, 0, |builder| {
            builder.build_reset(qubit(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn mz() -> Result<(), String> {
        assert_reference_ir("qis/mz", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
        })
    }

    #[test]
    fn read_result() -> Result<(), String> {
        assert_reference_ir("qis/read_result", 1, 1, |builder| {
            build_read_result(builder, result(&builder.module().get_context(), 0));
        })
    }

    #[test]
    fn empty_if() -> Result<(), String> {
        assert_reference_ir("qis/empty_if", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(result(&context, 0), |_| (), |_| ());
        })
    }

    #[test]
    fn if_then() -> Result<(), String> {
        assert_reference_ir("qis/if_then", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                |builder| builder.build_x(qubit(&context, 0)),
                |_| (),
            );
        })
    }

    #[test]
    fn if_else() -> Result<(), String> {
        assert_reference_ir("qis/if_else", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                |_| (),
                |builder| builder.build_x(qubit(&context, 0)),
            );
        })
    }

    #[test]
    fn if_then_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_then_continue", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                |builder| builder.build_x(qubit(&context, 0)),
                |_| (),
            );
            builder.build_h(qubit(&context, 0));
        })
    }

    #[test]
    fn if_else_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_else_continue", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                |_| (),
                |builder| builder.build_x(qubit(&context, 0)),
            );
            builder.build_h(qubit(&context, 0));
        })
    }

    #[test]
    fn if_then_else_continue() -> Result<(), String> {
        assert_reference_ir("qis/if_then_else_continue", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_if_result(
                result(&context, 0),
                |builder| builder.build_x(qubit(&context, 0)),
                |builder| builder.build_y(qubit(&context, 0)),
            );
            builder.build_h(qubit(&context, 0));
        })
    }

    #[test]
    fn if_then_then() -> Result<(), String> {
        assert_reference_ir("qis/if_then_then", 1, 2, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                |builder| {
                    builder.build_if_result(
                        result(&context, 1),
                        |builder| builder.build_x(qubit(&context, 0)),
                        |_| (),
                    );
                },
                |_| (),
            );
        })
    }

    #[test]
    fn if_else_else() -> Result<(), String> {
        assert_reference_ir("qis/if_else_else", 1, 2, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                |_| (),
                |builder| {
                    builder.build_if_result(
                        result(&context, 1),
                        |_| (),
                        |builder| builder.build_x(qubit(&context, 0)),
                    );
                },
            );
        })
    }

    #[test]
    fn if_then_else() -> Result<(), String> {
        assert_reference_ir("qis/if_then_else", 1, 2, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                |builder| {
                    builder.build_if_result(
                        result(&context, 1),
                        |_| (),
                        |builder| builder.build_x(qubit(&context, 0)),
                    );
                },
                |_| (),
            );
        })
    }

    #[test]
    fn if_else_then() -> Result<(), String> {
        assert_reference_ir("qis/if_else_then", 1, 2, |builder| {
            let context = builder.module().get_context();
            builder.build_mz(qubit(&context, 0), result(&context, 0));
            builder.build_mz(qubit(&context, 0), result(&context, 1));
            builder.build_if_result(
                result(&context, 0),
                |_| (),
                |builder| {
                    builder.build_if_result(
                        result(&context, 1),
                        |builder| builder.build_x(qubit(&context, 0)),
                        |_| (),
                    );
                },
            );
        })
    }

    #[test]
    fn if_unmeasured_result() -> Result<(), String> {
        assert_reference_ir("qis/if_unmeasured_result", 1, 1, |builder| {
            let context = builder.module().get_context();
            builder.build_if_result(
                result(&context, 0),
                |builder| builder.build_x(qubit(&context, 0)),
                |builder| builder.build_h(qubit(&context, 0)),
            );
        })
    }
}

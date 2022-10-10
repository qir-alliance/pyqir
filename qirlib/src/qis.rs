// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{builder::ModuleBuilder, types};
use crate::builder::BuilderExt;
use inkwell::{
    builder::Builder,
    module::{Linkage, Module},
    types::BasicMetadataTypeEnum,
    values::{FloatValue, FunctionValue, IntValue, PointerValue},
};
use log;
use std::borrow::Borrow;

pub trait BuilderBasicExt<'ctx> {
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
        build_one: impl Fn(),
        build_zero: impl Fn(),
    );

    #[allow(clippy::missing_errors_doc)]
    fn try_build_if_result<E>(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl Fn() -> Result<(), E>,
        build_zero: impl Fn() -> Result<(), E>,
    ) -> Result<(), E>;
}

impl<'ctx, 'm, B: Borrow<Builder<'ctx>>> BuilderBasicExt<'ctx> for ModuleBuilder<'ctx, 'm, B> {
    fn build_cx(&self, control: PointerValue, qubit: PointerValue) {
        self.build_call(
            cnot_body(self.module()),
            &[control.into(), qubit.into()],
            "",
        );
    }

    fn build_cz(&self, control: PointerValue, qubit: PointerValue) {
        self.build_call(cz_body(self.module()), &[control.into(), qubit.into()], "");
    }

    fn build_h(&self, qubit: PointerValue) {
        self.build_call(h_body(self.module()), &[qubit.into()], "");
    }

    fn build_s(&self, qubit: PointerValue) {
        self.build_call(s_body(self.module()), &[qubit.into()], "");
    }

    fn build_s_adj(&self, qubit: PointerValue) {
        self.build_call(s_adj(self.module()), &[qubit.into()], "");
    }

    fn build_t(&self, qubit: PointerValue) {
        self.build_call(t_body(self.module()), &[qubit.into()], "");
    }

    fn build_t_adj(&self, qubit: PointerValue) {
        self.build_call(t_adj(self.module()), &[qubit.into()], "");
    }

    fn build_x(&self, qubit: PointerValue) {
        self.build_call(x_body(self.module()), &[qubit.into()], "");
    }

    fn build_y(&self, qubit: PointerValue) {
        self.build_call(y_body(self.module()), &[qubit.into()], "");
    }

    fn build_z(&self, qubit: PointerValue) {
        self.build_call(z_body(self.module()), &[qubit.into()], "");
    }

    fn build_rx(&self, theta: FloatValue, qubit: PointerValue) {
        self.build_call(rx_body(self.module()), &[theta.into(), qubit.into()], "");
    }

    fn build_ry(&self, theta: FloatValue, qubit: PointerValue) {
        self.build_call(ry_body(self.module()), &[theta.into(), qubit.into()], "");
    }

    fn build_rz(&self, theta: FloatValue, qubit: PointerValue) {
        self.build_call(rz_body(self.module()), &[theta.into(), qubit.into()], "");
    }

    fn build_reset(&self, qubit: PointerValue) {
        self.build_call(reset_body(self.module()), &[qubit.into()], "");
    }

    fn build_mz(&self, qubit: PointerValue, result: PointerValue) {
        self.build_call(mz_body(self.module()), &[qubit.into(), result.into()], "");
    }

    fn build_if_result(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl Fn(),
        build_zero: impl Fn(),
    ) {
        let bool_cond = build_read_result(self, cond);
        self.build_if(bool_cond, build_one, build_zero);
    }

    fn try_build_if_result<E>(
        &self,
        cond: PointerValue<'ctx>,
        build_one: impl Fn() -> Result<(), E>,
        build_zero: impl Fn() -> Result<(), E>,
    ) -> Result<(), E> {
        let bool_cond = build_read_result(self, cond);
        self.try_build_if(bool_cond, build_one, build_zero)
    }
}

fn build_read_result<'ctx>(
    builder: &ModuleBuilder<'ctx, '_, impl Borrow<Builder<'ctx>>>,
    result: PointerValue<'ctx>,
) -> IntValue<'ctx> {
    builder
        .build_call(read_result(builder.module()), &[result.into()], "")
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value()
}

fn cnot_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_controlled_intrinsic_function_body(module, "cnot")
}

fn cz_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_controlled_intrinsic_function_body(module, "cz")
}

fn h_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(module, "h")
}

fn s_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(module, "s")
}

fn s_adj<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_adj(module, "s")
}

fn t_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(module, "t")
}

fn t_adj<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_adj(module, "t")
}

fn x_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(module, "x")
}

fn y_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(module, "y")
}

fn z_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(module, "z")
}

fn rx_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(module, "rx")
}

fn ry_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(module, "ry")
}

fn rz_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(module, "rz")
}

fn reset_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(module, "reset")
}

fn mz_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_mz_function_body(module, "mz")
}

/// `declare void @__quantum__qis__{}__body(%Qubit*, %Qubit*)`
fn get_controlled_intrinsic_function_body<'ctx>(
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let qubit_ptr_type = types::qubit(module);
    get_intrinsic_function_body_impl(
        module,
        name,
        &[qubit_ptr_type.into(), qubit_ptr_type.into()],
    )
}

/// `declare void @__quantum__qis__{}__body(double, %Qubit*)`
fn get_rotated_intrinsic_function_body<'ctx>(
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let qubit_ptr_type = types::qubit(module);
    get_intrinsic_function_body_impl(
        module,
        name,
        &[
            module.get_context().f64_type().into(),
            qubit_ptr_type.into(),
        ],
    )
}

/// `declare void @__quantum__qis__{}__body(%Qubit*)`
fn get_intrinsic_function_body<'ctx>(module: &Module<'ctx>, name: &str) -> FunctionValue<'ctx> {
    let qubit_ptr_type = types::qubit(module);
    get_intrinsic_function_body_impl(module, name, &[qubit_ptr_type.into()])
}

fn get_intrinsic_function_body_impl<'ctx>(
    module: &Module<'ctx>,
    name: &str,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let void_type = module.get_context().void_type();
        let fn_type = void_type.fn_type(param_types, false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare void @__quantum__qis__{}__body(%Qubit*, %Result*)`
fn get_intrinsic_mz_function_body<'ctx>(module: &Module<'ctx>, name: &str) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let result_ptr_type = types::result(module);
        let qubit_ptr_type = types::qubit(module);
        let void_type = module.get_context().void_type();
        let fn_type = void_type.fn_type(&[qubit_ptr_type.into(), result_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare void @__quantum__qis__{}__adj(%Qubit*)`
fn get_intrinsic_function_adj<'ctx>(module: &Module<'ctx>, name: &str) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__adj", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let void_type = module.get_context().void_type();
        let qubit_ptr_type = types::qubit(module);
        let fn_type = void_type.fn_type(&[qubit_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

fn get_function<'ctx>(module: &Module<'ctx>, function_name: &str) -> Option<FunctionValue<'ctx>> {
    let defined_function = module.get_function(function_name);
    match defined_function {
        None => {
            log::debug!(
                "{} global function was not defined in the module",
                function_name
            );
            None
        }
        Some(value) => Some(value),
    }
}

/// `declare i1 @__quantum__qis__read_result__body(%Result*)`
fn read_result<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", "read_result");
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let result_ptr_type = types::result(module);
        let bool_type = module.get_context().bool_type();
        let fn_type = bool_type.fn_type(&[result_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

#[cfg(test)]
mod qis_declaration_tests {
    use super::*;
    use inkwell::{context::Context, values::AnyValue};

    #[test]
    fn cnot_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = cnot_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__cnot__body(%Qubit*, %Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn cz_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = cz_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn h_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = h_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__h__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn s_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = s_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__s__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn s_adj_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = s_adj(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__s__adj(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn t_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = t_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__t__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn t_adj_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = t_adj(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__t__adj(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn x_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = x_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__x__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn y_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = y_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__y__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn z_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = z_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__z__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn rx_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = rx_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__rx__body(double, %Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn ry_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = ry_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__ry__body(double, %Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn rz_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = rz_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__rz__body(double, %Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn reset_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = reset_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__reset__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn mz_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = mz_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__mz__body(%Qubit*, %Result*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn read_result_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = read_result(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare i1 @__quantum__qis__read_result__body(%Result*)\n",
            str_val.to_string()
        );
    }
}
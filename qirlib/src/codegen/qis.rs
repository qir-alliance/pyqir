// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::types::BasicMetadataTypeEnum;
use inkwell::AddressSpace;
use log;

use inkwell::module::{Linkage, Module};
use inkwell::values::FunctionValue;

use super::types::{self, double, qubit, result};

pub fn cnot_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_controlled_intrinsic_function_body(context, module, "cnot")
}

pub fn cz_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_controlled_intrinsic_function_body(context, module, "cz")
}

pub fn h_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "h")
}

pub fn s_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "s")
}

pub fn s_adj<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_adj(context, module, "s")
}

pub fn t_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "t")
}

pub fn t_adj<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_adj(context, module, "t")
}

pub fn x_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "x")
}

pub fn y_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "y")
}

pub fn z_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "z")
}

pub fn rx_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(context, module, "rx")
}

pub fn ry_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(context, module, "ry")
}

pub fn rz_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(context, module, "rz")
}

pub fn reset_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "reset")
}

pub fn mz_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_mz_function_body(context, module, "mz")
}

pub(crate) fn m_body<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    get_intrinsic_m_function_body(module, "m")
}

/// `declare void @__quantum__qis__{}__body(%Qubit*, %Qubit*)`
pub(crate) fn get_controlled_intrinsic_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let qubit_ptr_type = qubit(module).ptr_type(AddressSpace::Generic);
    get_intrinsic_function_body_impl(
        context,
        module,
        name,
        &[qubit_ptr_type.into(), qubit_ptr_type.into()],
    )
}

/// `declare void @__quantum__qis__{}__body(double, %Qubit*)`
pub(crate) fn get_rotated_intrinsic_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let qubit_ptr_type = qubit(module).ptr_type(AddressSpace::Generic);
    get_intrinsic_function_body_impl(
        context,
        module,
        name,
        &[double(context).into(), qubit_ptr_type.into()],
    )
}

/// `declare void @__quantum__qis__{}__body(%Qubit*)`
pub(crate) fn get_intrinsic_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let qubit_ptr_type = qubit(module).ptr_type(AddressSpace::Generic);
    get_intrinsic_function_body_impl(context, module, name, &[qubit_ptr_type.into()])
}

fn get_intrinsic_function_body_impl<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let void_type = context.void_type();
        let fn_type = void_type.fn_type(param_types, false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare void @__quantum__qis__{}__body(%Qubit*, %Result*)`
pub(crate) fn get_intrinsic_mz_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let result_ptr_type = result(module).ptr_type(AddressSpace::Generic);
        let qubit_ptr_type = qubit(module).ptr_type(AddressSpace::Generic);
        let void_type = context.void_type();
        let fn_type = void_type.fn_type(&[qubit_ptr_type.into(), result_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare %Result* @__quantum__qis__{}__body(%Qubit*)`
pub(crate) fn get_intrinsic_m_function_body<'ctx>(
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let result_ptr_type = result(module).ptr_type(AddressSpace::Generic);
        let qubit_ptr_type = qubit(module).ptr_type(AddressSpace::Generic);
        let fn_type = result_ptr_type.fn_type(&[qubit_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare void @__quantum__qis__{}__adj(%Qubit*)`
pub(crate) fn get_intrinsic_function_adj<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__adj", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let void_type = context.void_type();
        let qubit_ptr_type = qubit(module).ptr_type(AddressSpace::Generic);
        let fn_type = void_type.fn_type(&[qubit_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

pub(crate) fn get_function<'ctx>(
    module: &Module<'ctx>,
    function_name: &str,
) -> Option<FunctionValue<'ctx>> {
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
pub fn read_result<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", "read_result");
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let result_ptr_type = result(module).ptr_type(AddressSpace::Generic);
        let bool_type = types::bool(context);
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
        let function = cnot_body(&context, &module);
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
        let function = cz_body(&context, &module);
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
        let function = h_body(&context, &module);
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
        let function = s_body(&context, &module);
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
        let function = s_adj(&context, &module);
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
        let function = t_body(&context, &module);
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
        let function = t_adj(&context, &module);
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
        let function = x_body(&context, &module);
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
        let function = y_body(&context, &module);
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
        let function = z_body(&context, &module);
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
        let function = rx_body(&context, &module);
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
        let function = ry_body(&context, &module);
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
        let function = rz_body(&context, &module);
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
        let function = reset_body(&context, &module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare void @__quantum__qis__reset__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn m_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = m_body(&module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare %Result* @__quantum__qis__m__body(%Qubit*)\n",
            str_val.to_string()
        );
    }

    #[test]
    fn mz_is_declared_correctly() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = mz_body(&context, &module);
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
        let function = read_result(&context, &module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare i1 @__quantum__qis__read_result__body(%Result*)\n",
            str_val.to_string()
        );
    }
}

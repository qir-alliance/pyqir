// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::AddressSpace;
use log;

use inkwell::module::{Linkage, Module};
use inkwell::values::FunctionValue;

use super::types::{double, qubit, result};

pub(crate) fn cnot_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_controlled_intrinsic_function_body(context, module, "cnot")
}

pub(crate) fn cz_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_controlled_intrinsic_function_body(context, module, "cz")
}

pub(crate) fn h_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "h")
}

pub(crate) fn s_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "s")
}

pub(crate) fn s_adj<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_adj(context, module, "s")
}

pub(crate) fn t_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "t")
}

pub(crate) fn t_adj<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_adj(context, module, "t")
}

pub(crate) fn x_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "x")
}

pub(crate) fn y_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "y")
}

pub(crate) fn z_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "z")
}

pub(crate) fn rx_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(context, module, "rx")
}

pub(crate) fn ry_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(context, module, "ry")
}

pub(crate) fn rz_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_rotated_intrinsic_function_body(context, module, "rz")
}

pub(crate) fn reset_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_function_body(context, module, "reset")
}

pub(crate) fn m_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    get_intrinsic_m_function_body(context, module, "m")
}

/// `declare void @__quantum__qis__{}__body(%Qubit*, %Qubit*)`
pub(crate) fn get_controlled_intrinsic_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let void_type = context.void_type();
        let qubit_ptr_type = qubit(context, module).ptr_type(AddressSpace::Generic);
        let fn_type = void_type.fn_type(&[qubit_ptr_type.into(), qubit_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare void @__quantum__qis__{}__body(double, %Qubit*)`
pub(crate) fn get_rotated_intrinsic_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let void_type = context.void_type();
        let qubit_ptr_type = qubit(context, module).ptr_type(AddressSpace::Generic);
        let fn_type = void_type.fn_type(&[double(context).into(), qubit_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare void @__quantum__qis__{}__body(%Qubit*)`
pub(crate) fn get_intrinsic_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let void_type = context.void_type();
        let qubit_ptr_type = qubit(context, module).ptr_type(AddressSpace::Generic);
        let fn_type = void_type.fn_type(&[qubit_ptr_type.into()], false);
        let fn_value =
            module.add_function(function_name.as_str(), fn_type, Some(Linkage::External));
        fn_value
    }
}

/// `declare %Result* @__quantum__qis__{}__body(%Qubit*)`
pub(crate) fn get_intrinsic_m_function_body<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
    name: &str,
) -> FunctionValue<'ctx> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    if let Some(function) = get_function(module, function_name.as_str()) {
        function
    } else {
        let result_ptr_type = result(context, module).ptr_type(AddressSpace::Generic);
        let qubit_ptr_type = qubit(context, module).ptr_type(AddressSpace::Generic);
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
        let qubit_ptr_type = qubit(context, module).ptr_type(AddressSpace::Generic);
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
        let function = m_body(&context, &module);
        let str_val = function.print_to_string();
        assert_eq!(
            "declare %Result* @__quantum__qis__m__body(%Qubit*)\n",
            str_val.to_string()
        );
    }
}

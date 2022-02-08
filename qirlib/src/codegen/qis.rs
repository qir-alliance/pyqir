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

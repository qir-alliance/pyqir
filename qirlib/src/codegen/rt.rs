// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    module::{Linkage, Module},
    values::FunctionValue,
    AddressSpace,
};

use super::types::{self, qubit, result};

/// `declare %Result* @__quantum__rt__result_get_zero()`
pub(crate) fn result_get_zero<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    if let Some(function) = get_function(module, "result_get_zero") {
        function
    } else {
        let result_ptr_type = result(context, module).ptr_type(AddressSpace::Generic);

        let fn_type = result_ptr_type.fn_type(&[], false);
        let fn_value = module.add_function(
            get_function_name("result_get_zero").as_str(),
            fn_type,
            Some(Linkage::External),
        );
        fn_value
    }
}

/// `declare %Result* @__quantum__rt__result_get_one()`
pub(crate) fn result_get_one<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    if let Some(function) = get_function(module, "result_get_one") {
        function
    } else {
        let result_ptr_type = result(context, module).ptr_type(AddressSpace::Generic);

        let fn_type = result_ptr_type.fn_type(&[], false);
        let fn_value = module.add_function(
            get_function_name("result_get_one").as_str(),
            fn_type,
            Some(Linkage::External),
        );
        fn_value
    }
}

/// `declare i1 @__quantum__rt__result_equal(%Result*, %Result*)`
pub(crate) fn result_equal<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    if let Some(function) = get_function(module, "result_equal") {
        function
    } else {
        let result_ptr_type = result(context, module).ptr_type(AddressSpace::Generic);
        let bool_type = types::bool(context);
        let fn_type = bool_type.fn_type(&[result_ptr_type.into(), result_ptr_type.into()], false);
        let fn_value = module.add_function(
            get_function_name("result_equal").as_str(),
            fn_type,
            Some(Linkage::External),
        );
        fn_value
    }
}

/// `declare %Qubit* @__quantum__rt__qubit_allocate()`
pub(crate) fn qubit_allocate<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    if let Some(function) = get_function(module, "qubit_allocate") {
        function
    } else {
        let qubit_ptr_type = qubit(context, module).ptr_type(AddressSpace::Generic);
        let fn_type = qubit_ptr_type.fn_type(&[], false);
        let fn_value = module.add_function(
            get_function_name("qubit_allocate").as_str(),
            fn_type,
            Some(Linkage::External),
        );
        fn_value
    }
}

/// `declare void @__quantum__rt__qubit_release(%Qubit*)`
pub(crate) fn qubit_release<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &inkwell::module::Module<'ctx>,
) -> FunctionValue<'ctx> {
    if let Some(function) = get_function(module, "qubit_release") {
        function
    } else {
        let void_type = context.void_type();
        let fn_type = void_type.fn_type(
            &[qubit(context, module)
                .ptr_type(AddressSpace::Generic)
                .into()],
            false,
        );
        let fn_value = module.add_function(
            get_function_name("qubit_release").as_str(),
            fn_type,
            Some(Linkage::External),
        );
        fn_value
    }
}

fn get_function_name(suffix: &str) -> String {
    format!("__quantum__rt__{}", suffix)
}

pub(crate) fn get_function<'ctx>(module: &Module<'ctx>, name: &str) -> Option<FunctionValue<'ctx>> {
    let function_name = format!("__quantum__rt__{}", name);
    let defined_function = module.get_function(&function_name[..]);

    match defined_function {
        None => {
            log::debug!("{} was not defined in the module", function_name);
            None
        }
        Some(value) => Some(value),
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::gates::CURRENT_GATES;
use crate::interop::SemanticModel;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

use log;

use inkwell::values::FunctionValue;

use super::gates::GateScope;

pub(crate) struct Simulator {
    _scope: GateScope,
}

impl<'ctx> Simulator {
    pub fn new(module: &Module<'ctx>, ee: &ExecutionEngine<'ctx>) -> Self {
        let simulator = Simulator {
            _scope: crate::gates::GateScope::new(),
        };

        Simulator::bind(module, ee);

        simulator
    }

    pub fn get_model() -> SemanticModel {
        let mut gs = CURRENT_GATES.write().unwrap();
        gs.infer_allocations();
        gs.get_model()
    }

    fn bind(module: &Module<'ctx>, ee: &ExecutionEngine<'ctx>) {
        let intrinsics = Intrinsics::new(module);
        if let Some(ins) = intrinsics.cnot {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__cnot__body as usize);
        }
        if let Some(ins) = intrinsics.cz {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__cz__body as usize);
        }
        if let Some(ins) = intrinsics.h {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__h__body as usize);
        }
        if let Some(ins) = intrinsics.s {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__s__body as usize);
        }
        if let Some(ins) = intrinsics.s_adj {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__s__adj as usize);
        }
        if let Some(ins) = intrinsics.t {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__t__body as usize);
        }
        if let Some(ins) = intrinsics.t_adj {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__t__adj as usize);
        }
        if let Some(ins) = intrinsics.x {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__x__body as usize);
        }
        if let Some(ins) = intrinsics.y {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__y__body as usize);
        }
        if let Some(ins) = intrinsics.z {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__z__body as usize);
        }
        if let Some(ins) = intrinsics.r_x {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__rx__body as usize);
        }
        if let Some(ins) = intrinsics.r_y {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__ry__body as usize);
        }
        if let Some(ins) = intrinsics.r_z {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__rz__body as usize);
        }
        if let Some(ins) = intrinsics.reset {
            ee.add_global_mapping(
                &ins,
                super::intrinsics::__quantum__qis__reset__body as usize,
            );
        }

        if let Some(ins) = intrinsics.m {
            ee.add_global_mapping(
                &ins,
                super::intrinsics::__quantum__qis__m__body as usize,
            );
        }

        let runtime = Runtime::new(module);
        if let Some(ins) = runtime.result_get_one {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__rt__result_get_one as usize);
        }
        if let Some(ins) = runtime.result_get_zero {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__rt__result_get_zero as usize);
        }
        if let Some(ins) = runtime.result_equal {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__rt__result_equal as usize);
        }
        if let Some(ins) = runtime.qubit_allocate {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__rt__qubit_allocate as usize);
        }
        if let Some(ins) = runtime.qubit_release {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__rt__qubit_release as usize);
        }
    }
}

pub struct Intrinsics<'ctx> {
    pub cnot: Option<FunctionValue<'ctx>>,
    pub cz: Option<FunctionValue<'ctx>>,
    pub m: Option<FunctionValue<'ctx>>,
    pub r_x: Option<FunctionValue<'ctx>>,
    pub r_y: Option<FunctionValue<'ctx>>,
    pub r_z: Option<FunctionValue<'ctx>>,
    pub reset: Option<FunctionValue<'ctx>>,
    pub h: Option<FunctionValue<'ctx>>,
    pub x: Option<FunctionValue<'ctx>>,
    pub y: Option<FunctionValue<'ctx>>,
    pub z: Option<FunctionValue<'ctx>>,
    pub s: Option<FunctionValue<'ctx>>,
    pub s_adj: Option<FunctionValue<'ctx>>,
    pub t: Option<FunctionValue<'ctx>>,
    pub t_adj: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Intrinsics<'ctx> {
    pub fn new(module: &Module<'ctx>) -> Self {
        let intrinsics = Intrinsics {
            cnot: Intrinsics::get_qis_intrinsic_function_body(module, "cnot"),
            cz: Intrinsics::get_qis_intrinsic_function_body(module, "Cz"),
            m: Intrinsics::get_qis_intrinsic_function_body(module, "M"),
            r_x: Intrinsics::get_qis_intrinsic_function_body(module, "Rx"),
            r_y: Intrinsics::get_qis_intrinsic_function_body(module, "Ry"),
            r_z: Intrinsics::get_qis_intrinsic_function_body(module, "Rz"),
            reset: Intrinsics::get_qis_intrinsic_function_body(module, "Reset"),
            h: Intrinsics::get_qis_intrinsic_function_body(module, "H"),
            x: Intrinsics::get_qis_intrinsic_function_body(module, "X"),
            y: Intrinsics::get_qis_intrinsic_function_body(module, "Y"),
            z: Intrinsics::get_qis_intrinsic_function_body(module, "Z"),
            s: Intrinsics::get_qis_intrinsic_function_body(module, "S"),
            s_adj: Intrinsics::get_qis_intrinsic_function_adj(module, "S"),
            t: Intrinsics::get_qis_intrinsic_function_body(module, "T"),
            t_adj: Intrinsics::get_qis_intrinsic_function_adj(module, "T"),
        };

        intrinsics
    }

    fn get_qis_intrinsic_function_body(
        module: &Module<'ctx>,
        name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
        get_function(module, function_name.as_str())
    }

    fn get_qis_intrinsic_function_adj(
        module: &Module<'ctx>,
        name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("__quantum__qis__{}__adj", name.to_lowercase());
        get_function(module, function_name.as_str())
    }
}

pub struct Runtime<'ctx> {
    pub result_get_one: Option<FunctionValue<'ctx>>,
    pub result_get_zero: Option<FunctionValue<'ctx>>,
    pub result_equal: Option<FunctionValue<'ctx>>,
    pub qubit_allocate: Option<FunctionValue<'ctx>>,
    pub qubit_release: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Runtime<'ctx> {
    pub fn new(module: &Module<'ctx>) -> Self {
        let intrinsics = Runtime {
            result_get_one: Runtime::get_rt_intrinsic_function_body(module, "result_get_one"),
            result_get_zero: Runtime::get_rt_intrinsic_function_body(module, "result_get_zero"),
            result_equal: Runtime::get_rt_intrinsic_function_body(module, "result_equal"),
            qubit_allocate: Runtime::get_rt_intrinsic_function_body(module, "qubit_allocate"),
            qubit_release: Runtime::get_rt_intrinsic_function_body(module, "qubit_release"),
        };

        intrinsics
    }

    fn get_rt_intrinsic_function_body(
        module: &Module<'ctx>,
        name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("__quantum__rt__{}", name.to_lowercase());
        get_function(module, function_name.as_str())
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
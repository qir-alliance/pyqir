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

        if let Some(ins) = intrinsics.h_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__h__body as usize);
        }

        if let Some(ins) = intrinsics.h_ctl_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__h__ctl as usize);
        }
        if let Some(ins) = intrinsics.m_ins {
            ee.add_global_mapping(
                &ins,
                super::intrinsics::__quantum__qis__measure__body as usize,
            );
        }
        if let Some(ins) = intrinsics.r_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__r__body as usize);
        }
        if let Some(ins) = intrinsics.r_adj_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__r__adj as usize);
        }
        if let Some(ins) = intrinsics.r_ctl_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__r__ctl as usize);
        }
        if let Some(ins) = intrinsics.r_ctl_adj_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__r__ctladj as usize);
        }

        if let Some(ins) = intrinsics.s_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__s__body as usize);
        }
        if let Some(ins) = intrinsics.s_adj_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__s__adj as usize);
        }
        if let Some(ins) = intrinsics.s_ctl_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__s__ctl as usize);
        }
        if let Some(ins) = intrinsics.s_ctl_adj_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__s__ctladj as usize);
        }

        if let Some(ins) = intrinsics.t_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__t__body as usize);
        }
        if let Some(ins) = intrinsics.t_adj_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__t__adj as usize);
        }
        if let Some(ins) = intrinsics.t_ctl_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__t__ctl as usize);
        }
        if let Some(ins) = intrinsics.t_ctl_adj_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__t__ctladj as usize);
        }

        if let Some(ins) = intrinsics.x_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__x__body as usize);
        }
        if let Some(ins) = intrinsics.x_ctl_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__x__ctl as usize);
        }

        if let Some(ins) = intrinsics.y_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__y__body as usize);
        }
        if let Some(ins) = intrinsics.y_ctl_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__y__ctl as usize);
        }

        if let Some(ins) = intrinsics.z_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__z__body as usize);
        }
        if let Some(ins) = intrinsics.z_ctl_ins {
            ee.add_global_mapping(&ins, super::intrinsics::__quantum__qis__z__ctl as usize);
        }
    }
}

pub struct Intrinsics<'ctx> {
    pub m: Option<FunctionValue<'ctx>>,
    pub m_ins: Option<FunctionValue<'ctx>>,
    pub r_x: Option<FunctionValue<'ctx>>,
    pub r_y: Option<FunctionValue<'ctx>>,
    pub r_z: Option<FunctionValue<'ctx>>,
    pub r_ins: Option<FunctionValue<'ctx>>,
    pub r_adj_ins: Option<FunctionValue<'ctx>>,
    pub r_ctl_ins: Option<FunctionValue<'ctx>>,
    pub r_ctl_adj_ins: Option<FunctionValue<'ctx>>,
    pub reset: Option<FunctionValue<'ctx>>,
    pub h: Option<FunctionValue<'ctx>>,
    pub h_ins: Option<FunctionValue<'ctx>>,
    pub h_ctl_ins: Option<FunctionValue<'ctx>>,
    pub x: Option<FunctionValue<'ctx>>,
    pub x_ins: Option<FunctionValue<'ctx>>,
    pub x_ctl: Option<FunctionValue<'ctx>>,
    pub x_ctl_ins: Option<FunctionValue<'ctx>>,
    pub y: Option<FunctionValue<'ctx>>,
    pub y_ins: Option<FunctionValue<'ctx>>,
    pub y_ctl: Option<FunctionValue<'ctx>>,
    pub y_ctl_ins: Option<FunctionValue<'ctx>>,
    pub z: Option<FunctionValue<'ctx>>,
    pub z_ins: Option<FunctionValue<'ctx>>,
    pub z_ctl: Option<FunctionValue<'ctx>>,
    pub z_ctl_ins: Option<FunctionValue<'ctx>>,
    pub s: Option<FunctionValue<'ctx>>,
    pub s_ins: Option<FunctionValue<'ctx>>,
    pub s_adj: Option<FunctionValue<'ctx>>,
    pub s_adj_ins: Option<FunctionValue<'ctx>>,
    pub s_ctl_ins: Option<FunctionValue<'ctx>>,
    pub s_ctl_adj_ins: Option<FunctionValue<'ctx>>,
    pub t: Option<FunctionValue<'ctx>>,
    pub t_ins: Option<FunctionValue<'ctx>>,
    pub t_adj: Option<FunctionValue<'ctx>>,
    pub t_adj_ins: Option<FunctionValue<'ctx>>,
    pub t_ctl_ins: Option<FunctionValue<'ctx>>,
    pub t_ctl_adj_ins: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Intrinsics<'ctx> {
    pub fn new(module: &Module<'ctx>) -> Self {
        let intrinsics = Intrinsics {
            m: Intrinsics::get_mqi_body(module, "M"),
            m_ins: Intrinsics::get_qis_intrinsic_function_body(module, "measure"),
            r_x: Intrinsics::get_mqi_body(module, "Rx"),
            r_y: Intrinsics::get_mqi_body(module, "Ry"),
            r_z: Intrinsics::get_mqi_body(module, "Rz"),
            r_ins: Intrinsics::get_qis_intrinsic_function_body(module, "r"),
            r_adj_ins: Intrinsics::get_qis_intrinsic_function_adj(module, "r"),
            r_ctl_ins: Intrinsics::get_qis_intrinsic_function_ctl(module, "r"),
            r_ctl_adj_ins: Intrinsics::get_qis_intrinsic_function_ctladj(module, "r"),
            reset: Intrinsics::get_mqi_body(module, "Reset"),
            h: Intrinsics::get_mqi_body(module, "H"),
            h_ins: Intrinsics::get_qis_intrinsic_function_body(module, "H"),
            h_ctl_ins: Intrinsics::get_qis_intrinsic_function_ctl(module, "H"),
            x: Intrinsics::get_mqi_body(module, "X"),
            x_ins: Intrinsics::get_qis_intrinsic_function_body(module, "X"),
            x_ctl: Intrinsics::get_mqi_ctl(module, "X"),
            x_ctl_ins: Intrinsics::get_qis_intrinsic_function_ctl(module, "X"),
            y: Intrinsics::get_mqi_body(module, "Y"),
            y_ins: Intrinsics::get_qis_intrinsic_function_body(module, "Y"),
            y_ctl: Intrinsics::get_mqi_ctl(module, "Y"),
            y_ctl_ins: Intrinsics::get_qis_intrinsic_function_ctl(module, "Y"),
            z: Intrinsics::get_mqi_body(module, "Z"),
            z_ins: Intrinsics::get_qis_intrinsic_function_body(module, "Z"),
            z_ctl: Intrinsics::get_mqi_ctl(module, "Z"),
            z_ctl_ins: Intrinsics::get_qis_intrinsic_function_ctl(module, "Z"),
            s: Intrinsics::get_mqi_body(module, "S"),
            s_ins: Intrinsics::get_qis_intrinsic_function_body(module, "S"),
            s_adj: Intrinsics::get_mqi_adj(module, "S"),
            s_adj_ins: Intrinsics::get_qis_intrinsic_function_adj(module, "S"),
            s_ctl_ins: Intrinsics::get_qis_intrinsic_function_ctl(module, "S"),
            s_ctl_adj_ins: Intrinsics::get_qis_intrinsic_function_ctladj(module, "S"),
            t: Intrinsics::get_mqi_body(module, "T"),
            t_ins: Intrinsics::get_qis_intrinsic_function_body(module, "T"),
            t_adj: Intrinsics::get_mqi_adj(module, "T"),
            t_adj_ins: Intrinsics::get_qis_intrinsic_function_adj(module, "T"),
            t_ctl_ins: Intrinsics::get_qis_intrinsic_function_ctl(module, "T"),
            t_ctl_adj_ins: Intrinsics::get_qis_intrinsic_function_ctladj(module, "T"),
        };

        intrinsics
    }

    fn get_qis_intrinsic_function_ctl(
        module: &Module<'ctx>,
        name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("__quantum__qis__{}__ctl", name.to_lowercase());
        Intrinsics::get_function(module, function_name.as_str())
    }

    fn get_qis_intrinsic_function_ctladj(
        module: &Module<'ctx>,
        name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("__quantum__qis__{}__ctladj", name.to_lowercase());
        Intrinsics::get_function(module, function_name.as_str())
    }

    fn get_qis_intrinsic_function_body(
        module: &Module<'ctx>,
        name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
        Intrinsics::get_function(module, function_name.as_str())
    }

    fn get_qis_intrinsic_function_adj(
        module: &Module<'ctx>,
        name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("__quantum__qis__{}__adj", name.to_lowercase());
        Intrinsics::get_function(module, function_name.as_str())
    }

    fn get_mqi_body(module: &Module<'ctx>, name: &str) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("Microsoft__Quantum__Intrinsic__{}__body", name);
        Intrinsics::get_function(module, function_name.as_str())
    }

    fn get_mqi_ctl(module: &Module<'ctx>, name: &str) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("Microsoft__Quantum__Intrinsic__{}__ctl", name);
        Intrinsics::get_function(module, function_name.as_str())
    }

    fn get_mqi_adj(module: &Module<'ctx>, name: &str) -> Option<FunctionValue<'ctx>> {
        let function_name = format!("Microsoft__Quantum__Intrinsic__{}__adj", name);
        Intrinsics::get_function(module, function_name.as_str())
    }

    fn get_function(module: &Module<'ctx>, function_name: &str) -> Option<FunctionValue<'ctx>> {
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
}

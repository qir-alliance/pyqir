// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log;

use inkwell::module::Module;
use inkwell::values::FunctionValue;

use super::CodeGenerator;

pub trait Intrinsics<'ctx> {
    fn qis_cnot_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_cz_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_h_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_s_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_s_adj(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_t_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_t_adj(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_x_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_y_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_z_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_rx_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_ry_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_rz_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_reset_body(&self) -> Option<FunctionValue<'ctx>>;
    fn qis_m_body(&self) -> Option<FunctionValue<'ctx>>;
}

impl<'ctx> Intrinsics<'ctx> for CodeGenerator<'ctx> {
    fn qis_cnot_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "cnot")
    }

    fn qis_cz_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "cz")
    }

    fn qis_h_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "h")
    }

    fn qis_s_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "s")
    }

    fn qis_s_adj(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_adj(&self.module, "s")
    }

    fn qis_t_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "t")
    }

    fn qis_t_adj(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_adj(&self.module, "t")
    }

    fn qis_x_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "x")
    }

    fn qis_y_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "y")
    }

    fn qis_z_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "z")
    }

    fn qis_rx_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "rx")
    }

    fn qis_ry_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "ry")
    }

    fn qis_rz_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "rz")
    }

    fn qis_reset_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "reset")
    }

    fn qis_m_body(&self) -> Option<FunctionValue<'ctx>> {
        get_qis_intrinsic_function_body(&self.module, "m")
    }
}

fn get_qis_intrinsic_function_body<'ctx>(
    module: &Module<'ctx>,
    name: &str,
) -> Option<FunctionValue<'ctx>> {
    let function_name = format!("__quantum__qis__{}__body", name.to_lowercase());
    get_function(module, function_name.as_str())
}

fn get_qis_intrinsic_function_adj<'ctx>(
    module: &Module<'ctx>,
    name: &str,
) -> Option<FunctionValue<'ctx>> {
    let function_name = format!("__quantum__qis__{}__adj", name.to_lowercase());
    get_function(module, function_name.as_str())
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

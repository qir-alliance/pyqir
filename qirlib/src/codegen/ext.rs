// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    types::{FloatType, IntType, StructType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, InstructionValue},
};

use super::{
    basicvalues::{f64_to_f64, i64_to_i32, i8_null_ptr, u64_to_i32, u64_to_i64},
    qis::{
        cnot_body, cz_body, h_body, m_body, reset_body, rx_body, ry_body, rz_body, s_adj, s_body,
        t_adj, t_body, x_body, y_body, z_body,
    },
    qubits::{emit_allocate_qubit, emit_release_qubit},
    rt::{qubit_allocate, qubit_release, result_equal, result_get_one, result_get_zero},
    types::{array, int32, int64, int8, qubit, result},
    CodeGenerator,
};

pub trait BasicValues<'ctx> {
    fn i8_null_ptr(&self) -> BasicMetadataValueEnum<'ctx>;
    fn f64_to_f64(&self, value: f64) -> BasicMetadataValueEnum<'ctx>;
    fn u64_to_i32(&self, value: u64) -> BasicMetadataValueEnum<'ctx>;
    fn i64_to_i32(&self, value: i64) -> BasicMetadataValueEnum<'ctx>;
    fn u64_to_i64(&self, value: u64) -> BasicMetadataValueEnum<'ctx>;
    fn usize_to_i64(&self, value: usize) -> BasicMetadataValueEnum<'ctx>;
}

impl<'ctx> BasicValues<'ctx> for CodeGenerator<'ctx> {
    fn i8_null_ptr(&self) -> BasicMetadataValueEnum<'ctx> {
        i8_null_ptr(self.context)
    }

    fn f64_to_f64(&self, value: f64) -> BasicMetadataValueEnum<'ctx> {
        f64_to_f64(self.context, value)
    }

    fn u64_to_i32(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i32(self.context, value)
    }

    fn i64_to_i32(&self, value: i64) -> BasicMetadataValueEnum<'ctx> {
        i64_to_i32(self.context, value)
    }

    fn u64_to_i64(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i64(self.context, value)
    }

    fn usize_to_i64(&self, value: usize) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i64(self.context, value as u64)
    }
}

pub trait Intrinsics<'ctx> {
    fn qis_cnot_body(&self) -> FunctionValue<'ctx>;
    fn qis_cz_body(&self) -> FunctionValue<'ctx>;
    fn qis_h_body(&self) -> FunctionValue<'ctx>;
    fn qis_s_body(&self) -> FunctionValue<'ctx>;
    fn qis_s_adj(&self) -> FunctionValue<'ctx>;
    fn qis_t_body(&self) -> FunctionValue<'ctx>;
    fn qis_t_adj(&self) -> FunctionValue<'ctx>;
    fn qis_x_body(&self) -> FunctionValue<'ctx>;
    fn qis_y_body(&self) -> FunctionValue<'ctx>;
    fn qis_z_body(&self) -> FunctionValue<'ctx>;
    fn qis_rx_body(&self) -> FunctionValue<'ctx>;
    fn qis_ry_body(&self) -> FunctionValue<'ctx>;
    fn qis_rz_body(&self) -> FunctionValue<'ctx>;
    fn qis_reset_body(&self) -> FunctionValue<'ctx>;
    fn qis_m_body(&self) -> FunctionValue<'ctx>;
}

impl<'ctx> Intrinsics<'ctx> for CodeGenerator<'ctx> {
    fn qis_cnot_body(&self) -> FunctionValue<'ctx> {
        cnot_body(self.context, &self.module)
    }

    fn qis_cz_body(&self) -> FunctionValue<'ctx> {
        cz_body(self.context, &self.module)
    }

    fn qis_h_body(&self) -> FunctionValue<'ctx> {
        h_body(self.context, &self.module)
    }

    fn qis_s_body(&self) -> FunctionValue<'ctx> {
        s_body(self.context, &self.module)
    }

    fn qis_s_adj(&self) -> FunctionValue<'ctx> {
        s_adj(self.context, &self.module)
    }

    fn qis_t_body(&self) -> FunctionValue<'ctx> {
        t_body(self.context, &self.module)
    }

    fn qis_t_adj(&self) -> FunctionValue<'ctx> {
        t_adj(self.context, &self.module)
    }

    fn qis_x_body(&self) -> FunctionValue<'ctx> {
        x_body(self.context, &self.module)
    }

    fn qis_y_body(&self) -> FunctionValue<'ctx> {
        y_body(self.context, &self.module)
    }

    fn qis_z_body(&self) -> FunctionValue<'ctx> {
        z_body(self.context, &self.module)
    }

    fn qis_rx_body(&self) -> FunctionValue<'ctx> {
        rx_body(self.context, &self.module)
    }

    fn qis_ry_body(&self) -> FunctionValue<'ctx> {
        ry_body(self.context, &self.module)
    }

    fn qis_rz_body(&self) -> FunctionValue<'ctx> {
        rz_body(self.context, &self.module)
    }

    fn qis_reset_body(&self) -> FunctionValue<'ctx> {
        reset_body(self.context, &self.module)
    }

    fn qis_m_body(&self) -> FunctionValue<'ctx> {
        m_body(self.context, &self.module)
    }
}

pub trait Qubits<'ctx> {
    fn emit_allocate_qubit(&self, result_name: &str) -> BasicValueEnum<'ctx>;
    fn emit_release_qubit(&self, qubit: &BasicValueEnum<'ctx>) -> InstructionValue<'ctx>;
}

impl<'ctx> Qubits<'ctx> for CodeGenerator<'ctx> {
    fn emit_allocate_qubit(&self, result_name: &str) -> BasicValueEnum<'ctx> {
        emit_allocate_qubit(self.context, &self.builder, &self.module, result_name)
    }

    fn emit_release_qubit(&self, qubit: &BasicValueEnum<'ctx>) -> InstructionValue<'ctx> {
        emit_release_qubit(self.context, &self.builder, &self.module, qubit)
    }
}

pub trait RuntimeLibrary<'ctx> {
    fn result_get_zero(&self) -> FunctionValue<'ctx>;
    fn result_get_one(&self) -> FunctionValue<'ctx>;
    fn result_equal(&self) -> FunctionValue<'ctx>;
    fn qubit_allocate(&self) -> FunctionValue<'ctx>;
    fn qubit_release(&self) -> FunctionValue<'ctx>;
}

impl<'ctx> RuntimeLibrary<'ctx> for CodeGenerator<'ctx> {
    fn result_get_zero(&self) -> FunctionValue<'ctx> {
        result_get_zero(self.context, &self.module)
    }

    fn result_get_one(&self) -> FunctionValue<'ctx> {
        result_get_one(self.context, &self.module)
    }

    fn result_equal(&self) -> FunctionValue<'ctx> {
        result_equal(self.context, &self.module)
    }

    fn qubit_allocate(&self) -> FunctionValue<'ctx> {
        qubit_allocate(self.context, &self.module)
    }

    fn qubit_release(&self) -> FunctionValue<'ctx> {
        qubit_release(self.context, &self.module)
    }
}

pub trait Types<'ctx> {
    fn int64_type(&self) -> IntType<'ctx>;
    fn int32_type(&self) -> IntType<'ctx>;
    fn int8_type(&self) -> IntType<'ctx>;
    fn double_type(&self) -> FloatType<'ctx>;
    fn bool_type(&self) -> IntType<'ctx>;
    fn qubit_type(&self) -> StructType<'ctx>;
    fn result_type(&self) -> StructType<'ctx>;
    fn array_type(&self) -> StructType<'ctx>;
}

impl<'ctx> Types<'ctx> for CodeGenerator<'ctx> {
    fn int64_type(&self) -> IntType<'ctx> {
        int64(self.context)
    }

    fn int32_type(&self) -> IntType<'ctx> {
        int32(self.context)
    }

    fn int8_type(&self) -> IntType<'ctx> {
        int8(self.context)
    }

    fn double_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    fn qubit_type(&self) -> StructType<'ctx> {
        qubit(self.context, &self.module)
    }

    fn result_type(&self) -> StructType<'ctx> {
        result(self.context, &self.module)
    }

    fn array_type(&self) -> StructType<'ctx> {
        array(self.context, &self.module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::CodeGenerator;
    use inkwell::context::Context;

    #[test]
    fn qubit_can_be_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Qubit", generator.qubit_type());
    }

    #[test]
    fn result_can_be_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Result", generator.result_type());
    }

    #[test]
    fn array_can_be_declared() {
        let context = Context::create();
        let module = context.create_module("test");
        let generator = CodeGenerator::new(&context, module).unwrap();

        verify_opaque_struct("Array", generator.array_type());
    }

    fn verify_opaque_struct(name: &str, struct_type: StructType) {
        assert_eq!(struct_type.get_name().unwrap().to_str(), Ok(name));
        assert!(struct_type.is_opaque());
        assert_eq!(struct_type.get_field_types(), &[]);
    }
}

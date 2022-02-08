// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    module::Module,
    types::{FloatType, IntType, StructType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, InstructionValue},
};
use std::path::Path;

use self::{
    basicvalues::{f64_to_f64, i64_to_i32, i8_null_ptr, u64_to_i32, u64_to_i64},
    calls::{emit_call_with_return, emit_void_call},
    qis::{
        cnot_body, cz_body, h_body, m_body, reset_body, rx_body, ry_body, rz_body, s_adj, s_body,
        t_adj, t_body, x_body, y_body, z_body,
    },
    qubits::{emit_allocate_qubit, emit_release_qubit},
    rt::{qubit_allocate, qubit_release, result_equal, result_get_one, result_get_zero},
    types::{int32, int64, int8, qubit, result},
};

pub mod basicvalues;
pub mod calls;
pub mod qis;
pub mod qubits;
pub mod rt;
pub mod types;

pub struct CodeGenerator<'ctx> {
    pub context: &'ctx inkwell::context::Context,
    pub module: inkwell::module::Module<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// # Errors
    ///
    /// Will return `Err` if module fails to load
    pub fn new(
        context: &'ctx inkwell::context::Context,
        module: Module<'ctx>,
    ) -> Result<Self, String> {
        let builder = context.create_builder();
        Ok(CodeGenerator {
            context,
            module,
            builder,
        })
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn emit_bitcode(&self, file_path: &str) {
        let bitcode_path = Path::new(file_path);
        self.module.write_bitcode_to_path(bitcode_path);
    }

    /// # Errors
    ///
    /// Will return `Err` if LLVM Module fails validation
    pub fn emit_ir(&self, file_path: &str) -> Result<(), String> {
        let ir_path = Path::new(file_path);
        if let Err(llvmstr) = self.module.print_to_file(ir_path) {
            return Err(llvmstr.to_string());
        }
        Ok(())
    }

    pub fn get_ir_string(&self) -> String {
        let ir = self.module.print_to_string();
        ir.to_string()
    }

    pub fn get_bitcode_base64_string(&self) -> String {
        let buffer = self.module.write_bitcode_to_memory();
        let bytes = buffer.as_slice();
        base64::encode(bytes)
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn emit_void_call(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> InstructionValue<'ctx> {
        emit_void_call(&self.builder, function, args)
    }

    pub fn emit_call_with_return(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> BasicValueEnum<'ctx> {
        emit_call_with_return(&self.builder, function, args, name)
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn i8_null_ptr(&self) -> BasicMetadataValueEnum<'ctx> {
        i8_null_ptr(self.context)
    }

    pub fn f64_to_f64(&self, value: f64) -> BasicMetadataValueEnum<'ctx> {
        f64_to_f64(self.context, value)
    }

    pub fn u64_to_i32(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i32(self.context, value)
    }

    pub fn i64_to_i32(&self, value: i64) -> BasicMetadataValueEnum<'ctx> {
        i64_to_i32(self.context, value)
    }

    pub fn u64_to_i64(&self, value: u64) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i64(self.context, value)
    }

    pub fn usize_to_i64(&self, value: usize) -> BasicMetadataValueEnum<'ctx> {
        u64_to_i64(self.context, value as u64)
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn qis_cnot_body(&self) -> FunctionValue<'ctx> {
        cnot_body(self.context, &self.module)
    }

    pub fn qis_cz_body(&self) -> FunctionValue<'ctx> {
        cz_body(self.context, &self.module)
    }

    pub fn qis_h_body(&self) -> FunctionValue<'ctx> {
        h_body(self.context, &self.module)
    }

    pub fn qis_s_body(&self) -> FunctionValue<'ctx> {
        s_body(self.context, &self.module)
    }

    pub fn qis_s_adj(&self) -> FunctionValue<'ctx> {
        s_adj(self.context, &self.module)
    }

    pub fn qis_t_body(&self) -> FunctionValue<'ctx> {
        t_body(self.context, &self.module)
    }

    pub fn qis_t_adj(&self) -> FunctionValue<'ctx> {
        t_adj(self.context, &self.module)
    }

    pub fn qis_x_body(&self) -> FunctionValue<'ctx> {
        x_body(self.context, &self.module)
    }

    pub fn qis_y_body(&self) -> FunctionValue<'ctx> {
        y_body(self.context, &self.module)
    }

    pub fn qis_z_body(&self) -> FunctionValue<'ctx> {
        z_body(self.context, &self.module)
    }

    pub fn qis_rx_body(&self) -> FunctionValue<'ctx> {
        rx_body(self.context, &self.module)
    }

    pub fn qis_ry_body(&self) -> FunctionValue<'ctx> {
        ry_body(self.context, &self.module)
    }

    pub fn qis_rz_body(&self) -> FunctionValue<'ctx> {
        rz_body(self.context, &self.module)
    }

    pub fn qis_reset_body(&self) -> FunctionValue<'ctx> {
        reset_body(self.context, &self.module)
    }

    pub fn qis_m_body(&self) -> FunctionValue<'ctx> {
        m_body(self.context, &self.module)
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn emit_allocate_qubit(&self, result_name: &str) -> BasicValueEnum<'ctx> {
        emit_allocate_qubit(self.context, &self.builder, &self.module, result_name)
    }

    pub fn emit_release_qubit(&self, qubit: &BasicValueEnum<'ctx>) -> InstructionValue<'ctx> {
        emit_release_qubit(self.context, &self.builder, &self.module, qubit)
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn rt_result_get_zero(&self) -> FunctionValue<'ctx> {
        result_get_zero(self.context, &self.module)
    }

    pub fn rt_result_get_one(&self) -> FunctionValue<'ctx> {
        result_get_one(self.context, &self.module)
    }

    pub fn rt_result_equal(&self) -> FunctionValue<'ctx> {
        result_equal(self.context, &self.module)
    }

    pub fn rt_qubit_allocate(&self) -> FunctionValue<'ctx> {
        qubit_allocate(self.context, &self.module)
    }

    pub fn rt_qubit_release(&self) -> FunctionValue<'ctx> {
        qubit_release(self.context, &self.module)
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn int64_type(&self) -> IntType<'ctx> {
        int64(self.context)
    }

    pub fn int32_type(&self) -> IntType<'ctx> {
        int32(self.context)
    }

    pub fn int8_type(&self) -> IntType<'ctx> {
        int8(self.context)
    }

    pub fn double_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    pub fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    pub fn qubit_type(&self) -> StructType<'ctx> {
        qubit(self.context, &self.module)
    }

    pub fn result_type(&self) -> StructType<'ctx> {
        result(self.context, &self.module)
    }
}

#[cfg(test)]
mod core_tests {
    use crate::codegen::CodeGenerator;
    use inkwell::context::Context;
    use std::{fs::File, io::prelude::*};
    use tempfile::tempdir;

    #[test]
    fn emitted_bitcode_files_are_identical_to_base64_encoded() {
        let dir = tempdir().expect("");
        let tmp_path = dir.into_path();
        let name = "test";
        let file_path = tmp_path.join(format!("{}.bc", name));
        let file_path_string = file_path.display().to_string();

        let context = Context::create();
        let module = context.create_module(name);
        let generator = CodeGenerator::new(&context, module).unwrap();
        generator.emit_bitcode(file_path_string.as_str());
        let mut emitted_bitcode_file =
            File::open(file_path_string.as_str()).expect("Could not open emitted bitcode file");
        let mut buffer = vec![];

        emitted_bitcode_file
            .read_to_end(&mut buffer)
            .expect("Could not read emitted bitcode file");
        let emitted_bitcode_bytes = buffer.as_slice();

        let b64_bitcode = generator.get_bitcode_base64_string();
        let decoded = base64::decode(b64_bitcode).expect("could not decode base64 encoded module");
        let decoded_bitcode_bytes = decoded.as_slice();

        assert_eq!(emitted_bitcode_bytes, decoded_bitcode_bytes);
    }
}

#[cfg(test)]
mod types_tests {
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

    fn verify_opaque_struct(name: &str, struct_type: StructType) {
        assert_eq!(struct_type.get_name().unwrap().to_str(), Ok(name));
        assert!(struct_type.is_opaque());
        assert_eq!(struct_type.get_field_types(), &[]);
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    evaluator::PyNonadaptiveJit,
    generator::{
        bitcode_to_ir, ir_to_bitcode, r#const, Attribute, BasicQisBuilder, Builder, Module,
        SimpleModule, TypeFactory,
    },
    instructions::{
        Call, FCmp, FloatPredicate, ICmp, Instruction, IntPredicate, Opcode, Phi, Switch,
    },
    types::{is_qubit, is_result, ArrayType, FunctionType, IntType, PointerType, StructType, Type},
    values::{
        constant_bytes, is_entry_point, is_interop_friendly, qubit_id, required_num_qubits,
        required_num_results, result_id, BasicBlock, Constant, FloatConstant, Function,
        IntConstant, Value,
    },
};
use pyo3::prelude::*;

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArrayType>()?;
    m.add_class::<Attribute>()?;
    m.add_class::<BasicBlock>()?;
    m.add_class::<BasicQisBuilder>()?;
    m.add_class::<Builder>()?;
    m.add_class::<Call>()?;
    m.add_class::<Constant>()?;
    m.add_class::<FCmp>()?;
    m.add_class::<FloatConstant>()?;
    m.add_class::<FloatPredicate>()?;
    m.add_class::<Function>()?;
    m.add_class::<FunctionType>()?;
    m.add_class::<ICmp>()?;
    m.add_class::<Instruction>()?;
    m.add_class::<IntConstant>()?;
    m.add_class::<IntPredicate>()?;
    m.add_class::<IntType>()?;
    m.add_class::<Module>()?;
    m.add_class::<Opcode>()?;
    m.add_class::<Phi>()?;
    m.add_class::<PointerType>()?;
    m.add_class::<PyNonadaptiveJit>()?;
    m.add_class::<SimpleModule>()?;
    m.add_class::<StructType>()?;
    m.add_class::<Switch>()?;
    m.add_class::<Type>()?;
    m.add_class::<TypeFactory>()?;
    m.add_class::<Value>()?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;
    m.add_function(wrap_pyfunction!(constant_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    m.add_function(wrap_pyfunction!(is_entry_point, m)?)?;
    m.add_function(wrap_pyfunction!(is_interop_friendly, m)?)?;
    m.add_function(wrap_pyfunction!(is_qubit, m)?)?;
    m.add_function(wrap_pyfunction!(is_result, m)?)?;
    m.add_function(wrap_pyfunction!(qubit_id, m)?)?;
    m.add_function(wrap_pyfunction!(r#const, m)?)?;
    m.add_function(wrap_pyfunction!(required_num_qubits, m)?)?;
    m.add_function(wrap_pyfunction!(required_num_results, m)?)?;
    m.add_function(wrap_pyfunction!(result_id, m)?)?;
    Ok(())
}

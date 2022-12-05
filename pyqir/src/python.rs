// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    context::Context,
    evaluator::PyNonadaptiveJit,
    instructions::{
        Call, FCmp, FloatPredicate, ICmp, Instruction, IntPredicate, Opcode, Phi, Switch,
    },
    module::{Linkage, Module},
    qis::BasicQisBuilder,
    types::{
        is_qubit_type, is_result_type, qubit_type, result_type, ArrayType, FunctionType, IntType,
        PointerType, StructType, Type,
    },
    values::{
        entry_point, extract_byte_string, global_byte_string, is_entry_point, is_interop_friendly,
        qubit, qubit_id, r#const, required_num_qubits, required_num_results, result, result_id,
        Attribute, AttributeDict, AttributeGroup, BasicBlock, Constant, FloatConstant, Function,
        IntConstant, Value,
    },
};
use pyo3::prelude::*;

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArrayType>()?;
    m.add_class::<Attribute>()?;
    m.add_class::<AttributeDict>()?;
    m.add_class::<AttributeGroup>()?;
    m.add_class::<BasicBlock>()?;
    m.add_class::<BasicQisBuilder>()?;
    m.add_class::<Builder>()?;
    m.add_class::<Call>()?;
    m.add_class::<Constant>()?;
    m.add_class::<Context>()?;
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
    m.add_class::<Linkage>()?;
    m.add_class::<Module>()?;
    m.add_class::<Opcode>()?;
    m.add_class::<Phi>()?;
    m.add_class::<PointerType>()?;
    m.add_class::<PyNonadaptiveJit>()?;
    m.add_class::<StructType>()?;
    m.add_class::<Switch>()?;
    m.add_class::<Type>()?;
    m.add_class::<Value>()?;
    m.add_function(wrap_pyfunction!(entry_point, m)?)?;
    m.add_function(wrap_pyfunction!(extract_byte_string, m)?)?;
    m.add_function(wrap_pyfunction!(global_byte_string, m)?)?;
    m.add_function(wrap_pyfunction!(is_entry_point, m)?)?;
    m.add_function(wrap_pyfunction!(is_interop_friendly, m)?)?;
    m.add_function(wrap_pyfunction!(is_qubit_type, m)?)?;
    m.add_function(wrap_pyfunction!(is_result_type, m)?)?;
    m.add_function(wrap_pyfunction!(qubit_id, m)?)?;
    m.add_function(wrap_pyfunction!(qubit_type, m)?)?;
    m.add_function(wrap_pyfunction!(qubit, m)?)?;
    m.add_function(wrap_pyfunction!(r#const, m)?)?;
    m.add_function(wrap_pyfunction!(required_num_qubits, m)?)?;
    m.add_function(wrap_pyfunction!(required_num_results, m)?)?;
    m.add_function(wrap_pyfunction!(result_id, m)?)?;
    m.add_function(wrap_pyfunction!(result_type, m)?)?;
    m.add_function(wrap_pyfunction!(result, m)?)?;
    Ok(())
}

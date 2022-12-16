// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    core::Context,
    instructions::{
        Call, FCmp, FloatPredicate, ICmp, Instruction, IntPredicate, Opcode, Phi, Switch,
    },
    module::{Linkage, Module, ModuleFlagBehavior},
    qis::BasicQisBuilder,
    qis::{barrier, swap},
    rt::{array_record_output, initialize, result_record_output, tuple_record_output},
    types::{
        is_qubit_type, is_result_type, qubit_type, result_type, ArrayType, FunctionType, IntType,
        PointerType, StructType, Type,
    },
    values::{
        entry_point, extract_byte_string, global_byte_string, is_entry_point, is_interop_friendly,
        qubit, qubit_id, r#const, required_num_qubits, required_num_results, result, result_id,
        Attribute, AttributeList, AttributeSet, BasicBlock, Constant, FloatConstant, Function,
        IntConstant, Value,
    },
};
use pyo3::{prelude::*, types::PyDict, wrap_pymodule};

#[pymodule]
fn _native(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArrayType>()?;
    m.add_class::<Attribute>()?;
    m.add_class::<AttributeList>()?;
    m.add_class::<AttributeSet>()?;
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
    m.add_class::<ModuleFlagBehavior>()?;
    m.add_class::<Opcode>()?;
    m.add_class::<Phi>()?;
    m.add_class::<PointerType>()?;
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

    m.add_wrapped(wrap_pymodule!(_qis))?;
    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("pyqir.qis._native", m.getattr("_qis")?)?;

    m.add_wrapped(wrap_pymodule!(_rt))?;
    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("pyqir.rt._native", m.getattr("_rt")?)?;

    Ok(())
}

#[pymodule]
fn _qis(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(barrier, m)?)?;
    m.add_function(wrap_pyfunction!(swap, m)?)?;
    Ok(())
}

#[pymodule]
fn _rt(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(array_record_output, m)?)?;
    m.add_function(wrap_pyfunction!(initialize, m)?)?;
    m.add_function(wrap_pyfunction!(result_record_output, m)?)?;
    m.add_function(wrap_pyfunction!(tuple_record_output, m)?)?;
    Ok(())
}

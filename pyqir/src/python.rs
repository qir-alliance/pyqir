// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    evaluator::PyNonadaptiveJit,
    generator::{
        bitcode_to_ir, ir_to_bitcode, r#const, BasicQisBuilder, Builder, SimpleModule, Type,
        TypeFactory, Value,
    },
};
use pyo3::prelude::*;

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<BasicQisBuilder>()?;
    m.add_class::<Builder>()?;
    m.add_class::<PyNonadaptiveJit>()?;
    m.add_class::<SimpleModule>()?;
    m.add_class::<Type>()?;
    m.add_class::<TypeFactory>()?;
    m.add_class::<Value>()?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;
    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    m.add_function(wrap_pyfunction!(r#const, m)?)?;
    Ok(())
}

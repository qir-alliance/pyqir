// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{builder::Builder, context, values::Value};
use pyo3::prelude::*;
use qirlib::rt::BuilderExt;
use std::convert::TryInto;

/// Inserts a marker in the generated output that indicates the start
/// of an array and how many array elements it has.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value num_elements: How many array elements the array has
/// :param str label: A string label for the array. Depending on the output schema, the label is included in the output or omitted.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn array_record_output(
    py: Python,
    builder: Py<Builder>,
    num_elements: &Value,
    label: &Value,
) -> PyResult<()> {
    let builder: PyRef<Builder> = builder.borrow(py);
    context::require_same(
        py,
        [builder.context(), num_elements.context(), label.context()],
    )?;
    unsafe { builder.get() }.build_array_record_output(
        unsafe { num_elements.get() }.try_into()?,
        unsafe { label.get() }.try_into()?,
    );
    Ok(())
}

/// Adds a measurement result to the generated output.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value result: A result measurement to record
/// :param str label: A string label for the result value. Depending on the output schema, the label is included in the output or omitted.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn result_record_output(
    py: Python,
    builder: Py<Builder>,
    result: &Value,
    label: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    context::require_same(py, [builder.context(), result.context(), label.context()])?;
    unsafe { builder.get() }.build_result_record_output(
        unsafe { result.get() }.try_into()?,
        unsafe { label.get() }.try_into()?,
    );
    Ok(())
}

/// Initializes the execution environment. Sets all qubits to a zero-state
/// if they are not dynamically managed.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value reserved: Reserved. For base profile QIR, a const null i8* Value should be passed.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn initialize(py: Python, builder: Py<Builder>, reserved: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    context::require_same(py, [builder.context(), reserved.context()])?;
    unsafe { builder.get() }.build_initialize(unsafe { reserved.get() }.try_into()?);
    Ok(())
}

/// Inserts a marker in the generated output that indicates the start
/// of a tuple and how many tuple elements it has.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value num_elements: How many tuple elements the tuple has
/// :param str label: A string label for the tuple. Depending on the output schema, the label is included in the output or omitted.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn tuple_record_output(
    py: Python,
    builder: Py<Builder>,
    num_elements: &Value,
    label: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    context::require_same(
        py,
        [builder.context(), num_elements.context(), label.context()],
    )?;
    unsafe { builder.get() }.build_tuple_record_output(
        unsafe { num_elements.get() }.try_into()?,
        unsafe { label.get() }.try_into()?,
    );
    Ok(())
}

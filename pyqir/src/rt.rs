// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    values::{Owner, Value},
};
use pyo3::prelude::*;
use qirlib::rt;

/// Inserts a marker in the generated output that indicates the start
/// of an array and how many array elements it has.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value num_elements: How many array elements the array has
/// :param Value label: A string label for the array. Depending on the output schema, the label is included in the output or omitted.
#[pyfunction]
#[pyo3(text_signature = "(builder, num_elements, label)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn array_record_output(
    py: Python,
    builder: Py<Builder>,
    num_elements: &Value,
    label: &Value,
) -> PyResult<()> {
    let builder: PyRef<Builder> = builder.borrow(py);
    Owner::merge(py, [builder.owner(), num_elements.owner(), label.owner()])?;
    unsafe {
        rt::build_array_record_output(
            builder.cast().as_ptr(),
            num_elements.cast().as_ptr(),
            label.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Adds a measurement result to the generated output.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value result: A result measurement to record
/// :param Value label: A string label for the result value. Depending on the output schema, the label is included in the output or omitted.
#[pyfunction]
#[pyo3(text_signature = "(builder, result, label)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn result_record_output(
    py: Python,
    builder: Py<Builder>,
    result: &Value,
    label: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), result.owner(), label.owner()])?;
    unsafe {
        rt::build_result_record_output(
            builder.cast().as_ptr(),
            result.cast().as_ptr(),
            label.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Initializes the execution environment. Sets all qubits to a zero-state
/// if they are not dynamically managed.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value data: For base profile QIR, a const null i8* Value should be passed.
#[pyfunction]
#[pyo3(text_signature = "(builder, data)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn initialize(py: Python, builder: Py<Builder>, data: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), data.owner()])?;
    unsafe {
        rt::build_initialize(builder.cast().as_ptr(), data.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a marker in the generated output that indicates the start
/// of a tuple and how many tuple elements it has.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value num_elements: How many tuple elements the tuple has
/// :param Value label: A string label for the tuple. Depending on the output schema, the label is included in the output or omitted.
#[pyfunction]
#[pyo3(text_signature = "(builder, num_elements, label)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn tuple_record_output(
    py: Python,
    builder: Py<Builder>,
    num_elements: &Value,
    label: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), num_elements.owner(), label.owner()])?;
    unsafe {
        rt::build_tuple_record_output(
            builder.cast().as_ptr(),
            num_elements.cast().as_ptr(),
            label.cast().as_ptr(),
        );
    }
    Ok(())
}

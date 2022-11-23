// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{builder::Builder, context, values::Value};
use pyo3::prelude::*;
use qirlib::rt::BuilderExt;
use std::convert::TryInto;

#[pyfunction]
pub fn array_record_output(
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
    unsafe { builder.get() }.build_array_record_output(
        unsafe { num_elements.get() }.try_into()?,
        unsafe { label.get() }.try_into()?,
    );
    Ok(())
}

#[pyfunction]
pub fn result_record_output(
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
    unsafe { builder.get() }.build_result_record_output(
        unsafe { num_elements.get() }.try_into()?,
        unsafe { label.get() }.try_into()?,
    );
    Ok(())
}

#[pyfunction]
pub fn initialize(py: Python, builder: Py<Builder>, reserved: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    context::require_same(py, [builder.context(), reserved.context()])?;
    unsafe { builder.get() }.build_initialize(unsafe { reserved.get() }.try_into()?);
    Ok(())
}

#[pyfunction]
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

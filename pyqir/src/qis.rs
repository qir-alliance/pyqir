// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    values::{Owner, Value},
};
use llvm_sys::{
    core::{LLVMConstReal, LLVMDoubleTypeInContext},
    prelude::*,
};
use pyo3::prelude::*;
use qirlib::qis;

/// Inserts a barrier instruction
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn barrier(py: Python, builder: Py<Builder>) {
    let builder = builder.borrow(py);
    unsafe {
        qis::build_barrier(builder.as_ptr());
    }
}

/// Inserts a swap gate
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit1: The first qubit to apply the gate to.
/// :param Value qubit2: The second qubit to apply the gate to.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit1, qubit2)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn swap(
    py: Python,
    builder: Py<Builder>,
    qubit1: &Value,
    qubit2: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit1.owner(), qubit2.owner()])?;
    unsafe {
        qis::build_swap(builder.as_ptr(), qubit1.as_ptr(), qubit2.as_ptr());
    }
    Ok(())
}

/// Inserts Toffoli or doubly-controlled :math:`X` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value control1: The first control qubit.
/// :param Value control2: The second control qubit.
/// :param Value target: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, control1, control2, target)")]
pub(crate) fn ccx(
    py: Python,
    builder: Py<Builder>,
    control1: &Value,
    control2: &Value,
    target: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(
        py,
        [
            builder.owner(),
            control1.owner(),
            control2.owner(),
            target.owner(),
        ],
    )?;
    unsafe {
        qis::build_ccx(
            builder.as_ptr(),
            control1.as_ptr(),
            control2.as_ptr(),
            target.as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a controlled Pauli :math:`X` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value control: The control qubit.
/// :param Value target: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, control, target)")]
pub(crate) fn cx(
    py: Python,
    builder: Py<Builder>,
    control: &Value,
    target: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), control.owner(), target.owner()])?;
    unsafe {
        qis::build_cx(builder.as_ptr(), control.as_ptr(), target.as_ptr());
    }
    Ok(())
}

/// Inserts a controlled Pauli :math:`Z` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value control: The control qubit.
/// :param Value target: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, control, target)")]
pub(crate) fn cz(
    py: Python,
    builder: Py<Builder>,
    control: &Value,
    target: &Value,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), control.owner(), target.owner()])?;
    unsafe {
        qis::build_cz(builder.as_ptr(), control.as_ptr(), target.as_ptr());
    }
    Ok(())
}

/// Inserts a Hadamard gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn h(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_h(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts a Z-basis measurement operation.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The qubit to measure.
/// :param Value result: A result where the measurement result will be written to.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit, result)")]
pub(crate) fn mz(py: Python, builder: Py<Builder>, qubit: &Value, result: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner(), result.owner()])?;
    unsafe {
        qis::build_mz(builder.as_ptr(), qubit.as_ptr(), result.as_ptr());
    }
    Ok(())
}

/// Inserts a reset operation.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The qubit to reset.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn reset(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_reset(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts a rotation gate about the :math:`x` axis.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param typing.Union[Value, float] theta: The angle to rotate by.
/// :param Value qubit: The qubit to rotate.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, theta, qubit)")]
pub(crate) fn rx(py: Python, builder: Py<Builder>, theta: Angle, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(
        py,
        [Some(builder.owner()), theta.owner(), Some(qubit.owner())]
            .into_iter()
            .flatten(),
    )?;

    let context = builder.owner().context(py);
    let context = context.borrow(py);
    unsafe {
        qis::build_rx(
            builder.as_ptr(),
            theta.to_value(context.as_ptr()),
            qubit.as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a rotation gate about the :math:`y` axis.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param typing.Union[Value, float] theta: The angle to rotate by.
/// :param Value qubit: The qubit to rotate.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, theta, qubit)")]
pub(crate) fn ry(py: Python, builder: Py<Builder>, theta: Angle, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(
        py,
        [Some(builder.owner()), theta.owner(), Some(qubit.owner())]
            .into_iter()
            .flatten(),
    )?;

    let context = builder.owner().context(py);
    let context = context.borrow(py);
    unsafe {
        qis::build_ry(
            builder.as_ptr(),
            theta.to_value(context.as_ptr()),
            qubit.as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a rotation gate about the :math:`z` axis.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param typing.Union[Value, float] theta: The angle to rotate by.
/// :param Value qubit: The qubit to rotate.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, theta, qubit)")]
pub(crate) fn rz(py: Python, builder: Py<Builder>, theta: Angle, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(
        py,
        [Some(builder.owner()), theta.owner(), Some(qubit.owner())]
            .into_iter()
            .flatten(),
    )?;

    let context = builder.owner().context(py);
    let context = context.borrow(py);
    unsafe {
        qis::build_rz(
            builder.as_ptr(),
            theta.to_value(context.as_ptr()),
            qubit.as_ptr(),
        );
    }
    Ok(())
}

/// Inserts an :math:`S` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn s(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_s(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts an adjoint :math:`S` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn s_adj(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_s_adj(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts a :math:`T` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn t(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_t(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts an adjoint :math:`T` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn t_adj(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_t_adj(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts a Pauli :math:`X` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn x(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_x(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts a Pauli :math:`Y` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn y(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_y(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts a Pauli :math:`Z` gate.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn z(py: Python, builder: Py<Builder>, qubit: &Value) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_z(builder.as_ptr(), qubit.as_ptr());
    }
    Ok(())
}

/// Inserts a branch conditioned on a measurement result.
///
/// Instructions inserted when ``one`` is called will be inserted into the one branch.
/// Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
/// and zero callables should use this module's builder to build instructions.
///
/// :param builder: The underlying builder used to build QIS instructions.
/// :param Value cond: The result condition to branch on.
/// :param typing.Callable[[], None] one:
///     A callable that inserts instructions for the branch where the result is one.
/// :param typing.Callable[[], None] zero:
///     A callable that inserts instructions for the branch where the result is zero.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, cond, one, zero)")]
pub(crate) fn if_result(
    py: Python,
    builder: Py<Builder>,
    cond: &Value,
    one: Option<&PyAny>,
    zero: Option<&PyAny>,
) -> PyResult<()> {
    let builder = builder.borrow(py);
    Owner::merge(py, [builder.owner(), cond.owner()])?;
    unsafe {
        qis::try_build_if_result(
            builder.as_ptr(),
            cond.as_ptr(),
            || one.iter().try_for_each(|f| f.call0().map(|_| ())),
            || zero.iter().try_for_each(|f| f.call0().map(|_| ())),
        )
    }
}

#[derive(FromPyObject)]
pub(crate) enum Angle<'py> {
    Value(PyRef<'py, Value>),
    Constant(f64),
}

impl Angle<'_> {
    fn owner(&self) -> Option<&Owner> {
        match self {
            Angle::Value(v) => Some(v.owner()),
            Angle::Constant(_) => None,
        }
    }

    unsafe fn to_value(&self, context: LLVMContextRef) -> LLVMValueRef {
        match self {
            Angle::Value(v) => v.as_ptr(),
            &Angle::Constant(c) => LLVMConstReal(LLVMDoubleTypeInContext(context), c),
        }
    }
}

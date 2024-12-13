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
/// :param Builder builder: The IR Builder used to create the instructions
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn barrier(builder: &Builder) {
    unsafe {
        qis::build_barrier(builder.cast().as_ptr());
    }
}

/// Inserts a swap gate
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit1: The first qubit to apply the gate to.
/// :param Value qubit2: The second qubit to apply the gate to.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit1, qubit2)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn swap(py: Python, builder: &Builder, qubit1: &Value, qubit2: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit1.owner(), qubit2.owner()])?;
    unsafe {
        qis::build_swap(
            builder.cast().as_ptr(),
            qubit1.cast().as_ptr(),
            qubit2.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts Toffoli or doubly-controlled :math:`X` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value control1: The first control qubit.
/// :param Value control2: The second control qubit.
/// :param Value target: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, control1, control2, target)")]
pub(crate) fn ccx(
    py: Python,
    builder: &Builder,
    control1: &Value,
    control2: &Value,
    target: &Value,
) -> PyResult<()> {
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
            builder.cast().as_ptr(),
            control1.cast().as_ptr(),
            control2.cast().as_ptr(),
            target.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a controlled Pauli :math:`X` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value control: The control qubit.
/// :param Value target: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, control, target)")]
pub(crate) fn cx(py: Python, builder: &Builder, control: &Value, target: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), control.owner(), target.owner()])?;
    unsafe {
        qis::build_cx(
            builder.cast().as_ptr(),
            control.cast().as_ptr(),
            target.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a controlled Pauli :math:`Z` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value control: The control qubit.
/// :param Value target: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, control, target)")]
pub(crate) fn cz(py: Python, builder: &Builder, control: &Value, target: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), control.owner(), target.owner()])?;
    unsafe {
        qis::build_cz(
            builder.cast().as_ptr(),
            control.cast().as_ptr(),
            target.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a Hadamard gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn h(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_h(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a Z-basis measurement operation.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The qubit to measure.
/// :param Value result: A result where the measurement result will be written to.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit, result)")]
pub(crate) fn mz(py: Python, builder: &Builder, qubit: &Value, result: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner(), result.owner()])?;
    unsafe {
        qis::build_mz(
            builder.cast().as_ptr(),
            qubit.cast().as_ptr(),
            result.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a reset operation.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The qubit to reset.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn reset(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_reset(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a rotation gate about the :math:`x` axis.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param typing.Union[Value, float] theta: The angle to rotate by.
/// :param Value qubit: The qubit to rotate.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, theta, qubit)")]
pub(crate) fn rx(py: Python, builder: &Builder, theta: Angle, qubit: &Value) -> PyResult<()> {
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
            builder.cast().as_ptr(),
            theta.to_value(context.cast().as_ptr()),
            qubit.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a rotation gate about the :math:`y` axis.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param typing.Union[Value, float] theta: The angle to rotate by.
/// :param Value qubit: The qubit to rotate.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, theta, qubit)")]
pub(crate) fn ry(py: Python, builder: &Builder, theta: Angle, qubit: &Value) -> PyResult<()> {
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
            builder.cast().as_ptr(),
            theta.to_value(context.cast().as_ptr()),
            qubit.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts a rotation gate about the :math:`z` axis.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param typing.Union[Value, float] theta: The angle to rotate by.
/// :param Value qubit: The qubit to rotate.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, theta, qubit)")]
pub(crate) fn rz(py: Python, builder: &Builder, theta: Angle, qubit: &Value) -> PyResult<()> {
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
            builder.cast().as_ptr(),
            theta.to_value(context.cast().as_ptr()),
            qubit.cast().as_ptr(),
        );
    }
    Ok(())
}

/// Inserts an :math:`S` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn s(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_s(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts an adjoint :math:`S` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn s_adj(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_s_adj(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a :math:`T` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn t(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_t(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts an adjoint :math:`T` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn t_adj(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_t_adj(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a Pauli :math:`X` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn x(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_x(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a Pauli :math:`Y` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn y(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_y(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a Pauli :math:`Z` gate.
///
/// :param Builder builder: The IR Builder used to create the instructions
/// :param Value qubit: The target qubit.
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder, qubit)")]
pub(crate) fn z(py: Python, builder: &Builder, qubit: &Value) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), qubit.owner()])?;
    unsafe {
        qis::build_z(builder.cast().as_ptr(), qubit.cast().as_ptr());
    }
    Ok(())
}

/// Inserts a branch conditioned on a measurement result.
///
/// Instructions inserted when ``one`` is called will be inserted into the one branch.
/// Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
/// and zero callables should use this module's builder to build instructions.
///
/// :param Builder builder: The IR Builder used to create the instructions
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
    builder: &Builder,
    cond: &Value,
    one: Option<&PyAny>,
    zero: Option<&PyAny>,
) -> PyResult<()> {
    Owner::merge(py, [builder.owner(), cond.owner()])?;
    unsafe {
        qis::try_build_if_result(
            builder.cast().as_ptr(),
            cond.cast().as_ptr(),
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
            Angle::Value(v) => v.cast().as_ptr(),
            &Angle::Constant(c) => LLVMConstReal(LLVMDoubleTypeInContext(context), c),
        }
    }
}

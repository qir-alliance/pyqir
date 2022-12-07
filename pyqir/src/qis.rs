// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    values::{ConvertError, Owner, Value},
};
use inkwell::values::FloatValue;
use pyo3::prelude::*;
use qirlib::qis::BuilderExt as qis;
use std::convert::TryInto;

/// An instruction builder that generates instructions from the basic quantum instruction set.
///
/// :param Builder builder: The underlying builder used to build QIS instructions.
#[pyclass]
#[pyo3(text_signature = "(builder)")]
pub(crate) struct BasicQisBuilder {
    builder: Py<Builder>,
}

/// Inserts a barrier instruction
///
/// :rtype: None
#[pyfunction]
#[pyo3(text_signature = "(builder)")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn barrier(py: Python, builder: Py<Builder>) {
    let builder = builder.borrow(py);
    unsafe { builder.get() }.build_barrier();
}

/// Inserts a swap gate
///
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
    unsafe { builder.get() }.build_swap(
        unsafe { qubit1.get() }.try_into()?,
        unsafe { qubit2.get() }.try_into()?,
    );
    Ok(())
}

#[pymethods]
impl BasicQisBuilder {
    #[new]
    fn new(builder: Py<Builder>) -> Self {
        BasicQisBuilder { builder }
    }

    /// Inserts Toffoli or doubly-controlled :math:`X` gate.
    ///
    /// :param Value control1: The first control qubit.
    /// :param Value control2: The second control qubit.
    /// :param Value target: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, control1, control2, target)")]
    fn ccx(&self, py: Python, control1: &Value, control2: &Value, target: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(
            py,
            [
                builder.owner(),
                control1.owner(),
                control2.owner(),
                target.owner(),
            ],
        )?;
        unsafe { builder.get() }.build_ccx(
            unsafe { control1.get() }.try_into()?,
            unsafe { control2.get() }.try_into()?,
            unsafe { target.get() }.try_into()?,
        );
        Ok(())
    }

    /// Inserts a controlled Pauli :math:`X` gate.
    ///
    /// :param Value control: The control qubit.
    /// :param Value target: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, control, target)")]
    fn cx(&self, py: Python, control: &Value, target: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), control.owner(), target.owner()])?;
        unsafe { builder.get() }.build_cx(
            unsafe { control.get() }.try_into()?,
            unsafe { target.get() }.try_into()?,
        );
        Ok(())
    }

    /// Inserts a controlled Pauli :math:`Z` gate.
    ///
    /// :param Value control: The control qubit.
    /// :param Value target: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, control, target)")]
    fn cz(&self, py: Python, control: &Value, target: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), control.owner(), target.owner()])?;
        unsafe { builder.get() }.build_cz(
            unsafe { control.get() }.try_into()?,
            unsafe { target.get() }.try_into()?,
        );
        Ok(())
    }

    /// Inserts a Hadamard gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn h(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_h(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a Z-basis measurement operation.
    ///
    /// :param Value qubit: The qubit to measure.
    /// :param Value result: A result where the measurement result will be written to.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit, result)")]
    fn mz(&self, py: Python, qubit: &Value, result: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner(), result.owner()])?;
        unsafe { builder.get() }.build_mz(
            unsafe { qubit.get() }.try_into()?,
            unsafe { result.get() }.try_into()?,
        );
        Ok(())
    }

    /// Inserts a reset operation.
    ///
    /// :param Value qubit: The qubit to reset.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn reset(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_reset(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`x` axis.
    ///
    /// :param typing.Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rx(&self, py: Python, theta: Angle, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(
            py,
            [Some(builder.owner()), theta.owner(), Some(qubit.owner())]
                .into_iter()
                .flatten(),
        )?;

        let context = builder.owner().context(py);
        let context = context.borrow(py);
        unsafe {
            builder
                .get()
                .build_rx(theta.to_float(&context)?, qubit.get().try_into()?);
        }

        Ok(())
    }

    /// Inserts a rotation gate about the :math:`y` axis.
    ///
    /// :param typing.Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn ry(&self, py: Python, theta: Angle, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(
            py,
            [Some(builder.owner()), theta.owner(), Some(qubit.owner())]
                .into_iter()
                .flatten(),
        )?;

        let context = builder.owner().context(py);
        let context = context.borrow(py);
        unsafe {
            builder
                .get()
                .build_ry(theta.to_float(&context)?, qubit.get().try_into()?);
        }

        Ok(())
    }

    /// Inserts a rotation gate about the :math:`z` axis.
    ///
    /// :param typing.Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rz(&self, py: Python, theta: Angle, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(
            py,
            [Some(builder.owner()), theta.owner(), Some(qubit.owner())]
                .into_iter()
                .flatten(),
        )?;

        let context = builder.owner().context(py);
        let context = context.borrow(py);
        unsafe {
            builder
                .get()
                .build_rz(theta.to_float(&context)?, qubit.get().try_into()?);
        }

        Ok(())
    }

    /// Inserts an :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_s(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts an adjoint :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s_adj(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_s_adj(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a :math:`T` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_t(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts an adjoint :math:`T` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t_adj(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_t_adj(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a Pauli :math:`X` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn x(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_x(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a Pauli :math:`Y` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn y(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_y(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a Pauli :math:`Z` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn z(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), qubit.owner()])?;
        unsafe { builder.get() }.build_z(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a branch conditioned on a measurement result.
    ///
    /// Instructions inserted when ``one`` is called will be inserted into the one branch.
    /// Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
    /// and zero callables should use this module's builder to build instructions.
    ///
    /// :param Value cond: The result condition to branch on.
    /// :param typing.Callable[[], None] one:
    ///     A callable that inserts instructions for the branch where the result is one.
    /// :param typing.Callable[[], None] zero:
    ///     A callable that inserts instructions for the branch where the result is zero.
    /// :rtype: None
    #[pyo3(text_signature = "(self, cond, one, zero)")]
    fn if_result(
        &self,
        py: Python,
        cond: &Value,
        one: Option<&PyAny>,
        zero: Option<&PyAny>,
    ) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        Owner::merge(py, [builder.owner(), cond.owner()])?;
        unsafe { builder.get() }.try_build_if_result(
            unsafe { cond.get() }.try_into()?,
            || one.iter().try_for_each(|f| f.call0().map(|_| ())),
            || zero.iter().try_for_each(|f| f.call0().map(|_| ())),
        )
    }
}

#[derive(FromPyObject)]
enum Angle<'py> {
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

    unsafe fn to_float<'ctx>(
        &self,
        context: &'ctx inkwell::context::Context,
    ) -> Result<FloatValue<'ctx>, ConvertError> {
        match self {
            Angle::Value(v) => v.get().try_into(),
            &Angle::Constant(c) => Ok(context.f64_type().const_float(c)),
        }
    }
}

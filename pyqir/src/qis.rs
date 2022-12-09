// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    values::{ConvertError, Owner, Value},
};
use inkwell::{values::FloatValue, LLVMReference};
use pyo3::prelude::*;
use qirlib::qis;
use std::convert::TryInto;

/// An instruction builder that generates instructions from the basic quantum instruction set.
///
/// :param Builder builder: The underlying builder used to build QIS instructions.
#[pyclass]
#[pyo3(text_signature = "(builder)")]
pub(crate) struct BasicQisBuilder {
    builder: Py<Builder>,
}

#[pymethods]
impl BasicQisBuilder {
    #[new]
    fn new(builder: Py<Builder>) -> Self {
        BasicQisBuilder { builder }
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
        unsafe {
            qis::build_cx(
                builder.get().get_ref(),
                control.get().get_ref(),
                target.get().get_ref(),
            );
        }
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
        unsafe {
            qis::build_cz(
                builder.get().get_ref(),
                control.get().get_ref(),
                target.get().get_ref(),
            );
        }
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
        unsafe {
            qis::build_h(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::build_mz(
                builder.get().get_ref(),
                qubit.get().get_ref(),
                result.get().get_ref(),
            );
        }
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
        unsafe {
            qis::build_reset(builder.get().get_ref(), qubit.get().get_ref());
        }
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
            qis::build_rx(
                builder.get().get_ref(),
                theta.to_float(&context)?.get_ref(),
                qubit.get().get_ref(),
            );
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
            qis::build_ry(
                builder.get().get_ref(),
                theta.to_float(&context)?.get_ref(),
                qubit.get().get_ref(),
            );
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
            qis::build_rz(
                builder.get().get_ref(),
                theta.to_float(&context)?.get_ref(),
                qubit.get().get_ref(),
            );
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
        unsafe {
            qis::build_s(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::build_s_adj(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::build_t(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::build_t_adj(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::build_x(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::build_y(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::build_z(builder.get().get_ref(), qubit.get().get_ref());
        }
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
        unsafe {
            qis::try_build_if_result(
                builder.get().get_ref(),
                cond.get().get_ref(),
                || one.iter().try_for_each(|f| f.call0().map(|_| ())),
                || zero.iter().try_for_each(|f| f.call0().map(|_| ())),
            )
        }
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

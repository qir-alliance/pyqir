// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    builder::Builder,
    context,
    utils::call_if_some,
    values::{self, Value},
};
use inkwell::values::PointerValue;
use pyo3::prelude::*;
use qirlib::BuilderBasicQisExt;
use std::{convert::TryInto, mem::transmute};

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
        context::require_same(py, [builder.context(), control.context(), target.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_cx(
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
        context::require_same(py, [builder.context(), control.context(), target.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_cz(
            unsafe { control.get() }.try_into()?,
            unsafe { target.get() }.try_into()?,
        );
        Ok(())
    }

    /// Inserts a Hadamard gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn h(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_h(unsafe { qubit.get() }.try_into()?);
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
        context::require_same(py, [builder.context(), qubit.context(), result.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_mz(
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
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_reset(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`x` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rx(&self, py: Python, theta: &PyAny, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(
            py,
            values::extract_contexts([theta])
                .chain([builder.context().clone(), qubit.context().clone()]),
        )?;

        let context = builder.context().borrow(py);
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        let theta = unsafe { values::extract_any(&context.f64_type(), theta)? };
        builder.build_rx(theta.try_into()?, unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`y` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn ry(&self, py: Python, theta: &PyAny, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(
            py,
            values::extract_contexts([theta])
                .chain([builder.context().clone(), qubit.context().clone()]),
        )?;

        let context = builder.context().borrow(py);
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        let theta = unsafe { values::extract_any(&context.f64_type(), theta)? };
        builder.build_ry(theta.try_into()?, unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`z` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rz(&self, py: Python, theta: &PyAny, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(
            py,
            values::extract_contexts([theta])
                .chain([builder.context().clone(), qubit.context().clone()]),
        )?;

        let context = builder.context().borrow(py);
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        let theta = unsafe { values::extract_any(&context.f64_type(), theta)? };
        builder.build_rz(theta.try_into()?, unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts an :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_s(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts an adjoint :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s_adj(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_s_adj(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a :math:`T` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_t(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts an adjoint :math:`T` gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t_adj(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_t_adj(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a Pauli :math:`X` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn x(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_x(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a Pauli :math:`Y` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn y(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_y(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a Pauli :math:`Z` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn z(&self, py: Python, qubit: &Value) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        context::require_same(py, [builder.context(), qubit.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        builder.build_z(unsafe { qubit.get() }.try_into()?);
        Ok(())
    }

    /// Inserts a branch conditioned on a measurement result.
    ///
    /// Instructions inserted when ``one`` is called will be inserted into the one branch.
    /// Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
    /// and zero callables should use this module's builder to build instructions.
    ///
    /// :param Value cond: The result condition to branch on.
    /// :param Callable[[], None] one:
    ///     A callable that inserts instructions for the branch where the result is one.
    /// :param Callable[[], None] zero:
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
        context::require_same(py, [builder.context(), cond.context()])?;
        let module = builder.module().borrow(py);
        let builder = unsafe { qirlib::Builder::from(builder.get(), module.get()) };
        let cond: PointerValue = unsafe { cond.get() }.try_into()?;
        let cond = unsafe { transmute::<PointerValue<'_>, PointerValue<'static>>(cond) };
        builder.try_build_if_result(cond, |_| call_if_some(one), |_| call_if_some(zero))
    }
}

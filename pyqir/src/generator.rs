// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
//
// Safety
// ------
//
// To store Inkwell/LLVM objects in Python classes, we transmute the 'ctx lifetime to static. You
// need to be careful when using Inkwell types with unsafely extended lifetimes. Follow these rules:
//
// 1. When storing in a data type, always include a Py<Context> field containing the context
//    originally referred to by 'ctx.
// 2. Before calling Inkwell methods that use 'ctx, call Context::require_same to assert that all
//    contexts being used are the same.

#![allow(clippy::used_underscore_binding)]

use crate::{
    builder::Builder,
    context,
    types::Type,
    utils::{any_to_meta, call_if_some, extract_constant},
    values::{self, Value},
};
use inkwell::{attributes::Attribute as InkwellAttribute, values::PointerValue};
use pyo3::{
    exceptions::PyOSError,
    prelude::*,
    types::{PyBytes, PyString, PyUnicode},
};
use qirlib::{module, BuilderBasicQisExt};
use std::{convert::TryInto, mem::transmute};

#[pyclass(unsendable)]
pub(crate) struct Attribute(pub(crate) InkwellAttribute);

#[pymethods]
impl Attribute {
    #[getter]
    fn value(&self) -> &str {
        self.0
            .get_string_value()
            .to_str()
            .expect("Value is not valid UTF-8.")
    }
}

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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_cx(control.get().try_into()?, target.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_cz(control.get().try_into()?, target.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_h(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_mz(qubit.get().try_into()?, result.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_reset(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        let theta = unsafe { values::extract_inkwell(&context.f64_type(), theta)? };
        builder.build_rx(
            any_to_meta(theta).unwrap().into_float_value(),
            qubit.get().try_into()?,
        );
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        let theta = unsafe { values::extract_inkwell(&context.f64_type(), theta)? };
        builder.build_ry(
            any_to_meta(theta).unwrap().into_float_value(),
            qubit.get().try_into()?,
        );
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        let theta = unsafe { values::extract_inkwell(&context.f64_type(), theta)? };
        builder.build_rz(
            any_to_meta(theta).unwrap().into_float_value(),
            qubit.get().try_into()?,
        );
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_s(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_s_adj(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_t(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_t_adj(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_x(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_y(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        builder.build_z(qubit.get().try_into()?);
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
        let builder = qirlib::Builder::from(builder.get(), module.get());
        let cond: PointerValue = cond.get().try_into()?;
        let cond = unsafe { transmute::<PointerValue<'_>, PointerValue<'static>>(cond) };
        builder.try_build_if_result(cond, |_| call_if_some(one), |_| call_if_some(zero))
    }
}

/// Creates a constant value.
///
/// :param Type ty: The type of the value.
/// :param Union[int, float] value: The value of the constant.
/// :returns: The constant value.
/// :rtype: Value
#[pyfunction]
#[pyo3(text_signature = "(ty, value)")]
pub(crate) fn r#const(py: Python, ty: &Type, value: &PyAny) -> PyResult<PyObject> {
    let context = ty.context().clone();
    let value = extract_constant(&ty.get(), value)?;
    unsafe { Value::from_any(py, context, value) }
}

/// Converts the supplied QIR string to its bitcode equivalent.
///
/// :param str ir: The QIR string to convert
/// :param Optional[str] module_name: The name of the QIR module, default is "" if None
/// :param Optional[str] source_file_name: The source file name of the QIR module. Unchanged if None
/// :return: The equivalent bitcode as bytes.
/// :rtype: bytes
#[pyfunction]
#[pyo3(text_signature = "(ir, module_name=None, source_file_name=None)")]
pub(crate) fn ir_to_bitcode<'a>(
    py: Python<'a>,
    ir: &str,
    module_name: Option<&str>,
    source_file_name: Option<&str>,
) -> PyResult<&'a PyBytes> {
    let bitcode =
        module::ir_to_bitcode(ir, module_name, source_file_name).map_err(PyOSError::new_err)?;
    Ok(PyBytes::new(py, &bitcode))
}

/// Converts the supplied bitcode to its QIR string equivalent.
///
/// :param bytes ir: The bitcode bytes to convert
/// :param Optional[str] module_name: The name of the QIR module, default is "" if None
/// :param Optional[str] source_file_name: The source file name of the QIR module. Unchanged if None
/// :return: The equivalent QIR string.
/// :rtype: str
#[pyfunction]
#[pyo3(text_signature = "(bitcode, module_name=None, source_file_name=None)")]
pub(crate) fn bitcode_to_ir<'a>(
    py: Python<'a>,
    bitcode: &PyBytes,
    module_name: Option<&str>,
    source_file_name: Option<&str>,
) -> PyResult<&'a PyString> {
    let ir = module::bitcode_to_ir(bitcode.as_bytes(), module_name, source_file_name)
        .map_err(PyOSError::new_err)?;
    Ok(PyUnicode::new(py, &ir))
}

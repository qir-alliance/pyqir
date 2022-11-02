// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    context::{self, Context},
    instructions::IntPredicate,
    module::Module,
    utils::{call_if_some, try_callable_value},
    values::{self, Value},
};
use inkwell::values::IntValue;
use pyo3::{exceptions::PyValueError, prelude::*, types::PySequence};
use std::{
    convert::{Into, TryInto},
    mem::transmute,
    result::Result,
};

/// An instruction builder.
#[pyclass(unsendable)]
pub(crate) struct Builder {
    builder: inkwell::builder::Builder<'static>,
    context: Py<Context>,
    // TODO: In principle, the module could be extracted from the builder.
    // See https://github.com/TheDan64/inkwell/issues/347.
    module: Py<Module>,
}

#[pymethods]
impl Builder {
    /// Inserts a bitwise logical and instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn and_(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_and::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts a bitwise logical or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn or_(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_or::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts a bitwise logical exclusive or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn xor(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_xor::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts an addition instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The sum.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn add(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_int_add::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts a subtraction instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The difference.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn sub(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_int_sub::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts a multiplication instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The product.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn mul(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_int_mul::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts a shift left instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn shl(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_left_shift::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts a logical (zero fill) shift right instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn lshr(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_right_shift::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            false,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts an integer comparison instruction.
    ///
    /// :param IntPredicate pred: The predicate to compare by.
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :return: The boolean result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, pred, lhs, rhs)")]
    #[allow(clippy::needless_pass_by_value)]
    fn icmp(&self, py: Python, pred: IntPredicate, lhs: Value, rhs: Value) -> PyResult<PyObject> {
        context::require_same(py, [&self.context, lhs.context(), rhs.context()])?;
        let value = self.builder.build_int_compare::<IntValue>(
            pred.into(),
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, self.context.clone(), value) }
    }

    /// Inserts a call instruction.
    ///
    /// :param Value value: The value to call.
    /// :param Sequence[Union[Value, bool, int, float]] args: The arguments to the function.
    /// :returns: The return value, or None if the function has a void return type.
    /// :rtype: Optional[Value]
    #[pyo3(text_signature = "(self, callee, args)")]
    fn call(&self, py: Python, callee: &Value, args: &PySequence) -> PyResult<Option<PyObject>> {
        context::require_same(
            py,
            values::extract_contexts(args.iter()?.filter_map(Result::ok))
                .chain([self.context.clone(), callee.context().clone()]),
        )?;

        let (callable, param_types) = try_callable_value(unsafe { callee.get() })
            .ok_or_else(|| PyValueError::new_err("Value is not callable."))?;

        if param_types.len() != args.len()? {
            return Err(PyValueError::new_err(format!(
                "Expected {} arguments, got {}.",
                param_types.len(),
                args.len()?
            )));
        }

        let args = args
            .iter()?
            .zip(param_types)
            .map(|(v, t)| {
                unsafe { values::extract_any(&t, v?) }?
                    .try_into()
                    .map_err(Into::into)
            })
            .collect::<PyResult<Vec<_>>>()?;

        let call = self.builder.build_call(callable, &args, "");
        let value = call.try_as_basic_value().left();
        value
            .map(|v| unsafe { Value::from_any(py, callee.context().clone(), v) })
            .transpose()
    }

    /// Inserts a branch conditioned on a boolean.
    ///
    /// Instructions inserted when ``true`` is called will be inserted into the true branch.
    /// Instructions inserted when ``false`` is called will be inserted into the false branch. The
    /// true and false callables should use this module's builder to build instructions.
    ///
    /// :param Value cond: The boolean condition to branch on.
    /// :param Callable[[], None] true:
    ///     A callable that inserts instructions for the branch where the condition is true.
    /// :param Callable[[], None] false:
    ///     A callable that inserts instructions for the branch where the condition is false.
    #[pyo3(text_signature = "(self, cond, true, false)")]
    fn if_(
        &self,
        py: Python,
        cond: &Value,
        r#true: Option<&PyAny>,
        r#false: Option<&PyAny>,
    ) -> PyResult<()> {
        context::require_same(py, [&self.context, cond.context()])?;
        let module = self.module.borrow(py);
        let builder = qirlib::Builder::from(&self.builder, unsafe { module.get() });
        builder.try_build_if(
            unsafe { cond.get() }.try_into()?,
            |_| call_if_some(r#true),
            |_| call_if_some(r#false),
        )
    }
}

impl Builder {
    pub(crate) fn new(py: Python, module: Py<Module>) -> Self {
        let context = module.borrow(py).context().clone();
        let builder = {
            let context = context.borrow(py);
            let builder = context.create_builder();
            unsafe {
                transmute::<inkwell::builder::Builder<'_>, inkwell::builder::Builder<'static>>(
                    builder,
                )
            }
        };

        Self {
            builder,
            context,
            module,
        }
    }

    pub(crate) unsafe fn get(&self) -> &inkwell::builder::Builder<'static> {
        &self.builder
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }

    pub(crate) fn module(&self) -> &Py<Module> {
        &self.module
    }
}

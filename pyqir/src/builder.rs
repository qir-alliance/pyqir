// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    context::Context,
    instructions::IntPredicate,
    types,
    values::{AnyValue, BasicBlock, Literal, Owner, Value},
};
use inkwell::{
    types::{AnyTypeEnum, BasicTypeEnum, FunctionType},
    values::{AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum, CallableValue, IntValue},
};
use pyo3::{exceptions::PyValueError, prelude::*};
use qirlib::builder::Ext;
use std::{
    convert::{Into, TryFrom, TryInto},
    mem::transmute,
    result::Result,
};

/// An instruction builder.
///
/// :param Context context: The global context.
#[pyclass(unsendable)]
pub(crate) struct Builder {
    builder: inkwell::builder::Builder<'static>,
    owner: Owner,
}

#[pymethods]
impl Builder {
    #[new]
    pub(crate) fn new(py: Python, context: Py<Context>) -> Self {
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
            owner: context.into(),
        }
    }

    /// Tells this builder to insert subsequent instructions at the end of the block.
    ///
    /// :param BasicBlock block: The block to insert into.
    /// :rtype: None
    #[pyo3(text_signature = "(block)")]
    fn insert_at_end(&mut self, py: Python, block: PyRef<BasicBlock>) -> PyResult<()> {
        let owner = block.as_ref().owner();
        if *owner.context(py).borrow(py) != *self.owner.context(py).borrow(py) {
            Err(PyValueError::new_err(
                "Block is not from the same context as builder.",
            ))?;
        }

        self.owner = owner.clone_ref(py);
        self.builder.position_at_end(unsafe { block.get() });
        Ok(())
    }

    /// Inserts a bitwise logical and instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn and_(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_and::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts a bitwise logical or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn or_(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_or::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts a bitwise logical exclusive or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn xor(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_xor::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts an addition instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The sum.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn add(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_int_add::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts a subtraction instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The difference.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn sub(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_int_sub::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts a multiplication instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The product.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn mul(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_int_mul::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts a shift left instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn shl(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_left_shift::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts a logical (zero fill) shift right instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn lshr(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_right_shift::<IntValue>(
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            false,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts an integer comparison instruction.
    ///
    /// :param IntPredicate pred: The predicate to compare by.
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :return: The boolean result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, pred, lhs, rhs)")]
    fn icmp(&self, py: Python, pred: IntPredicate, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        let value = self.builder.build_int_compare::<IntValue>(
            pred.into(),
            unsafe { lhs.get() }.try_into()?,
            unsafe { rhs.get() }.try_into()?,
            "",
        );
        unsafe { Value::from_any(py, owner, value) }
    }

    /// Inserts a call instruction.
    ///
    /// :param Value value: The value to call.
    /// :param typing.Sequence[typing.Union[Value, bool, int, float]] args:
    ///     The arguments to the function.
    /// :returns: The return value, or None if the function has a void return type.
    /// :rtype: Optional[Value]
    #[pyo3(text_signature = "(self, callee, args)")]
    fn call(&self, py: Python, callee: &Value, args: Vec<Argument>) -> PyResult<Option<PyObject>> {
        let arg_owners = args.iter().filter_map(Argument::owner);
        let owner = Owner::merge(py, arg_owners.chain([&self.owner, callee.owner()]))?;

        let callable: Callable = unsafe { callee.get() }.try_into()?;
        let param_types = callable.ty.get_param_types();
        if param_types.len() != args.len() {
            Err(PyValueError::new_err(format!(
                "Expected {} arguments, got {}.",
                param_types.len(),
                args.len()
            )))?;
        }

        let args = args
            .iter()
            .zip(param_types)
            .map(|(arg, ty)| unsafe { arg.to_value(ty) }.map_err(Into::into))
            .collect::<PyResult<Vec<_>>>()?;

        let call = self.builder.build_call(callable.value, &args, "");
        let value = call.try_as_basic_value().left();
        value
            .map(|v| unsafe { Value::from_any(py, owner, v) })
            .transpose()
    }

    /// Inserts a branch conditioned on a boolean.
    ///
    /// Instructions inserted when ``true`` is called will be inserted into the true branch.
    /// Instructions inserted when ``false`` is called will be inserted into the false branch. The
    /// true and false callables should use this module's builder to build instructions.
    ///
    /// :param Value cond: The boolean condition to branch on.
    /// :param typing.Callable[[], None] true:
    ///     A callable that inserts instructions for the branch where the condition is true.
    /// :param typing.Callable[[], None] false:
    ///     A callable that inserts instructions for the branch where the condition is false.
    #[pyo3(text_signature = "(self, cond, true, false)")]
    fn if_(
        &self,
        py: Python,
        cond: &Value,
        r#true: Option<&PyAny>,
        r#false: Option<&PyAny>,
    ) -> PyResult<()> {
        Owner::merge(py, [&self.owner, cond.owner()])?;
        self.builder.try_build_if(
            unsafe { cond.get() }.try_into()?,
            || r#true.iter().try_for_each(|f| f.call0().map(|_| ())),
            || r#false.iter().try_for_each(|f| f.call0().map(|_| ())),
        )
    }

    /// Inserts an unconditional branch instruction.
    ///
    /// :param BasicBlock dest: The destination block.
    /// :returns: The branch instruction.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(dest)")]
    fn br(&self, py: Python, dest: PyRef<BasicBlock>) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, dest.as_ref().owner()])?;
        let inst = self
            .builder
            .build_unconditional_branch(unsafe { dest.get() });
        unsafe { Value::from_any(py, owner, inst) }
    }

    /// Inserts a return instruction.
    ///
    /// :param Value value: The value to return. If `None`, returns void.
    /// :returns: The return instruction.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(value)")]
    fn ret(&self, py: Python, value: Option<&Value>) -> PyResult<PyObject> {
        let (inst, owner) = match value {
            None => (self.builder.build_return(None), self.owner.clone_ref(py)),
            Some(value) => {
                let owner = Owner::merge(py, [&self.owner, value.owner()])?;
                let value = BasicValueEnum::try_from(unsafe { value.get() })?;
                (self.builder.build_return(Some(&value)), owner)
            }
        };
        unsafe { Value::from_any(py, owner, inst) }
    }
}

impl Builder {
    pub(crate) unsafe fn get(&self) -> &inkwell::builder::Builder<'static> {
        &self.builder
    }

    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }
}

struct Callable<'ctx> {
    value: CallableValue<'ctx>,
    ty: FunctionType<'ctx>,
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for Callable<'ctx> {
    type Error = PyErr;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::FunctionValue(f)) => Some(Self {
                value: CallableValue::from(f),
                ty: f.get_type(),
            }),
            AnyValue::Any(AnyValueEnum::PointerValue(p)) => match p.get_type().get_element_type() {
                AnyTypeEnum::FunctionType(ty) => Some(Self {
                    value: CallableValue::try_from(p).unwrap(),
                    ty,
                }),
                _ => None,
            },
            _ => None,
        }
        .ok_or_else(|| PyValueError::new_err("Value is not callable."))
    }
}

#[derive(FromPyObject)]
enum Argument<'py> {
    Value(PyRef<'py, Value>),
    Literal(Literal<'py>),
}

impl Argument<'_> {
    fn owner(&self) -> Option<&Owner> {
        match self {
            Argument::Value(v) => Some(v.owner()),
            Argument::Literal(_) => None,
        }
    }

    unsafe fn to_value(&self, ty: BasicTypeEnum<'static>) -> PyResult<BasicMetadataValueEnum> {
        match self {
            Argument::Value(v) => v.get().try_into().map_err(Into::into),
            Argument::Literal(l) => l
                .to_value(types::basic_to_any(ty))?
                .try_into()
                .map_err(Into::into),
        }
    }
}

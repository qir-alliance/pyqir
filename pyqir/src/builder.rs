// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    context::Context,
    instructions::IntPredicate,
    values::{AnyValue, BasicBlock, Literal, Owner, Value},
};
use inkwell::{
    types::{AnyTypeEnum, BasicTypeEnum, FunctionType},
    values::{AnyValueEnum, BasicMetadataValueEnum, CallSiteValue, InstructionValue},
    LLVMReference,
};
use libc::c_char;
use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildAnd, LLVMBuildBr, LLVMBuildCall, LLVMBuildICmp, LLVMBuildLShr,
        LLVMBuildMul, LLVMBuildOr, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildShl, LLVMBuildSub,
        LLVMBuildXor, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMPositionBuilderAtEnd,
    },
    prelude::*,
};
use pyo3::{exceptions::PyValueError, prelude::*};
use qirlib::builder::try_build_if;
use std::{
    convert::{Into, TryFrom, TryInto},
    ops::Deref,
    result::Result,
};

const NO_NAME: *const c_char = b"\0".as_ptr().cast();

/// An instruction builder.
///
/// :param Context context: The LLVM context.
#[pyclass(unsendable)]
pub(crate) struct Builder {
    builder: LLVMBuilderRef,
    owner: Owner,
}

#[pymethods]
impl Builder {
    #[new]
    pub(crate) fn new(py: Python, context: Py<Context>) -> Self {
        let builder = unsafe { LLVMCreateBuilderInContext(context.borrow(py).get_ref()) };
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
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, block.get().get_ref());
        }
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
        unsafe {
            let value = LLVMBuildAnd(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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
        unsafe {
            let value = LLVMBuildOr(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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
        unsafe {
            let value = LLVMBuildXor(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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
        unsafe {
            let value = LLVMBuildAdd(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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
        unsafe {
            let value = LLVMBuildSub(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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
        unsafe {
            let value = LLVMBuildMul(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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
        unsafe {
            let value = LLVMBuildShl(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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
        unsafe {
            let value = LLVMBuildLShr(
                self.builder,
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
    }

    /// Inserts an integer comparison instruction.
    ///
    /// :param IntPredicate pred: The predicate to compare by.
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The boolean result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, pred, lhs, rhs)")]
    fn icmp(&self, py: Python, pred: IntPredicate, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildICmp(
                self.builder,
                inkwell::IntPredicate::from(pred).into(),
                lhs.get().get_ref(),
                rhs.get().get_ref(),
                NO_NAME,
            );
            Value::from_any(py, owner, AnyValueEnum::new(value))
        }
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

        let mut args = args
            .iter()
            .zip(param_types)
            .map(|(arg, ty)| unsafe { arg.to_value(ty).map(|v| v.get_ref()) }.map_err(Into::into))
            .collect::<PyResult<Vec<_>>>()?;

        unsafe {
            let call = CallSiteValue::new(LLVMBuildCall(
                self.builder,
                callee.get().get_ref(),
                args.as_mut_ptr(),
                args.len().try_into().unwrap(),
                NO_NAME,
            ));

            let value = call.try_as_basic_value().left();
            value.map(|v| Value::from_any(py, owner, v)).transpose()
        }
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
        unsafe {
            try_build_if(
                self.builder,
                cond.get().get_ref(),
                || r#true.iter().try_for_each(|f| f.call0().map(|_| ())),
                || r#false.iter().try_for_each(|f| f.call0().map(|_| ())),
            )
        }
    }

    /// Inserts an unconditional branch instruction.
    ///
    /// :param BasicBlock dest: The destination block.
    /// :returns: The branch instruction.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(dest)")]
    fn br(&self, py: Python, dest: PyRef<BasicBlock>) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, dest.as_ref().owner()])?;
        unsafe {
            let inst = LLVMBuildBr(self.builder, dest.get().get_ref());
            Value::from_any(py, owner, InstructionValue::new(inst))
        }
    }

    /// Inserts a return instruction.
    ///
    /// :param Value value: The value to return. If `None`, returns void.
    /// :returns: The return instruction.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(value)")]
    fn ret(&self, py: Python, value: Option<&Value>) -> PyResult<PyObject> {
        let (inst, owner) = match value {
            None => (
                unsafe { LLVMBuildRetVoid(self.builder) },
                self.owner.clone_ref(py),
            ),
            Some(value) => {
                let owner = Owner::merge(py, [&self.owner, value.owner()])?;
                let inst = unsafe { LLVMBuildRet(self.builder, value.get().get_ref()) };
                (inst, owner)
            }
        };
        unsafe { Value::from_any(py, owner, InstructionValue::new(inst)) }
    }
}

impl Builder {
    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }
}

impl Deref for Builder {
    type Target = LLVMBuilderRef;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}

struct Callable<'ctx> {
    ty: FunctionType<'ctx>,
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for Callable<'ctx> {
    type Error = PyErr;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::FunctionValue(f)) => Some(Self { ty: f.get_type() }),
            AnyValue::Any(AnyValueEnum::PointerValue(p)) => match p.get_type().get_element_type() {
                AnyTypeEnum::FunctionType(ty) => Some(Self { ty }),
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
            Argument::Literal(l) => l.to_value(basic_to_any(ty))?.try_into().map_err(Into::into),
        }
    }
}

fn basic_to_any(ty: BasicTypeEnum) -> AnyTypeEnum {
    match ty {
        BasicTypeEnum::ArrayType(a) => a.into(),
        BasicTypeEnum::FloatType(f) => f.into(),
        BasicTypeEnum::IntType(i) => i.into(),
        BasicTypeEnum::PointerType(p) => p.into(),
        BasicTypeEnum::StructType(s) => s.into(),
        BasicTypeEnum::VectorType(v) => v.into(),
    }
}

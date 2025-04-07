// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    core::Context,
    instructions::IntPredicate,
    types::Type,
    values::{BasicBlock, Literal, Owner, Value},
};
use const_str::raw_cstr;
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, prelude::*, LLVMBuilder, LLVMType, LLVMTypeKind};
use pyo3::{exceptions::PyValueError, prelude::*};
use qirlib::builder::try_build_if;
use std::{
    convert::{Into, TryInto},
    ops::Deref,
    ptr::NonNull,
};

/// An instruction builder.
///
/// :param Context context: The LLVM context.
#[pyclass(unsendable)]
pub(crate) struct Builder {
    builder: NonNull<LLVMBuilder>,
    owner: Owner,
}

#[pymethods]
impl Builder {
    #[new]
    pub(crate) fn new(py: Python, context: Py<Context>) -> Self {
        let builder = unsafe { LLVMCreateBuilderInContext(context.borrow(py).cast().as_ptr()) };
        Self {
            builder: NonNull::new(builder).unwrap(),
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
            LLVMPositionBuilderAtEnd(self.cast().as_ptr(), block.cast().as_ptr());
        }
        Ok(())
    }

    /// Tells this builder to insert subsequent instructions before the given instruction.
    ///
    /// :param Value instr: The instruction to insert before.
    /// :rtype: None
    #[pyo3(text_signature = "(instr)")]
    fn insert_before(&mut self, py: Python, instr: &Value) -> PyResult<()> {
        let owner = Owner::merge(py, [&self.owner, instr.owner()])?;
        if *owner.context(py).borrow(py) != *self.owner.context(py).borrow(py) {
            Err(PyValueError::new_err(
                "Instruction is not from the same context as builder.",
            ))?;
        }

        unsafe {
            LLVMPositionBuilderBefore(self.cast().as_ptr(), instr.cast().as_ptr());
        }
        Ok(())
    }

    /// Tells this builder to insert subsequent instructions after the given instruction.
    ///
    /// :param Value instr: The instruction to insert after.
    /// :rtype: None
    #[pyo3(text_signature = "(instr)")]
    fn insert_after(&mut self, py: Python, instr: &Value) -> PyResult<()> {
        let owner = Owner::merge(py, [&self.owner, instr.owner()])?;
        if *owner.context(py).borrow(py) != *self.owner.context(py).borrow(py) {
            Err(PyValueError::new_err(
                "Instruction is not from the same context as builder.",
            ))?;
        }

        unsafe {
            let next_instr = LLVMGetNextInstruction(instr.cast().as_ptr());
            if next_instr.is_null() {
                let block = LLVMGetInstructionParent(instr.cast().as_ptr());
                LLVMPositionBuilderAtEnd(self.cast().as_ptr(), block);
            } else {
                LLVMPositionBuilderBefore(self.cast().as_ptr(), next_instr);
            }
        }
        Ok(())
    }

    /// Inserts the given instruction.
    fn instr(&mut self, py: Python, instr: &Value) -> PyResult<()> {
        let owner = Owner::merge(py, [&self.owner, instr.owner()])?;
        if *owner.context(py).borrow(py) != *self.owner.context(py).borrow(py) {
            Err(PyValueError::new_err(
                "Instruction is not from the same context as builder.",
            ))?;
        }

        unsafe {
            LLVMInsertIntoBuilder(self.cast().as_ptr(), instr.cast().as_ptr());
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
    fn and_<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildAnd(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a bitwise logical or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn or_<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildOr(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a bitwise logical exclusive or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn xor<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildXor(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts an addition instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The sum.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn add<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildAdd(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a subtraction instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The difference.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn sub<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildSub(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a multiplication instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The product.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn mul<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildMul(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a shift left instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn shl<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildShl(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a logical (zero fill) shift right instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn lshr<'py>(&self, py: Python<'py>, lhs: &Value, rhs: &Value) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildLShr(
                self.cast().as_ptr(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
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
    fn icmp<'py>(
        &self,
        py: Python<'py>,
        pred: IntPredicate,
        lhs: &Value,
        rhs: &Value,
    ) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildICmp(
                self.cast().as_ptr(),
                pred.into(),
                lhs.cast().as_ptr(),
                rhs.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a call instruction.
    ///
    /// :param Value value: The value to call.
    /// :param typing.Sequence[typing.Union[Value, bool, int, float]] args:
    ///     The arguments to the function.
    /// :returns: The return value, or None if the function has a void return type.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, callee, args)")]
    fn call<'py>(
        &self,
        py: Python<'py>,
        callee: &Value,
        args: Vec<Argument>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let arg_owners = args.iter().filter_map(Argument::owner);
        let owner = Owner::merge(py, arg_owners.chain([&self.owner, callee.owner()]))?;

        unsafe {
            let fn_type = callable_fn_type(callee.cast().as_ptr())
                .ok_or_else(|| PyValueError::new_err("Callee is not callable."))?
                .cast()
                .as_ptr();
            let count = LLVMCountParamTypes(fn_type).try_into().unwrap();
            let mut param_types = Vec::with_capacity(count);
            LLVMGetParamTypes(fn_type, param_types.as_mut_ptr());
            param_types.set_len(count);

            if count != args.len() {
                Err(PyValueError::new_err(format!(
                    "Expected {} arguments, got {}.",
                    param_types.len(),
                    args.len()
                )))?;
            }

            let mut args = args
                .iter()
                .zip(param_types)
                .map(|(arg, ty)| arg.to_value(ty))
                .collect::<PyResult<Vec<_>>>()?;

            #[allow(deprecated)]
            let value = LLVMBuildCall(
                self.cast().as_ptr(),
                callee.cast().as_ptr(),
                args.as_mut_ptr(),
                args.len().try_into().unwrap(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
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
    #[pyo3(signature = (cond, r#true = None, r#false = None))]
    fn if_<'py>(
        &self,
        py: Python<'py>,
        cond: &Value,
        r#true: Option<Bound<'py, PyAny>>,
        r#false: Option<Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        Owner::merge(py, [&self.owner, cond.owner()])?;
        unsafe {
            try_build_if(
                self.cast().as_ptr(),
                cond.cast().as_ptr(),
                || r#true.into_iter().try_for_each(|f| f.call0().map(|_| ())),
                || r#false.into_iter().try_for_each(|f| f.call0().map(|_| ())),
            )
        }
    }

    /// Inserts an unconditional branch instruction.
    ///
    /// :param BasicBlock dest: The destination block.
    /// :returns: The branch instruction.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(dest)")]
    fn br<'py>(&self, py: Python<'py>, dest: PyRef<BasicBlock>) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, dest.as_ref().owner()])?;
        unsafe {
            let value = LLVMBuildBr(self.builder.cast().as_ptr(), dest.cast().as_ptr());
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts an conditional branch instruction.
    ///
    /// :param BasicBlock if_: The condition
    /// :param BasicBlock then: The destination block if condition is 1
    /// :param BasicBlock else_: The destination block if condition is 0
    /// :returns: The branch instruction.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(if_, then, else_)")]
    fn condbr<'py>(
        &self,
        py: Python<'py>,
        if_: &Value,
        then: PyRef<BasicBlock>,
        else_: PyRef<BasicBlock>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(
            py,
            [
                &self.owner,
                if_.owner(),
                then.as_ref().owner(),
                else_.as_ref().owner(),
            ],
        )?;
        unsafe {
            let value = LLVMBuildCondBr(
                self.builder.cast().as_ptr(),
                if_.cast().as_ptr(),
                then.cast().as_ptr(),
                else_.cast().as_ptr(),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a phi node.
    ///
    /// :returns: The phi node.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(type)")]
    fn phi<'py>(&self, py: Python<'py>, r#type: &Type) -> PyResult<Bound<'py, PyAny>> {
        unsafe {
            let owner = self.owner.clone_ref(py);
            let value = LLVMBuildPhi(
                self.builder.cast().as_ptr(),
                r#type.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a return instruction.
    ///
    /// :param Value value: The value to return. If `None`, returns void.
    /// :returns: The return instruction.
    /// :rtype: Instruction
    #[pyo3(signature = (value = None))]
    fn ret<'py>(&self, py: Python<'py>, value: Option<&Value>) -> PyResult<Bound<'py, PyAny>> {
        let (value, owner) = match value {
            None => (
                unsafe { LLVMBuildRetVoid(self.cast().as_ptr()) },
                self.owner.clone_ref(py),
            ),
            Some(value) => {
                let owner = Owner::merge(py, [&self.owner, value.owner()])?;
                let inst = unsafe { LLVMBuildRet(self.cast().as_ptr(), value.cast().as_ptr()) };
                (inst, owner)
            }
        };
        unsafe { Value::from_raw(py, owner, value) }
    }

    /// The ‘zext’ instruction zero extends its operand to the given type.
    ///
    /// :param Value val: Value to be converted.
    /// :param Type ty: Target type.
    /// :returns: The zext instruction.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, val, ty)")]
    fn zext<'py>(&self, py: Python<'py>, val: &Value, ty: &Type) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, val.owner()])?;
        unsafe {
            let value = LLVMBuildZExt(
                self.cast().as_ptr(),
                val.cast().as_ptr(),
                ty.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }

    /// The ‘trunc’ instruction truncates its operand to the given type.
    ///
    /// :param Value val: Value to be converted.
    /// :param Type ty: Target type.
    /// :returns: The trunc instruction.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, val, ty)")]
    fn trunc<'py>(&self, py: Python<'py>, val: &Value, ty: &Type) -> PyResult<Bound<'py, PyAny>> {
        let owner = Owner::merge(py, [&self.owner, val.owner()])?;
        unsafe {
            let value = LLVMBuildTrunc(
                self.cast().as_ptr(),
                val.cast().as_ptr(),
                ty.cast().as_ptr(),
                raw_cstr!(""),
            );
            Value::from_raw(py, owner, value)
        }
    }
}

impl Builder {
    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }
}

impl Deref for Builder {
    type Target = NonNull<LLVMBuilder>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder.cast().as_ptr());
        }
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

    unsafe fn to_value(&self, ty: LLVMTypeRef) -> PyResult<LLVMValueRef> {
        match self {
            Argument::Value(v) => Ok(v.cast().as_ptr()),
            Argument::Literal(l) => l.to_value(ty),
        }
    }
}

unsafe fn callable_fn_type(value: LLVMValueRef) -> Option<NonNull<LLVMType>> {
    let ty = LLVMTypeOf(value);
    match LLVMGetTypeKind(ty) {
        LLVMTypeKind::LLVMFunctionTypeKind => Some(NonNull::new(ty).unwrap()),
        LLVMTypeKind::LLVMPointerTypeKind => {
            let pointee = LLVMGetElementType(ty);
            if LLVMGetTypeKind(pointee) == LLVMTypeKind::LLVMFunctionTypeKind {
                Some(NonNull::new(pointee).unwrap())
            } else {
                None
            }
        }
        _ => None,
    }
}

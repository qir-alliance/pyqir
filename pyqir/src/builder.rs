// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    core::Context,
    instructions::IntPredicate,
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
        let builder = unsafe { LLVMCreateBuilderInContext(context.borrow(py).as_ptr()) };
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
            LLVMPositionBuilderAtEnd(self.as_ptr(), block.as_ptr());
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
            let value = LLVMBuildAnd(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn or_(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildOr(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn xor(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildXor(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn add(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildAdd(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn sub(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildSub(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn mul(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildMul(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn shl(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildShl(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn lshr(&self, py: Python, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildLShr(self.as_ptr(), lhs.as_ptr(), rhs.as_ptr(), raw_cstr!(""));
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
    fn icmp(&self, py: Python, pred: IntPredicate, lhs: &Value, rhs: &Value) -> PyResult<PyObject> {
        let owner = Owner::merge(py, [&self.owner, lhs.owner(), rhs.owner()])?;
        unsafe {
            let value = LLVMBuildICmp(
                self.as_ptr(),
                pred.into(),
                lhs.as_ptr(),
                rhs.as_ptr(),
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
    fn call(&self, py: Python, callee: &Value, args: Vec<Argument>) -> PyResult<PyObject> {
        let arg_owners = args.iter().filter_map(Argument::owner);
        let owner = Owner::merge(py, arg_owners.chain([&self.owner, callee.owner()]))?;

        unsafe {
            let fn_type = callable_fn_type(callee.as_ptr())
                .ok_or_else(|| PyValueError::new_err("Callee is not callable."))?
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
                .map(|(arg, ty)| arg.to_value(ty).map_err(Into::into))
                .collect::<PyResult<Vec<_>>>()?;
            
            #[allow(deprecated)]
            let value = LLVMBuildCall(
                self.as_ptr(),
                callee.as_ptr(),
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
                self.as_ptr(),
                cond.as_ptr(),
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
            let value = LLVMBuildBr(self.builder.as_ptr(), dest.as_ptr());
            Value::from_raw(py, owner, value)
        }
    }

    /// Inserts a return instruction.
    ///
    /// :param Value value: The value to return. If `None`, returns void.
    /// :returns: The return instruction.
    /// :rtype: Instruction
    #[pyo3(text_signature = "(value)")]
    fn ret(&self, py: Python, value: Option<&Value>) -> PyResult<PyObject> {
        let (value, owner) = match value {
            None => (
                unsafe { LLVMBuildRetVoid(self.as_ptr()) },
                self.owner.clone_ref(py),
            ),
            Some(value) => {
                let owner = Owner::merge(py, [&self.owner, value.owner()])?;
                let inst = unsafe { LLVMBuildRet(self.as_ptr(), value.as_ptr()) };
                (inst, owner)
            }
        };
        unsafe { Value::from_raw(py, owner, value) }
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
            LLVMDisposeBuilder(self.builder.as_ptr());
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
            Argument::Value(v) => Ok(v.as_ptr()),
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

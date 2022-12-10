// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::values::{BasicBlock, Owner, Value};
#[allow(clippy::wildcard_imports)]
use llvm_sys::{core::*, LLVMIntPredicate, LLVMOpcode, LLVMRealPredicate, LLVMValue};
use pyo3::{conversion::ToPyObject, prelude::*};
use std::{convert::Into, ptr::NonNull};

/// An instruction.
#[pyclass(extends = Value, subclass)]
pub(crate) struct Instruction;

#[pymethods]
impl Instruction {
    /// The instruction opcode.
    ///
    /// :type: Opcode
    #[getter]
    fn opcode(slf: PyRef<Self>) -> Opcode {
        unsafe { LLVMGetInstructionOpcode(slf.into_super().as_ptr()) }.into()
    }

    /// The operands to the instruction.
    ///
    /// :type: typing.List[Value]
    #[getter]
    fn operands(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let slf = slf.into_super();
        let owner = slf.owner();
        unsafe {
            (0..u32::try_from(LLVMGetNumOperands(slf.as_ptr())).unwrap())
                .map(|i| {
                    let operand = LLVMGetOperand(slf.as_ptr(), i);
                    Value::from_ptr(py, owner.clone_ref(py), NonNull::new(operand).unwrap())
                })
                .collect()
        }
    }

    /// The basic blocks that are successors to this instruction. If this is not a terminator, the
    /// list is empty.
    ///
    /// :type: typing.List[BasicBlock]
    #[getter]
    fn successors(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        if unsafe { LLVMIsATerminatorInst(slf.as_ref().as_ptr()) }.is_null() {
            Ok(Vec::new())
        } else {
            // then_some is stabilized in Rust 1.62.
            #[allow(clippy::unnecessary_lazy_evaluations)]
            Self::operands(slf, py)?
                .into_iter()
                .filter_map(|o| {
                    o.as_ref(py)
                        .is_instance_of::<BasicBlock>()
                        .map(|b| b.then(|| o))
                        .transpose()
                })
                .collect()
        }
    }

    /// Removes this instruction from its parent basic block, then deletes it from memory.
    ///
    /// .. warning:: Using this instruction after erasing it is undefined behavior.
    ///
    /// :rtype: None
    fn erase(slf: PyRef<Self>) {
        unsafe {
            LLVMInstructionEraseFromParent(slf.into_super().as_ptr());
        }
    }
}

impl Instruction {
    pub(crate) unsafe fn from_ptr(
        py: Python,
        owner: Owner,
        value: NonNull<LLVMValue>,
    ) -> PyResult<PyObject> {
        let base = Value::new(owner, value).add_subclass(Self);
        match LLVMGetInstructionOpcode(value.as_ptr()) {
            LLVMOpcode::LLVMSwitch => Ok(Py::new(py, base.add_subclass(Switch))?.to_object(py)),
            LLVMOpcode::LLVMICmp => Ok(Py::new(py, base.add_subclass(ICmp))?.to_object(py)),
            LLVMOpcode::LLVMFCmp => Ok(Py::new(py, base.add_subclass(FCmp))?.to_object(py)),
            LLVMOpcode::LLVMCall => Ok(Py::new(py, base.add_subclass(Call))?.to_object(py)),
            LLVMOpcode::LLVMPHI => Ok(Py::new(py, base.add_subclass(Phi))?.to_object(py)),
            _ => Ok(Py::new(py, base)?.to_object(py)),
        }
    }
}

/// An instruction opcode.
#[pyclass]
pub(crate) enum Opcode {
    #[pyo3(name = "ADD")]
    Add,
    #[pyo3(name = "ADDR_SPACE_CAST")]
    AddrSpaceCast,
    #[pyo3(name = "ALLOCA")]
    Alloca,
    #[pyo3(name = "AND")]
    And,
    #[pyo3(name = "ASHR")]
    AShr,
    #[pyo3(name = "ATOMIC_CMP_XCHG")]
    AtomicCmpXchg,
    #[pyo3(name = "ATOMIC_RMW")]
    AtomicRmw,
    #[pyo3(name = "BIT_CAST")]
    BitCast,
    #[pyo3(name = "BR")]
    Br,
    #[pyo3(name = "CALL")]
    Call,
    #[pyo3(name = "CALL_BR")]
    CallBr,
    #[pyo3(name = "CATCH_RET")]
    CatchRet,
    #[pyo3(name = "CATCH_PAD")]
    CatchPad,
    #[pyo3(name = "CATCH_SWITCH")]
    CatchSwitch,
    #[pyo3(name = "CLEANUP_PAD")]
    CleanupPad,
    #[pyo3(name = "CLEANUP_RET")]
    CleanupRet,
    #[pyo3(name = "EXTRACT_ELEMENT")]
    ExtractElement,
    #[pyo3(name = "EXTRACT_VALUE")]
    ExtractValue,
    #[pyo3(name = "FADD")]
    FAdd,
    #[pyo3(name = "FCMP")]
    FCmp,
    #[pyo3(name = "FDIV")]
    FDiv,
    #[pyo3(name = "FENCE")]
    Fence,
    #[pyo3(name = "FMUL")]
    FMul,
    #[pyo3(name = "FNEG")]
    FNeg,
    #[pyo3(name = "FP_EXT")]
    FPExt,
    #[pyo3(name = "FP_TO_SI")]
    FPToSI,
    #[pyo3(name = "FP_TO_UI")]
    FPToUI,
    #[pyo3(name = "FP_TRUNC")]
    FPTrunc,
    #[pyo3(name = "FREEZE")]
    Freeze,
    #[pyo3(name = "FREM")]
    FRem,
    #[pyo3(name = "FSUB")]
    FSub,
    #[pyo3(name = "GET_ELEMENT_PTR")]
    GetElementPtr,
    #[pyo3(name = "ICMP")]
    ICmp,
    #[pyo3(name = "INDIRECT_BR")]
    IndirectBr,
    #[pyo3(name = "INSERT_ELEMENT")]
    InsertElement,
    #[pyo3(name = "INSERT_VALUE")]
    InsertValue,
    #[pyo3(name = "INT_TO_PTR")]
    IntToPtr,
    #[pyo3(name = "INVOKE")]
    Invoke,
    #[pyo3(name = "LANDING_PAD")]
    LandingPad,
    #[pyo3(name = "LOAD")]
    Load,
    #[pyo3(name = "LSHR")]
    LShr,
    #[pyo3(name = "MUL")]
    Mul,
    #[pyo3(name = "OR")]
    Or,
    #[pyo3(name = "PHI")]
    Phi,
    #[pyo3(name = "PTR_TO_INT")]
    PtrToInt,
    #[pyo3(name = "RESUME")]
    Resume,
    #[pyo3(name = "RET")]
    Ret,
    #[pyo3(name = "SDIV")]
    SDiv,
    #[pyo3(name = "SELECT")]
    Select,
    #[pyo3(name = "SEXT")]
    SExt,
    #[pyo3(name = "SHL")]
    Shl,
    #[pyo3(name = "SHUFFLE_VECTOR")]
    ShuffleVector,
    #[pyo3(name = "SI_TO_FP")]
    SIToFP,
    #[pyo3(name = "SREM")]
    SRem,
    #[pyo3(name = "STORE")]
    Store,
    #[pyo3(name = "SUB")]
    Sub,
    #[pyo3(name = "SWITCH")]
    Switch,
    #[pyo3(name = "TRUNC")]
    Trunc,
    #[pyo3(name = "UDIV")]
    UDiv,
    #[pyo3(name = "UI_TO_FP")]
    UIToFP,
    #[pyo3(name = "UNREACHABLE")]
    Unreachable,
    #[pyo3(name = "UREM")]
    URem,
    #[pyo3(name = "USER_OP_1")]
    UserOp1,
    #[pyo3(name = "USER_OP_2")]
    UserOp2,
    #[pyo3(name = "VA_ARG")]
    VaArg,
    #[pyo3(name = "XOR")]
    Xor,
    #[pyo3(name = "ZEXT")]
    ZExt,
}

impl From<LLVMOpcode> for Opcode {
    fn from(opcode: LLVMOpcode) -> Self {
        match opcode {
            LLVMOpcode::LLVMAdd => Self::Add,
            LLVMOpcode::LLVMAddrSpaceCast => Self::AddrSpaceCast,
            LLVMOpcode::LLVMAlloca => Self::Alloca,
            LLVMOpcode::LLVMAnd => Self::And,
            LLVMOpcode::LLVMAShr => Self::AShr,
            LLVMOpcode::LLVMAtomicCmpXchg => Self::AtomicCmpXchg,
            LLVMOpcode::LLVMAtomicRMW => Self::AtomicRmw,
            LLVMOpcode::LLVMBitCast => Self::BitCast,
            LLVMOpcode::LLVMBr => Self::Br,
            LLVMOpcode::LLVMCall => Self::Call,
            LLVMOpcode::LLVMCallBr => Self::CallBr,
            LLVMOpcode::LLVMCatchPad => Self::CatchPad,
            LLVMOpcode::LLVMCatchRet => Self::CatchRet,
            LLVMOpcode::LLVMCatchSwitch => Self::CatchSwitch,
            LLVMOpcode::LLVMCleanupPad => Self::CleanupPad,
            LLVMOpcode::LLVMCleanupRet => Self::CleanupRet,
            LLVMOpcode::LLVMExtractElement => Self::ExtractElement,
            LLVMOpcode::LLVMExtractValue => Self::ExtractValue,
            LLVMOpcode::LLVMFNeg => Self::FNeg,
            LLVMOpcode::LLVMFAdd => Self::FAdd,
            LLVMOpcode::LLVMFCmp => Self::FCmp,
            LLVMOpcode::LLVMFDiv => Self::FDiv,
            LLVMOpcode::LLVMFence => Self::Fence,
            LLVMOpcode::LLVMFMul => Self::FMul,
            LLVMOpcode::LLVMFPExt => Self::FPExt,
            LLVMOpcode::LLVMFPToSI => Self::FPToSI,
            LLVMOpcode::LLVMFPToUI => Self::FPToUI,
            LLVMOpcode::LLVMFPTrunc => Self::FPTrunc,
            LLVMOpcode::LLVMFreeze => Self::Freeze,
            LLVMOpcode::LLVMFRem => Self::FRem,
            LLVMOpcode::LLVMFSub => Self::FSub,
            LLVMOpcode::LLVMGetElementPtr => Self::GetElementPtr,
            LLVMOpcode::LLVMICmp => Self::ICmp,
            LLVMOpcode::LLVMIndirectBr => Self::IndirectBr,
            LLVMOpcode::LLVMInsertElement => Self::InsertElement,
            LLVMOpcode::LLVMInsertValue => Self::InsertValue,
            LLVMOpcode::LLVMIntToPtr => Self::IntToPtr,
            LLVMOpcode::LLVMInvoke => Self::Invoke,
            LLVMOpcode::LLVMLandingPad => Self::LandingPad,
            LLVMOpcode::LLVMLoad => Self::Load,
            LLVMOpcode::LLVMLShr => Self::LShr,
            LLVMOpcode::LLVMMul => Self::Mul,
            LLVMOpcode::LLVMOr => Self::Or,
            LLVMOpcode::LLVMPHI => Self::Phi,
            LLVMOpcode::LLVMPtrToInt => Self::PtrToInt,
            LLVMOpcode::LLVMResume => Self::Resume,
            LLVMOpcode::LLVMRet => Self::Ret,
            LLVMOpcode::LLVMSDiv => Self::SDiv,
            LLVMOpcode::LLVMSelect => Self::Select,
            LLVMOpcode::LLVMSExt => Self::SExt,
            LLVMOpcode::LLVMShl => Self::Shl,
            LLVMOpcode::LLVMShuffleVector => Self::ShuffleVector,
            LLVMOpcode::LLVMSIToFP => Self::SIToFP,
            LLVMOpcode::LLVMSRem => Self::SRem,
            LLVMOpcode::LLVMStore => Self::Store,
            LLVMOpcode::LLVMSub => Self::Sub,
            LLVMOpcode::LLVMSwitch => Self::Switch,
            LLVMOpcode::LLVMTrunc => Self::Trunc,
            LLVMOpcode::LLVMUDiv => Self::UDiv,
            LLVMOpcode::LLVMUIToFP => Self::UIToFP,
            LLVMOpcode::LLVMUnreachable => Self::Unreachable,
            LLVMOpcode::LLVMURem => Self::URem,
            LLVMOpcode::LLVMUserOp1 => Self::UserOp1,
            LLVMOpcode::LLVMUserOp2 => Self::UserOp2,
            LLVMOpcode::LLVMVAArg => Self::VaArg,
            LLVMOpcode::LLVMXor => Self::Xor,
            LLVMOpcode::LLVMZExt => Self::ZExt,
        }
    }
}

/// A switch instruction.
#[pyclass(extends = Instruction)]
pub(crate) struct Switch;

#[pymethods]
impl Switch {
    /// The condition of the switch.
    ///
    /// :type: Value
    #[getter]
    fn cond(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let slf = slf.into_super().into_super();
        unsafe {
            let value = LLVMGetOperand(slf.as_ptr(), 0);
            Value::from_ptr(py, slf.owner().clone_ref(py), NonNull::new(value).unwrap())
        }
    }

    /// The default successor block if none of the cases match.
    ///
    /// :type: BasicBlock
    #[getter]
    fn default(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let slf = slf.into_super().into_super();
        unsafe {
            let value = LLVMGetOperand(slf.as_ptr(), 1);
            Value::from_ptr(py, slf.owner().clone_ref(py), NonNull::new(value).unwrap())
        }
    }

    /// The switch cases.
    ///
    /// :type: typing.List[typing.Tuple[Value, BasicBlock]]
    #[getter]
    fn cases(slf: PyRef<Self>, py: Python) -> PyResult<Vec<(PyObject, PyObject)>> {
        let slf = slf.into_super().into_super();
        let owner = slf.owner();
        unsafe {
            (2..u32::try_from(LLVMGetNumOperands(slf.as_ptr())).unwrap())
                .step_by(2)
                .map(|i| {
                    let cond = LLVMGetOperand(slf.as_ptr(), i);
                    let succ = LLVMGetOperand(slf.as_ptr(), i + 1);
                    Ok((
                        Value::from_ptr(py, owner.clone_ref(py), NonNull::new(cond).unwrap())?,
                        Value::from_ptr(py, owner.clone_ref(py), NonNull::new(succ).unwrap())?,
                    ))
                })
                .collect()
        }
    }
}

/// An integer comparison instruction.
#[pyclass(extends = Instruction)]
pub(crate) struct ICmp;

#[pymethods]
impl ICmp {
    /// The comparison predicate.
    ///
    /// :type: IntPredicate
    #[getter]
    fn predicate(slf: PyRef<Self>) -> IntPredicate {
        unsafe { LLVMGetICmpPredicate(slf.into_super().into_super().as_ptr()) }.into()
    }
}

/// An integer comparison predicate.
#[pyclass]
#[derive(Clone, Copy)]
pub(crate) enum IntPredicate {
    #[pyo3(name = "EQ")]
    Eq,
    #[pyo3(name = "NE")]
    Ne,
    #[pyo3(name = "UGT")]
    Ugt,
    #[pyo3(name = "UGE")]
    Uge,
    #[pyo3(name = "ULT")]
    Ult,
    #[pyo3(name = "ULE")]
    Ule,
    #[pyo3(name = "SGT")]
    Sgt,
    #[pyo3(name = "SGE")]
    Sge,
    #[pyo3(name = "SLT")]
    Slt,
    #[pyo3(name = "SLE")]
    Sle,
}

impl From<LLVMIntPredicate> for IntPredicate {
    fn from(pred: LLVMIntPredicate) -> Self {
        match pred {
            LLVMIntPredicate::LLVMIntEQ => Self::Eq,
            LLVMIntPredicate::LLVMIntNE => Self::Ne,
            LLVMIntPredicate::LLVMIntUGT => Self::Ugt,
            LLVMIntPredicate::LLVMIntUGE => Self::Uge,
            LLVMIntPredicate::LLVMIntULT => Self::Ult,
            LLVMIntPredicate::LLVMIntULE => Self::Ule,
            LLVMIntPredicate::LLVMIntSGT => Self::Sgt,
            LLVMIntPredicate::LLVMIntSGE => Self::Sge,
            LLVMIntPredicate::LLVMIntSLT => Self::Slt,
            LLVMIntPredicate::LLVMIntSLE => Self::Sle,
        }
    }
}

impl From<IntPredicate> for LLVMIntPredicate {
    fn from(pred: IntPredicate) -> Self {
        match pred {
            IntPredicate::Eq => Self::LLVMIntEQ,
            IntPredicate::Ne => Self::LLVMIntNE,
            IntPredicate::Ugt => Self::LLVMIntUGT,
            IntPredicate::Uge => Self::LLVMIntUGE,
            IntPredicate::Ult => Self::LLVMIntULT,
            IntPredicate::Ule => Self::LLVMIntULE,
            IntPredicate::Sgt => Self::LLVMIntSGT,
            IntPredicate::Sge => Self::LLVMIntSGE,
            IntPredicate::Slt => Self::LLVMIntSLT,
            IntPredicate::Sle => Self::LLVMIntSLE,
        }
    }
}

/// A floating-point comparison instruction.
#[pyclass(extends = Instruction)]
pub(crate) struct FCmp;

#[pymethods]
impl FCmp {
    /// The comparison predicate.
    ///
    /// :type: FloatPredicate
    #[getter]
    fn predicate(slf: PyRef<Self>) -> FloatPredicate {
        unsafe { LLVMGetFCmpPredicate(slf.into_super().into_super().as_ptr()) }.into()
    }
}

/// A floating-point comparison predicate.
#[pyclass]
#[derive(Clone, Copy)]
pub(crate) enum FloatPredicate {
    #[pyo3(name = "FALSE")]
    False,
    #[pyo3(name = "OEQ")]
    Oeq,
    #[pyo3(name = "OGT")]
    Ogt,
    #[pyo3(name = "OGE")]
    Oge,
    #[pyo3(name = "OLT")]
    Olt,
    #[pyo3(name = "OLE")]
    Ole,
    #[pyo3(name = "ONE")]
    One,
    #[pyo3(name = "ORD")]
    Ord,
    #[pyo3(name = "UNO")]
    Uno,
    #[pyo3(name = "UEQ")]
    Ueq,
    #[pyo3(name = "UGT")]
    Ugt,
    #[pyo3(name = "UGE")]
    Uge,
    #[pyo3(name = "ULT")]
    Ult,
    #[pyo3(name = "ULE")]
    Ule,
    #[pyo3(name = "UNE")]
    Une,
    #[pyo3(name = "TRUE")]
    True,
}

impl From<LLVMRealPredicate> for FloatPredicate {
    fn from(pred: LLVMRealPredicate) -> Self {
        match pred {
            LLVMRealPredicate::LLVMRealOEQ => Self::Oeq,
            LLVMRealPredicate::LLVMRealOGE => Self::Oge,
            LLVMRealPredicate::LLVMRealOGT => Self::Ogt,
            LLVMRealPredicate::LLVMRealOLE => Self::Ole,
            LLVMRealPredicate::LLVMRealOLT => Self::Olt,
            LLVMRealPredicate::LLVMRealONE => Self::One,
            LLVMRealPredicate::LLVMRealORD => Self::Ord,
            LLVMRealPredicate::LLVMRealPredicateFalse => Self::False,
            LLVMRealPredicate::LLVMRealPredicateTrue => Self::True,
            LLVMRealPredicate::LLVMRealUEQ => Self::Ueq,
            LLVMRealPredicate::LLVMRealUGE => Self::Uge,
            LLVMRealPredicate::LLVMRealUGT => Self::Ugt,
            LLVMRealPredicate::LLVMRealULE => Self::Ule,
            LLVMRealPredicate::LLVMRealULT => Self::Ult,
            LLVMRealPredicate::LLVMRealUNE => Self::Une,
            LLVMRealPredicate::LLVMRealUNO => Self::Uno,
        }
    }
}

/// A call instruction.
#[pyclass(extends = Instruction)]
pub(crate) struct Call;

#[pymethods]
impl Call {
    /// The value being called.
    ///
    /// :type: Value
    #[getter]
    fn callee(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let slf = slf.into_super().into_super();
        unsafe {
            let value = LLVMGetCalledValue(slf.as_ptr());
            Value::from_ptr(py, slf.owner().clone_ref(py), NonNull::new(value).unwrap())
        }
    }

    /// The arguments to the call.
    ///
    /// :type: typing.List[Value]
    #[getter]
    fn args(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let mut args = Instruction::operands(slf.into_super(), py)?;
        args.pop().unwrap();
        Ok(args)
    }
}

/// A phi node instruction.
#[pyclass(extends = Instruction)]
pub(crate) struct Phi;

#[pymethods]
impl Phi {
    /// The incoming values and their preceding basic blocks.
    ///
    /// :type: typing.List[typing.Tuple[Value, BasicBlock]]
    #[getter]
    fn incoming(slf: PyRef<Self>, py: Python) -> PyResult<Vec<(PyObject, PyObject)>> {
        let slf = slf.into_super().into_super();
        let owner = slf.owner();
        unsafe {
            (0..LLVMCountIncoming(slf.as_ptr()))
                .map(|i| {
                    let value = LLVMGetIncomingValue(slf.as_ptr(), i);
                    let block = LLVMBasicBlockAsValue(LLVMGetIncomingBlock(slf.as_ptr(), i));
                    Ok((
                        Value::from_ptr(py, owner.clone_ref(py), NonNull::new(value).unwrap())?,
                        Value::from_ptr(py, owner.clone_ref(py), NonNull::new(block).unwrap())?,
                    ))
                })
                .collect()
        }
    }
}

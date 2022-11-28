// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::values::{BasicBlock, Owner, Value};
use either::Either::{Left, Right};
use inkwell::{
    values::InstructionOpcode,
    values::{InstructionValue, PhiValue},
    LLVMReference,
};
use llvm_sys::core::LLVMIsATerminatorInst;
use pyo3::{conversion::ToPyObject, prelude::*};
use std::{
    convert::{Into, TryInto},
    mem::transmute,
};

/// An instruction.
#[pyclass(extends = Value, subclass, unsendable)]
pub(crate) struct Instruction(InstructionValue<'static>);

#[pymethods]
impl Instruction {
    /// The instruction opcode.
    ///
    /// :type: Opcode
    #[getter]
    fn opcode(&self) -> Opcode {
        self.0.get_opcode().into()
    }

    /// The operands to the instruction.
    ///
    /// :type: List[Value]
    #[getter]
    fn operands(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let owner = slf.as_ref().owner();
        (0..slf.0.get_num_operands())
            .map(|i| match slf.0.get_operand(i).unwrap() {
                Left(value) => unsafe { Value::from_any(py, owner.clone_ref(py), value) },
                Right(block) => unsafe { Value::from_any(py, owner.clone_ref(py), block) },
            })
            .collect()
    }

    /// The basic blocks that are successors to this instruction. If this is not a terminator, the
    /// list is empty.
    ///
    /// :type: List[BasicBlock]
    #[getter]
    fn successors(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        if unsafe { LLVMIsATerminatorInst(slf.0.get_ref()) }.is_null() {
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
}

impl Instruction {
    pub(crate) unsafe fn from_inst(
        py: Python,
        owner: Owner,
        inst: InstructionValue,
    ) -> PyResult<PyObject> {
        let inst = transmute::<InstructionValue<'_>, InstructionValue<'static>>(inst);
        let base = Value::init(owner, inst.into()).add_subclass(Self(inst));
        match inst.get_opcode() {
            InstructionOpcode::Switch => Ok(Py::new(py, base.add_subclass(Switch))?.to_object(py)),
            InstructionOpcode::ICmp => Ok(Py::new(py, base.add_subclass(ICmp))?.to_object(py)),
            InstructionOpcode::FCmp => Ok(Py::new(py, base.add_subclass(FCmp))?.to_object(py)),
            InstructionOpcode::Call => Ok(Py::new(py, base.add_subclass(Call))?.to_object(py)),
            InstructionOpcode::Phi => {
                let phi = Phi(inst.try_into().unwrap());
                Ok(Py::new(py, base.add_subclass(phi))?.to_object(py))
            }
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

impl From<InstructionOpcode> for Opcode {
    fn from(opcode: InstructionOpcode) -> Self {
        match opcode {
            InstructionOpcode::Add => Self::Add,
            InstructionOpcode::AddrSpaceCast => Self::AddrSpaceCast,
            InstructionOpcode::Alloca => Self::Alloca,
            InstructionOpcode::And => Self::And,
            InstructionOpcode::AShr => Self::AShr,
            InstructionOpcode::AtomicCmpXchg => Self::AtomicCmpXchg,
            InstructionOpcode::AtomicRMW => Self::AtomicRmw,
            InstructionOpcode::BitCast => Self::BitCast,
            InstructionOpcode::Br => Self::Br,
            InstructionOpcode::Call => Self::Call,
            InstructionOpcode::CallBr => Self::CallBr,
            InstructionOpcode::CatchPad => Self::CatchPad,
            InstructionOpcode::CatchRet => Self::CatchRet,
            InstructionOpcode::CatchSwitch => Self::CatchSwitch,
            InstructionOpcode::CleanupPad => Self::CleanupPad,
            InstructionOpcode::CleanupRet => Self::CleanupRet,
            InstructionOpcode::ExtractElement => Self::ExtractElement,
            InstructionOpcode::ExtractValue => Self::ExtractValue,
            InstructionOpcode::FNeg => Self::FNeg,
            InstructionOpcode::FAdd => Self::FAdd,
            InstructionOpcode::FCmp => Self::FCmp,
            InstructionOpcode::FDiv => Self::FDiv,
            InstructionOpcode::Fence => Self::Fence,
            InstructionOpcode::FMul => Self::FMul,
            InstructionOpcode::FPExt => Self::FPExt,
            InstructionOpcode::FPToSI => Self::FPToSI,
            InstructionOpcode::FPToUI => Self::FPToUI,
            InstructionOpcode::FPTrunc => Self::FPTrunc,
            InstructionOpcode::Freeze => Self::Freeze,
            InstructionOpcode::FRem => Self::FRem,
            InstructionOpcode::FSub => Self::FSub,
            InstructionOpcode::GetElementPtr => Self::GetElementPtr,
            InstructionOpcode::ICmp => Self::ICmp,
            InstructionOpcode::IndirectBr => Self::IndirectBr,
            InstructionOpcode::InsertElement => Self::InsertElement,
            InstructionOpcode::InsertValue => Self::InsertValue,
            InstructionOpcode::IntToPtr => Self::IntToPtr,
            InstructionOpcode::Invoke => Self::Invoke,
            InstructionOpcode::LandingPad => Self::LandingPad,
            InstructionOpcode::Load => Self::Load,
            InstructionOpcode::LShr => Self::LShr,
            InstructionOpcode::Mul => Self::Mul,
            InstructionOpcode::Or => Self::Or,
            InstructionOpcode::Phi => Self::Phi,
            InstructionOpcode::PtrToInt => Self::PtrToInt,
            InstructionOpcode::Resume => Self::Resume,
            InstructionOpcode::Return => Self::Ret,
            InstructionOpcode::SDiv => Self::SDiv,
            InstructionOpcode::Select => Self::Select,
            InstructionOpcode::SExt => Self::SExt,
            InstructionOpcode::Shl => Self::Shl,
            InstructionOpcode::ShuffleVector => Self::ShuffleVector,
            InstructionOpcode::SIToFP => Self::SIToFP,
            InstructionOpcode::SRem => Self::SRem,
            InstructionOpcode::Store => Self::Store,
            InstructionOpcode::Sub => Self::Sub,
            InstructionOpcode::Switch => Self::Switch,
            InstructionOpcode::Trunc => Self::Trunc,
            InstructionOpcode::UDiv => Self::UDiv,
            InstructionOpcode::UIToFP => Self::UIToFP,
            InstructionOpcode::Unreachable => Self::Unreachable,
            InstructionOpcode::URem => Self::URem,
            InstructionOpcode::UserOp1 => Self::UserOp1,
            InstructionOpcode::UserOp2 => Self::UserOp2,
            InstructionOpcode::VAArg => Self::VaArg,
            InstructionOpcode::Xor => Self::Xor,
            InstructionOpcode::ZExt => Self::ZExt,
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
        let inst = slf.into_super();
        let cond = inst.0.get_operand(0).unwrap().left().unwrap();
        let owner = inst.as_ref().owner().clone_ref(py);
        unsafe { Value::from_any(py, owner, cond) }
    }

    /// The default successor block if none of the cases match.
    ///
    /// :type: BasicBlock
    #[getter]
    fn default(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let inst = slf.into_super();
        let block = inst.0.get_operand(1).unwrap().right().unwrap();
        let owner = inst.as_ref().owner().clone_ref(py);
        unsafe { Value::from_any(py, owner, block) }
    }

    /// The switch cases.
    ///
    /// :type: List[Tuple[Value, BasicBlock]]
    #[getter]
    fn cases(slf: PyRef<Self>, py: Python) -> PyResult<Vec<(PyObject, PyObject)>> {
        let inst_ref = slf.into_super();
        let inst = inst_ref.0;
        let value = inst_ref.into_super();
        let owner = value.owner();

        (2..inst.get_num_operands())
            .step_by(2)
            .map(|i| {
                let cond = inst.get_operand(i).unwrap().left().unwrap();
                let cond = unsafe { Value::from_any(py, owner.clone_ref(py), cond)? };
                let succ = inst.get_operand(i + 1).unwrap().right().unwrap();
                let succ = unsafe { Value::from_any(py, owner.clone_ref(py), succ)? };
                Ok((cond, succ))
            })
            .collect()
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
        slf.into_super().0.get_icmp_predicate().unwrap().into()
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

impl From<inkwell::IntPredicate> for IntPredicate {
    fn from(pred: inkwell::IntPredicate) -> Self {
        match pred {
            inkwell::IntPredicate::EQ => Self::Eq,
            inkwell::IntPredicate::NE => Self::Ne,
            inkwell::IntPredicate::UGT => Self::Ugt,
            inkwell::IntPredicate::UGE => Self::Uge,
            inkwell::IntPredicate::ULT => Self::Ult,
            inkwell::IntPredicate::ULE => Self::Ule,
            inkwell::IntPredicate::SGT => Self::Sgt,
            inkwell::IntPredicate::SGE => Self::Sge,
            inkwell::IntPredicate::SLT => Self::Slt,
            inkwell::IntPredicate::SLE => Self::Sle,
        }
    }
}

impl From<IntPredicate> for inkwell::IntPredicate {
    fn from(pred: IntPredicate) -> Self {
        match pred {
            IntPredicate::Eq => Self::EQ,
            IntPredicate::Ne => Self::NE,
            IntPredicate::Ugt => Self::UGT,
            IntPredicate::Uge => Self::UGE,
            IntPredicate::Ult => Self::ULT,
            IntPredicate::Ule => Self::ULE,
            IntPredicate::Sgt => Self::SGT,
            IntPredicate::Sge => Self::SGE,
            IntPredicate::Slt => Self::SLT,
            IntPredicate::Sle => Self::SLE,
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
        slf.into_super().0.get_fcmp_predicate().unwrap().into()
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

impl From<inkwell::FloatPredicate> for FloatPredicate {
    fn from(pred: inkwell::FloatPredicate) -> Self {
        match pred {
            inkwell::FloatPredicate::OEQ => Self::Oeq,
            inkwell::FloatPredicate::OGE => Self::Oge,
            inkwell::FloatPredicate::OGT => Self::Ogt,
            inkwell::FloatPredicate::OLE => Self::Ole,
            inkwell::FloatPredicate::OLT => Self::Olt,
            inkwell::FloatPredicate::ONE => Self::One,
            inkwell::FloatPredicate::ORD => Self::Ord,
            inkwell::FloatPredicate::PredicateFalse => Self::False,
            inkwell::FloatPredicate::PredicateTrue => Self::True,
            inkwell::FloatPredicate::UEQ => Self::Ueq,
            inkwell::FloatPredicate::UGE => Self::Uge,
            inkwell::FloatPredicate::UGT => Self::Ugt,
            inkwell::FloatPredicate::ULE => Self::Ule,
            inkwell::FloatPredicate::ULT => Self::Ult,
            inkwell::FloatPredicate::UNE => Self::Une,
            inkwell::FloatPredicate::UNO => Self::Uno,
        }
    }
}

/// A call instruction.
#[pyclass(extends = Instruction, unsendable)]
pub(crate) struct Call;

#[pymethods]
impl Call {
    /// The value being called.
    ///
    /// :type: Value
    #[getter]
    fn callee(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let inst = slf.into_super();
        let last = inst.0.get_operand(inst.0.get_num_operands() - 1);
        let callee = last.unwrap().left().unwrap();
        let owner = inst.into_super().owner().clone_ref(py);
        unsafe { Value::from_any(py, owner, callee) }
    }

    /// The arguments to the call.
    ///
    /// :type: List[Value]
    #[getter]
    fn args(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let inst = slf.into_super();
        let mut args = Instruction::operands(inst, py)?;
        args.pop().unwrap();
        Ok(args)
    }
}

/// A phi node instruction.
#[pyclass(extends = Instruction, unsendable)]
pub(crate) struct Phi(PhiValue<'static>);

#[pymethods]
impl Phi {
    /// The incoming values and their preceding basic blocks.
    ///
    /// :type: List[Tuple[Value, BasicBlock]]
    #[getter]
    fn incoming(slf: PyRef<Self>, py: Python) -> PyResult<Vec<(PyObject, PyObject)>> {
        let phi = slf.0;
        let value = slf.into_super().into_super();
        let owner = value.owner();

        (0..phi.count_incoming())
            .map(|i| {
                let (value, block) = phi.get_incoming(i).unwrap();
                let value = unsafe { Value::from_any(py, owner.clone_ref(py), value)? };
                let block = unsafe { Value::from_any(py, owner.clone_ref(py), block)? };
                Ok((value, block))
            })
            .collect()
    }
}

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
    context::{self, Context},
    types::Type,
    utils::{
        any_to_meta, call_if_some, clone_module, extract_constant, function_type,
        try_callable_value,
    },
    values::{self, BasicBlock, Value},
};
use either::Either::{Left, Right};
use inkwell::{
    attributes::Attribute as InkwellAttribute,
    builder::Builder as InkwellBuilder,
    context::Context as InkwellContext,
    memory_buffer::MemoryBuffer,
    module::Module as InkwellModule,
    types::AnyTypeEnum,
    values::{InstructionOpcode, PointerValue},
    values::{InstructionValue, IntValue, PhiValue},
};
use pyo3::{
    conversion::ToPyObject,
    exceptions::{PyOSError, PyValueError},
    prelude::*,
    types::{PyBytes, PySequence, PyString, PyUnicode},
};
use qirlib::{module, types, BuilderBasicQisExt};
use std::{
    convert::{Into, TryInto},
    mem::transmute,
    result::Result,
};

#[pyclass(unsendable)]
pub(crate) struct Module {
    module: InkwellModule<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Module {
    #[staticmethod]
    #[pyo3(text_signature = "(ir)")]
    fn from_ir(py: Python, ir: &str) -> PyResult<Self> {
        let context = InkwellContext::create();
        let buffer = MemoryBuffer::create_from_memory_range(ir.as_bytes(), "");
        let module = context
            .create_module_from_ir(buffer)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let module = unsafe { transmute::<InkwellModule<'_>, InkwellModule<'static>>(module) };
        let context = Py::new(py, Context::new(context))?;
        Ok(Self { module, context })
    }

    #[staticmethod]
    #[pyo3(text_signature = "(bitcode)")]
    fn from_bitcode(py: Python, bitcode: &[u8]) -> PyResult<Self> {
        let context = InkwellContext::create();
        let buffer = MemoryBuffer::create_from_memory_range(bitcode, "");
        let module = InkwellModule::parse_bitcode_from_buffer(&buffer, &context)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let module = unsafe { transmute::<InkwellModule<'_>, InkwellModule<'static>>(module) };
        let context = Py::new(py, Context::new(context))?;
        Ok(Self { module, context })
    }

    #[getter]
    fn functions(&self, py: Python) -> PyResult<Vec<PyObject>> {
        self.module
            .get_functions()
            .map(|f| unsafe { Value::from_any(py, self.context.clone(), f) })
            .collect()
    }

    #[getter]
    fn bitcode<'py>(&self, py: Python<'py>) -> &'py PyBytes {
        PyBytes::new(py, self.module.write_bitcode_to_memory().as_slice())
    }

    fn __str__(&self) -> String {
        self.module.to_string()
    }
}

impl Module {
    fn new(py: Python, context: Py<Context>, name: &str) -> Self {
        let module = {
            let context = context.borrow(py);
            let module = context.create_module(name);
            unsafe { transmute::<InkwellModule<'_>, InkwellModule<'static>>(module) }
        };
        Self { module, context }
    }
}

/// Provides access to all supported types.
#[pyclass]
pub(crate) struct TypeFactory {
    module: Py<Module>,
}

#[pymethods]
impl TypeFactory {
    /// The void type.
    ///
    /// :type: Type
    #[getter]
    fn void(&self, py: Python) -> PyResult<PyObject> {
        self.create_type(py, |m| m.get_context().void_type().into())
    }

    /// The boolean type.
    ///
    /// :type: Type
    #[getter]
    fn bool(&self, py: Python) -> PyResult<PyObject> {
        self.create_type(py, |m| m.get_context().bool_type().into())
    }

    /// An integer type.
    ///
    /// :param int width: The number of bits in the integers.
    /// :returns: The integer type.
    /// :rtype: Type
    #[pyo3(text_signature = "(width)")]
    fn int(&self, py: Python, width: u32) -> PyResult<PyObject> {
        self.create_type(py, |m| m.get_context().custom_width_int_type(width).into())
    }

    /// The double type.
    ///
    /// :type: Type
    #[getter]
    fn double(&self, py: Python) -> PyResult<PyObject> {
        self.create_type(py, |m| m.get_context().f64_type().into())
    }

    /// The qubit type.
    ///
    /// :type: Type
    #[getter]
    fn qubit(&self, py: Python) -> PyResult<PyObject> {
        self.create_type(py, |m| types::qubit(m).into())
    }

    /// The measurement result type.
    ///
    /// :type: Type
    #[getter]
    fn result(&self, py: Python) -> PyResult<PyObject> {
        self.create_type(py, |m| types::result(m).into())
    }

    /// A function type.
    ///
    /// :param Type ret: The return type.
    /// :param List[Type] params: The parameter types.
    /// :returns: The function type.
    /// :rtype: Type
    #[staticmethod]
    #[pyo3(text_signature = "(ret, params)")]
    #[allow(clippy::needless_pass_by_value)]
    fn function(py: Python, ret: &Type, params: Vec<Py<Type>>) -> PyResult<PyObject> {
        context::require_same(
            py,
            params
                .iter()
                .map(|t| t.borrow(py).context().clone())
                .chain([ret.context().clone()]),
        )?;

        let ty = function_type(
            &ret.get(),
            params.iter().map(|t| unsafe {
                transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(t.borrow(py).get())
            }),
        )
        .ok_or_else(|| PyValueError::new_err("Invalid return or parameter type."))?;

        unsafe { Type::from_any(py, ret.context().clone(), ty.into()) }
    }
}

impl TypeFactory {
    fn create_type(
        &self,
        py: Python,
        f: impl for<'ctx> Fn(&InkwellModule<'ctx>) -> AnyTypeEnum<'ctx>,
    ) -> PyResult<PyObject> {
        let module = self.module.borrow(py);
        let context = module.context.clone();
        let ty = f(&module.module);
        unsafe { Type::from_any(py, context, ty) }
    }
}

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

#[pyclass]
#[derive(Clone)]
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

#[pyclass]
#[derive(Clone)]
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

#[pyclass(extends = Value, subclass, unsendable)]
pub(crate) struct Instruction(InstructionValue<'static>);

#[pymethods]
impl Instruction {
    #[getter]
    fn opcode(&self) -> Opcode {
        self.0.get_opcode().into()
    }

    #[getter]
    fn operands(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let inst = slf.0;
        let value = slf.into_super();
        let context = value.context();
        (0..inst.get_num_operands())
            .map(|i| match inst.get_operand(i).unwrap() {
                Left(value) => unsafe { Value::from_any(py, context.clone(), value) },
                Right(block) => unsafe { Value::from_any(py, context.clone(), block) },
            })
            .collect()
    }

    #[getter]
    fn successors(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
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

impl Instruction {
    pub(crate) unsafe fn new_subtype(
        py: Python,
        context: Py<Context>,
        inst: InstructionValue,
    ) -> PyResult<PyObject> {
        let inst = transmute::<InstructionValue<'_>, InstructionValue<'static>>(inst);
        let base = Value::init(context, inst.into()).add_subclass(Self(inst));
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

#[pyclass(extends = Instruction)]
pub(crate) struct Switch;

#[pymethods]
impl Switch {
    #[getter]
    fn cond(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let inst = slf.into_super();
        let cond = inst.0.get_operand(0).unwrap().left().unwrap();
        let context = inst.into_super().context().clone();
        unsafe { Value::from_any(py, context, cond) }
    }

    #[getter]
    fn default(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let inst = slf.into_super();
        let block = inst.0.get_operand(1).unwrap().right().unwrap();
        let context = inst.into_super().context().clone();
        unsafe { Value::from_any(py, context, block) }
    }

    #[getter]
    fn cases(slf: PyRef<Self>, py: Python) -> PyResult<Vec<(PyObject, PyObject)>> {
        let inst_ref = slf.into_super();
        let inst = inst_ref.0;
        let value = inst_ref.into_super();
        let context = value.context();

        (2..inst.get_num_operands())
            .step_by(2)
            .map(|i| {
                let cond = inst.get_operand(i).unwrap().left().unwrap();
                let cond = unsafe { Value::from_any(py, context.clone(), cond)? };
                let succ = inst.get_operand(i + 1).unwrap().right().unwrap();
                let succ = unsafe { Value::from_any(py, context.clone(), succ)? };
                Ok((cond, succ))
            })
            .collect()
    }
}

#[pyclass(extends = Instruction)]
pub(crate) struct ICmp;

#[pymethods]
impl ICmp {
    #[getter]
    fn predicate(slf: PyRef<Self>) -> IntPredicate {
        slf.into_super().0.get_icmp_predicate().unwrap().into()
    }
}

#[pyclass(extends = Instruction)]
pub(crate) struct FCmp;

#[pymethods]
impl FCmp {
    #[getter]
    fn predicate(slf: PyRef<Self>) -> FloatPredicate {
        slf.into_super().0.get_fcmp_predicate().unwrap().into()
    }
}

#[pyclass(extends = Instruction, unsendable)]
pub(crate) struct Call;

#[pymethods]
impl Call {
    #[getter]
    fn callee(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let inst = slf.into_super();
        let last = inst.0.get_operand(inst.0.get_num_operands() - 1);
        let callee = last.unwrap().left().unwrap();
        let context = inst.into_super().context().clone();
        unsafe { Value::from_any(py, context, callee) }
    }

    #[getter]
    fn args(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let inst = slf.into_super();
        let mut args = Instruction::operands(inst, py)?;
        args.pop().unwrap();
        Ok(args)
    }
}

#[pyclass(extends = Instruction, unsendable)]
pub(crate) struct Phi(PhiValue<'static>);

#[pymethods]
impl Phi {
    #[getter]
    fn incoming(slf: PyRef<Self>, py: Python) -> PyResult<Vec<(PyObject, PyObject)>> {
        let phi = slf.0;
        let value = slf.into_super().into_super();
        let context = value.context();

        (0..phi.count_incoming())
            .map(|i| {
                let (value, block) = phi.get_incoming(i).unwrap();
                let value = unsafe { Value::from_any(py, context.clone(), value)? };
                let block = unsafe { Value::from_any(py, context.clone(), block)? };
                Ok((value, block))
            })
            .collect()
    }
}

/// An instruction builder.
#[pyclass(unsendable)]
pub(crate) struct Builder {
    builder: InkwellBuilder<'static>,
    context: Py<Context>,
    // TODO: In principle, the module could be extracted from the builder.
    // See https://github.com/TheDan64/inkwell/issues/347.
    module: Py<Module>,
}

impl Builder {
    fn new(py: Python, module: Py<Module>) -> Self {
        let context = module.borrow(py).context.clone();
        let builder = {
            let context = context.borrow(py);
            let builder = context.create_builder();
            unsafe { transmute::<InkwellBuilder<'_>, InkwellBuilder<'static>>(builder) }
        };

        Self {
            builder,
            context,
            module,
        }
    }
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
        let value =
            self.builder
                .build_and::<IntValue>(lhs.get().try_into()?, rhs.get().try_into()?, "");
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
        let value =
            self.builder
                .build_or::<IntValue>(lhs.get().try_into()?, rhs.get().try_into()?, "");
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
        let value =
            self.builder
                .build_xor::<IntValue>(lhs.get().try_into()?, rhs.get().try_into()?, "");
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
            lhs.get().try_into()?,
            rhs.get().try_into()?,
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
            lhs.get().try_into()?,
            rhs.get().try_into()?,
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
            lhs.get().try_into()?,
            rhs.get().try_into()?,
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
            lhs.get().try_into()?,
            rhs.get().try_into()?,
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
            lhs.get().try_into()?,
            rhs.get().try_into()?,
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
            lhs.get().try_into()?,
            rhs.get().try_into()?,
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

        let (callable, param_types) = try_callable_value(callee.get())
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
                let value = unsafe { values::extract_inkwell(&t, v?) }?;
                any_to_meta(value).ok_or_else(|| PyValueError::new_err("Invalid argument."))
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
        let builder = qirlib::Builder::from(&self.builder, &module.module);
        builder.try_build_if(
            cond.get().try_into()?,
            |_| call_if_some(r#true),
            |_| call_if_some(r#false),
        )
    }
}

/// A simple module represents an executable program with these restrictions:
///
/// - There is one global qubit register and one global result register. Both are statically
///   allocated with a fixed size.
/// - There is only a single function that runs as the entry point.
///
/// :param str name: The name of the module.
/// :param int num_qubits: The number of statically allocated qubits.
/// :param int num_results: The number of statically allocated results.
#[pyclass(unsendable)]
#[pyo3(text_signature = "(name, num_qubits, num_results)")]
pub(crate) struct SimpleModule {
    module: Py<Module>,
    builder: Py<Builder>,
    types: Py<TypeFactory>,
    num_qubits: u64,
    num_results: u64,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(py: Python, name: &str, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let context = Py::new(py, Context::new(InkwellContext::create()))?;
        let module = Py::new(py, Module::new(py, context, name))?;
        let builder = Py::new(py, Builder::new(py, module.clone()))?;

        {
            let builder = builder.borrow(py);
            let module = module.borrow(py);
            module::simple_init(&module.module, &builder.builder, num_qubits, num_results);
        }

        let types = Py::new(
            py,
            TypeFactory {
                module: module.clone(),
            },
        )?;

        Ok(SimpleModule {
            module,
            builder,
            types,
            num_qubits,
            num_results,
        })
    }

    #[getter]
    fn types(&self) -> Py<TypeFactory> {
        self.types.clone()
    }

    /// The global qubit register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn qubits(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let builder = self.builder.borrow(py);
        let module = self.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
        (0..self.num_qubits)
            .map(|id| unsafe {
                Value::from_any(py, module.context.clone(), builder.build_qubit(id))
            })
            .collect()
    }

    /// The global result register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn results(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let builder = self.builder.borrow(py);
        let module = self.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
        (0..self.num_results)
            .map(|id| unsafe {
                Value::from_any(py, module.context.clone(), builder.build_result(id))
            })
            .collect()
    }

    /// The instruction builder.
    ///
    /// :type: Builder
    #[getter]
    fn builder(&self) -> Py<Builder> {
        self.builder.clone()
    }

    /// Emits the LLVM IR for the module as plain text.
    ///
    /// :rtype: str
    fn ir(&self, py: Python) -> PyResult<String> {
        self.emit(py, |m| m.print_to_string().to_string())
    }

    /// Emits the LLVM bitcode for the module as a sequence of bytes.
    ///
    /// :rtype: bytes
    fn bitcode<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        self.emit(py, |m| {
            PyBytes::new(py, m.write_bitcode_to_memory().as_slice())
        })
    }

    /// Adds a declaration for an externally linked function to the module.
    ///
    /// :param str name: The name of the function.
    /// :param Type ty: The type of the function.
    /// :return: The function value.
    /// :rtype: Function
    #[pyo3(text_signature = "(self, name, ty)")]
    fn add_external_function(&mut self, py: Python, name: &str, ty: &Type) -> PyResult<PyObject> {
        let module = self.module.borrow(py);
        context::require_same(py, [&module.context, ty.context()])?;

        let context = ty.context().clone();
        let ty = unsafe { transmute::<AnyTypeEnum<'_>, AnyTypeEnum<'static>>(ty.get()) }
            .into_function_type();
        let function = module.module.add_function(name, ty, None);
        unsafe { Value::from_any(py, context, function) }
    }
}

impl SimpleModule {
    fn emit<T>(&self, py: Python, f: impl Fn(&InkwellModule) -> T) -> PyResult<T> {
        let module = self.module.borrow(py);
        let builder = self.builder.borrow(py);
        let ret = builder.builder.build_return(None);
        let new_context = InkwellContext::create();
        let new_module = clone_module(&module.module, &new_context)?;
        ret.erase_from_basic_block();
        module::simple_finalize(&new_module).map_err(PyOSError::new_err)?;
        Ok(f(&new_module))
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
        context::require_same(py, [&builder.context, control.context(), target.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, control.context(), target.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context(), result.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
                .chain([builder.context.clone(), qubit.context().clone()]),
        )?;

        let context = builder.context.borrow(py);
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
                .chain([builder.context.clone(), qubit.context().clone()]),
        )?;

        let context = builder.context.borrow(py);
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
                .chain([builder.context.clone(), qubit.context().clone()]),
        )?;

        let context = builder.context.borrow(py);
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, qubit.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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
        context::require_same(py, [&builder.context, cond.context()])?;
        let module = builder.module.borrow(py);
        let builder = qirlib::Builder::from(&builder.builder, &module.module);
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

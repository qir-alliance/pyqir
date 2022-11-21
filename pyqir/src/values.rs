// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    context::{self, Context},
    instructions::Instruction,
    module::{Attribute, Module},
    types::{FunctionType, Type},
};
use inkwell::{
    attributes::AttributeLoc,
    types::{AnyType, AnyTypeEnum},
    values::{
        AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue,
        GlobalValue, InstructionValue, IntValue, PointerValue,
    },
    LLVMReference,
};
use libc::c_char;
use llvm_sys::{
    core::{
        LLVMBasicBlockAsValue, LLVMDisposeMessage, LLVMGetValueKind, LLVMGetValueName2,
        LLVMIsConstant, LLVMPrintValueToString,
    },
    prelude::*,
    LLVMValueKind,
};
use pyo3::{
    conversion::ToPyObject,
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::PyBytes,
};
use qirlib::values;
use std::{
    convert::{Into, TryFrom, TryInto},
    ffi::CStr,
    fmt::{self, Display, Formatter},
    mem::transmute,
    ops::Deref,
    slice,
};

/// A value.
#[pyclass(subclass, unsendable)]
#[derive(Clone)]
pub(crate) struct Value {
    value: AnyValue<'static>,
    context: Py<Context>,
}

#[pymethods]
impl Value {
    /// The type of this value.
    ///
    /// :type: Type
    #[getter]
    fn r#type(&self, py: Python) -> PyResult<PyObject> {
        unsafe { Type::from_any(py, self.context.clone(), self.value.ty()) }
    }

    /// The name of this value or the empty string if this value is anonymous.
    #[getter]
    fn name(&self) -> &str {
        self.value
            .name()
            .to_str()
            .expect("Name is not valid UTF-8.")
    }

    fn __str__(&self) -> String {
        self.value.to_string()
    }
}

impl Value {
    pub(crate) unsafe fn from_any<'ctx>(
        py: Python,
        context: Py<Context>,
        value: impl Into<AnyValue<'ctx>>,
    ) -> PyResult<PyObject> {
        let value = transmute::<AnyValue<'_>, AnyValue<'static>>(value.into());
        #[allow(clippy::same_functions_in_if_condition)]
        if let Ok(inst) = value.try_into() {
            Instruction::from_inst(py, context, inst)
        } else if let Ok(block) = value.try_into() {
            let base = PyClassInitializer::from(Self { value, context });
            let block = base.add_subclass(BasicBlock(block));
            Ok(Py::new(py, block)?.to_object(py))
        } else if value.is_const() {
            Constant::from_any(py, context, value)
        } else {
            Ok(Py::new(py, Self { value, context })?.to_object(py))
        }
    }

    pub(crate) unsafe fn init(context: Py<Context>, value: AnyValue) -> PyClassInitializer<Self> {
        let value = transmute::<AnyValue<'_>, AnyValue<'static>>(value);
        PyClassInitializer::from(Self { value, context })
    }

    pub(crate) unsafe fn get(&self) -> AnyValue<'static> {
        self.value
    }

    pub(crate) fn context(&self) -> &Py<Context> {
        &self.context
    }
}

/// A basic block.
///
/// :param Context context: The global context.
/// :param str name: The block name.
/// :param Optional[Function] parent: The parent function.
/// :param Optional[BasicBlock] before: The block to insert this block before.
#[pyclass(extends = Value, unsendable)]
#[pyo3(text_signature = "(context, name, parent=None, before=None)")]
pub(crate) struct BasicBlock(inkwell::basic_block::BasicBlock<'static>);

#[pymethods]
impl BasicBlock {
    #[new]
    fn new(
        py: Python,
        context: Py<Context>,
        name: &str,
        parent: Option<&Function>,
        before: Option<&BasicBlock>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let block = {
            let context = context.borrow(py);
            let block = match (parent, before) {
                (None, None) => Err(PyValueError::new_err("Can't create block without parent.")),
                (Some(parent), None) => Ok(context.append_basic_block(parent.0, name)),
                (Some(parent), Some(before)) if before.0.get_parent() != Some(parent.0) => Err(
                    PyValueError::new_err("Insert before block isn't in parent function."),
                ),
                (_, Some(before)) => Ok(context.prepend_basic_block(before.0, name)),
            }?;

            unsafe {
                transmute::<
                    inkwell::basic_block::BasicBlock<'_>,
                    inkwell::basic_block::BasicBlock<'static>,
                >(block)
            }
        };

        let value = block.into();
        Ok(PyClassInitializer::from(Value { value, context }).add_subclass(Self(block)))
    }

    /// The instructions in this basic block.
    ///
    /// :type: List[Instruction]
    #[getter]
    fn instructions(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let block = slf.0;
        let context = &slf.into_super().context;
        let mut insts = Vec::new();
        let mut inst = block.get_first_instruction();

        while let Some(i) = inst {
            insts.push(unsafe { Instruction::from_inst(py, context.clone(), i) }?);
            inst = i.get_next_instruction();
        }

        Ok(insts)
    }

    /// The terminating instruction of this basic block if there is one.
    ///
    /// :type: Optional[Instruction]
    #[getter]
    fn terminator(slf: PyRef<Self>, py: Python) -> PyResult<Option<PyObject>> {
        match slf.0.get_terminator() {
            Some(terminator) => {
                let context = slf.into_super().context.clone();
                unsafe { Instruction::from_inst(py, context, terminator) }.map(Some)
            }
            None => Ok(None),
        }
    }
}

impl BasicBlock {
    pub(crate) unsafe fn get(&self) -> inkwell::basic_block::BasicBlock<'static> {
        self.0
    }
}

/// A constant value.
#[pyclass(extends = Value, subclass)]
pub(crate) struct Constant;

#[pymethods]
impl Constant {
    /// Creates the null or zero constant for the given type.
    ///
    /// :param Type type: The type of the constant.
    /// :returns: The null or zero constant.
    /// :rtype: Constant
    #[staticmethod]
    #[pyo3(text_signature = "(ty)")]
    fn null(py: Python, ty: &Type) -> PyResult<PyObject> {
        let value: AnyValueEnum = match unsafe { ty.get() } {
            AnyTypeEnum::ArrayType(a) => Ok(a.const_zero().into()),
            AnyTypeEnum::FloatType(f) => Ok(f.const_zero().into()),
            AnyTypeEnum::IntType(i) => Ok(i.const_zero().into()),
            AnyTypeEnum::PointerType(p) => Ok(p.const_zero().into()),
            AnyTypeEnum::StructType(s) => Ok(s.const_zero().into()),
            AnyTypeEnum::VectorType(v) => Ok(v.const_zero().into()),
            AnyTypeEnum::FunctionType(_) | AnyTypeEnum::VoidType(_) => {
                Err(PyValueError::new_err("Can't create null for this type."))
            }
        }?;
        unsafe { Value::from_any(py, ty.context().clone(), value) }
    }

    /// Whether this value is the null value for its type.
    ///
    /// :type: bool
    #[getter]
    fn is_null(slf: PyRef<Self>) -> bool {
        slf.into_super().value.is_null()
    }
}

impl Constant {
    unsafe fn from_any(py: Python, context: Py<Context>, value: AnyValue) -> PyResult<PyObject> {
        let value = transmute::<AnyValue<'_>, AnyValue<'static>>(value);
        let kind = unsafe { LLVMGetValueKind(value.get_ref()) };
        let base = PyClassInitializer::from(Value { value, context }).add_subclass(Constant);

        if kind == LLVMValueKind::LLVMConstantExprValueKind {
            Ok(Py::new(py, base.add_subclass(ConstantExpr))?.to_object(py))
        } else if value.is_const() {
            match value.try_into() {
                Ok(AnyValueEnum::IntValue(_)) => {
                    Ok(Py::new(py, base.add_subclass(IntConstant))?.to_object(py))
                }
                Ok(AnyValueEnum::FloatValue(_)) => {
                    Ok(Py::new(py, base.add_subclass(FloatConstant))?.to_object(py))
                }
                Ok(AnyValueEnum::FunctionValue(f)) => {
                    Ok(Py::new(py, base.add_subclass(Function(f)))?.to_object(py))
                }
                _ => Ok(Py::new(py, base)?.to_object(py)),
            }
        } else {
            Err(PyValueError::new_err("Value is not constant."))
        }
    }
}

/// A constant integer value.
#[pyclass(extends = Constant)]
pub(crate) struct IntConstant;

#[pymethods]
impl IntConstant {
    /// The value.
    ///
    /// :type: int
    #[getter]
    fn value(slf: PyRef<Self>) -> u64 {
        let int: IntValue = slf.into_super().into_super().value.try_into().unwrap();
        int.get_zero_extended_constant().unwrap()
    }
}

/// A constant floating-point value.
#[pyclass(extends = Constant)]
pub(crate) struct FloatConstant;

#[pymethods]
impl FloatConstant {
    /// The value.
    ///
    /// :type: float
    #[getter]
    fn value(slf: PyRef<Self>) -> f64 {
        let float: FloatValue = slf.into_super().into_super().value.try_into().unwrap();
        float.get_constant().unwrap().0
    }
}

/// A function value.
///
/// :param FunctionType ty: The function type.
/// :param Linkage linkage: The linkage kind.
/// :param str name: The function name.
/// :param Module module: The parent module.
#[pyclass(extends = Constant, unsendable)]
pub(crate) struct Function(FunctionValue<'static>);

#[pymethods]
impl Function {
    #[new]
    fn new(
        py: Python,
        ty: PyRef<FunctionType>,
        linkage: Linkage,
        name: &str,
        module: &Module,
    ) -> PyResult<PyClassInitializer<Self>> {
        let function_ty = unsafe { ty.get() };
        let context = module.context();
        context::require_same(py, [ty.into_super().context(), context])?;
        let function =
            unsafe { module.get() }.add_function(name, function_ty, Some(linkage.into()));
        Ok(unsafe { Value::init(context.clone(), function.into()) }
            .add_subclass(Constant)
            .add_subclass(Self(function)))
    }

    /// The parameters to this function.
    ///
    /// :type: List[Value]
    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let params = slf.0.get_params();
        let context = &slf.into_super().into_super().context;
        params
            .into_iter()
            .map(|p| unsafe { Value::from_any(py, context.clone(), p) })
            .collect()
    }

    /// The basic blocks in this function.
    ///
    /// :type: List[BasicBlock]
    #[getter]
    fn basic_blocks(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let function = slf.0;
        let context = &slf.into_super().into_super().context;
        function
            .get_basic_blocks()
            .into_iter()
            .map(|b| unsafe { Value::from_any(py, context.clone(), b) })
            .collect()
    }

    /// Gets an attribute of this function with the given name if it has one.
    ///
    /// :param str name: The name of the attribute.
    /// :rtype: Optional[Attribute]
    /// :returns: The attribute.
    #[pyo3(text_signature = "(name)")]
    fn attribute(&self, name: &str) -> Option<Attribute> {
        Some(Attribute(
            self.0.get_string_attribute(AttributeLoc::Function, name)?,
        ))
    }
}

impl Function {
    pub(crate) unsafe fn get(&self) -> FunctionValue<'static> {
        self.0
    }
}

/// The linkage kind for a global value in a module.
#[pyclass]
#[derive(Clone)]
pub(crate) enum Linkage {
    #[pyo3(name = "APPENDING")]
    Appending,
    #[pyo3(name = "AVAILABLE_EXTERNALLY")]
    AvailableExternally,
    #[pyo3(name = "COMMON")]
    Common,
    #[pyo3(name = "EXTERNAL")]
    External,
    #[pyo3(name = "EXTERNAL_WEAK")]
    ExternalWeak,
    #[pyo3(name = "INTERNAL")]
    Internal,
    #[pyo3(name = "LINK_ONCE_ANY")]
    LinkOnceAny,
    #[pyo3(name = "LINK_ONCE_ODR")]
    LinkOnceOdr,
    #[pyo3(name = "PRIVATE")]
    Private,
    #[pyo3(name = "WEAK_ANY")]
    WeakAny,
    #[pyo3(name = "WEAK_ODR")]
    WeakOdr,
}

impl From<Linkage> for inkwell::module::Linkage {
    fn from(linkage: Linkage) -> Self {
        match linkage {
            Linkage::Appending => Self::Appending,
            Linkage::AvailableExternally => Self::AvailableExternally,
            Linkage::Common => Self::Common,
            Linkage::External => Self::External,
            Linkage::ExternalWeak => Self::ExternalWeak,
            Linkage::Internal => Self::Internal,
            Linkage::LinkOnceAny => Self::LinkOnceAny,
            Linkage::LinkOnceOdr => Self::LinkOnceODR,
            Linkage::Private => Self::Private,
            Linkage::WeakAny => Self::WeakAny,
            Linkage::WeakOdr => Self::WeakODR,
        }
    }
}

#[pyclass(extends = Constant)]
pub(crate) struct ConstantExpr;

#[pymethods]
impl ConstantExpr {
    /// Creates a `getelementptr` (GEP) constant expression.
    ///
    /// :param Value value: The aggregate value.
    /// :param Sequence[Value] indices: The indices of the element.
    /// :param bool inbounds: Whether to create an in-bounds GEP.
    /// :returns: The GEP constant expression.
    /// :rtype: ConstantExpr
    #[staticmethod]
    #[allow(clippy::needless_pass_by_value)]
    fn getelementptr(
        py: Python,
        value: &Value,
        indices: Vec<Value>,
        inbounds: bool,
    ) -> PyResult<PyObject> {
        let indices = indices
            .iter()
            .map(|i| IntValue::try_from(i.value).map_err(Into::into))
            .collect::<PyResult<Vec<_>>>()?;
        let pointer = PointerValue::try_from(value.value)?;
        let gep = if inbounds {
            unsafe { pointer.const_in_bounds_gep(&indices) }
        } else {
            unsafe { pointer.const_gep(&indices) }
        };
        unsafe { Value::from_any(py, value.context.clone(), gep) }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum AnyValue<'ctx> {
    Any(AnyValueEnum<'ctx>),
    BasicBlock(inkwell::basic_block::BasicBlock<'ctx>),
}

impl<'ctx> AnyValue<'ctx> {
    fn ty(&self) -> AnyTypeEnum<'ctx> {
        match self {
            Self::Any(AnyValueEnum::ArrayValue(a)) => a.get_type().into(),
            Self::Any(AnyValueEnum::IntValue(i)) => i.get_type().into(),
            Self::Any(AnyValueEnum::FloatValue(f)) => f.get_type().into(),
            Self::Any(AnyValueEnum::PhiValue(p)) => p.as_instruction().get_type(),
            Self::Any(AnyValueEnum::FunctionValue(f)) => f.get_type().into(),
            Self::Any(AnyValueEnum::PointerValue(p)) => p.get_type().into(),
            Self::Any(AnyValueEnum::StructValue(s)) => s.get_type().into(),
            Self::Any(AnyValueEnum::VectorValue(v)) => v.get_type().into(),
            Self::Any(AnyValueEnum::InstructionValue(i)) => i.get_type(),
            Self::Any(AnyValueEnum::MetadataValue(m)) => {
                inkwell::values::AnyValue::as_any_value_enum(m).get_type()
            }
            Self::BasicBlock(b) => b.get_context().void_type().into(),
        }
    }

    fn name(&self) -> &CStr {
        let mut len = 0;
        let name = unsafe { LLVMGetValueName2(self.get_ref(), &mut len) };
        let name = unsafe { slice::from_raw_parts(name.cast(), len + 1) };
        CStr::from_bytes_with_nul(name).expect("Name is not a valid C string.")
    }

    fn is_const(&self) -> bool {
        unsafe { LLVMIsConstant(self.get_ref()) != 0 }
    }

    fn is_null(&self) -> bool {
        match self {
            Self::Any(AnyValueEnum::PointerValue(p)) => p.is_null(),
            Self::Any(_) | Self::BasicBlock(_) => false,
        }
    }
}

impl<'ctx> Display for AnyValue<'ctx> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Any(any) => {
                let message = inkwell::values::AnyValue::print_to_string(any);
                f.write_str(message.to_str().map_err(|_| fmt::Error)?)
            }
            Self::BasicBlock(block) => {
                let value = unsafe { LLVMBasicBlockAsValue(block.get_ref()) };
                let message = unsafe { Message(LLVMPrintValueToString(value)) };
                f.write_str(message.to_str().map_err(|_| fmt::Error)?)
            }
        }
    }
}

impl LLVMReference<LLVMValueRef> for AnyValue<'_> {
    unsafe fn get_ref(&self) -> LLVMValueRef {
        match self {
            Self::Any(any) => any.get_ref(),
            Self::BasicBlock(block) => LLVMBasicBlockAsValue(block.get_ref()),
        }
    }
}

impl<'ctx> From<AnyValueEnum<'ctx>> for AnyValue<'ctx> {
    fn from(any: AnyValueEnum<'ctx>) -> Self {
        Self::Any(any)
    }
}

impl<'ctx> From<BasicValueEnum<'ctx>> for AnyValue<'ctx> {
    fn from(basic: BasicValueEnum<'ctx>) -> Self {
        Self::Any(basic.into())
    }
}

impl<'ctx> From<IntValue<'ctx>> for AnyValue<'ctx> {
    fn from(int: IntValue<'ctx>) -> Self {
        Self::Any(int.into())
    }
}

impl<'ctx> From<FloatValue<'ctx>> for AnyValue<'ctx> {
    fn from(float: FloatValue<'ctx>) -> Self {
        Self::Any(float.into())
    }
}

impl<'ctx> From<FunctionValue<'ctx>> for AnyValue<'ctx> {
    fn from(function: FunctionValue<'ctx>) -> Self {
        Self::Any(function.into())
    }
}

impl<'ctx> From<PointerValue<'ctx>> for AnyValue<'ctx> {
    fn from(pointer: PointerValue<'ctx>) -> Self {
        Self::Any(pointer.into())
    }
}

impl<'ctx> From<GlobalValue<'ctx>> for AnyValue<'ctx> {
    fn from(global: GlobalValue<'ctx>) -> Self {
        Self::Any(global.as_pointer_value().into())
    }
}

impl<'ctx> From<InstructionValue<'ctx>> for AnyValue<'ctx> {
    fn from(instruction: InstructionValue<'ctx>) -> Self {
        Self::Any(instruction.into())
    }
}

impl<'ctx> From<inkwell::basic_block::BasicBlock<'ctx>> for AnyValue<'ctx> {
    fn from(block: inkwell::basic_block::BasicBlock<'ctx>) -> Self {
        Self::BasicBlock(block)
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for AnyValueEnum<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(a) => Ok(a),
            AnyValue::BasicBlock(_) => Err(ConvertError("value excluding basic blocks")),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for BasicValueEnum<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::ArrayValue(a)) => Some(a.into()),
            AnyValue::Any(AnyValueEnum::IntValue(i)) => Some(i.into()),
            AnyValue::Any(AnyValueEnum::FloatValue(f)) => Some(f.into()),
            AnyValue::Any(AnyValueEnum::PointerValue(p)) => Some(p.into()),
            AnyValue::Any(AnyValueEnum::StructValue(s)) => Some(s.into()),
            AnyValue::Any(AnyValueEnum::VectorValue(v)) => Some(v.into()),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => i
                .try_into()
                .map(BasicValueEnum::IntValue)
                .or_else(|()| i.try_into().map(BasicValueEnum::FloatValue))
                .or_else(|()| i.try_into().map(BasicValueEnum::PointerValue))
                .ok(),
            AnyValue::Any(
                AnyValueEnum::PhiValue(_)
                | AnyValueEnum::FunctionValue(_)
                | AnyValueEnum::MetadataValue(_),
            )
            | AnyValue::BasicBlock(_) => None,
        }
        .ok_or(ConvertError("basic value"))
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for BasicMetadataValueEnum<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::MetadataValue(m)) => Ok(m.into()),
            _ => BasicValueEnum::try_from(value)
                .map(BasicMetadataValueEnum::from)
                .map_err(|_| ConvertError("argument value")),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for IntValue<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::IntValue(i)) => Some(i),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => i.try_into().ok(),
            _ => None,
        }
        .ok_or(ConvertError("integer value"))
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for FloatValue<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::FloatValue(f)) => Some(f),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => i.try_into().ok(),
            _ => None,
        }
        .ok_or(ConvertError("float value"))
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for PointerValue<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::PointerValue(p)) => Some(p),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => i.try_into().ok(),
            _ => None,
        }
        .ok_or(ConvertError("pointer value"))
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for InstructionValue<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::ArrayValue(a)) => a.as_instruction(),
            AnyValue::Any(AnyValueEnum::IntValue(i)) => i.as_instruction(),
            AnyValue::Any(AnyValueEnum::FloatValue(f)) => f.as_instruction(),
            AnyValue::Any(AnyValueEnum::PhiValue(p)) => Some(p.as_instruction()),
            AnyValue::Any(AnyValueEnum::PointerValue(p)) => p.as_instruction(),
            AnyValue::Any(AnyValueEnum::StructValue(s)) => s.as_instruction(),
            AnyValue::Any(AnyValueEnum::VectorValue(v)) => v.as_instruction(),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => Some(i),
            AnyValue::Any(AnyValueEnum::FunctionValue(_) | AnyValueEnum::MetadataValue(_))
            | AnyValue::BasicBlock(_) => None,
        }
        .ok_or(ConvertError("instruction value"))
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for inkwell::basic_block::BasicBlock<'ctx> {
    type Error = ConvertError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(_) => Err(ConvertError("basic block")),
            AnyValue::BasicBlock(b) => Ok(b),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ConvertError(&'static str);

impl From<ConvertError> for PyErr {
    fn from(error: ConvertError) -> Self {
        PyValueError::new_err(format!("Couldn't convert value to {}.", error.0))
    }
}

struct Message(*mut c_char);

impl Deref for Message {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0) }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe { LLVMDisposeMessage(self.0) }
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
    let value = extract_constant(unsafe { &ty.get() }, value)?;
    unsafe { Value::from_any(py, context, value) }
}

/// Creates a static qubit value.
///
/// :param Context context: The global context.
/// :param int id: The static qubit ID.
/// :returns: A static qubit value.
/// :rtype: Value
#[pyfunction]
pub(crate) fn qubit(py: Python, context: Py<Context>, id: u64) -> PyResult<PyObject> {
    let value = {
        let context = context.borrow(py);
        let value = values::qubit(&context.void_type().get_context(), id);
        unsafe { transmute::<PointerValue<'_>, PointerValue<'static>>(value) }
    };
    unsafe { Value::from_any(py, context, value) }
}

/// If the value is a static qubit ID, extracts it.
///
/// :param Value value: The value.
/// :returns: The static qubit ID.
/// :rtype: Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn qubit_id(value: &Value) -> Option<u64> {
    values::qubit_id(unsafe { value.get() }.try_into().ok()?)
}

/// Creates a static result value.
///
/// :param Context context: The global context.
/// :param int id: The static result ID.
/// :returns: A static result value.
/// :rtype: Value
#[pyfunction]
pub(crate) fn result(py: Python, context: Py<Context>, id: u64) -> PyResult<PyObject> {
    let value = {
        let context = context.borrow(py);
        let value = values::result(&context.void_type().get_context(), id);
        unsafe { transmute::<PointerValue<'_>, PointerValue<'static>>(value) }
    };
    unsafe { Value::from_any(py, context, value) }
}

/// If the value is a static result ID, extracts it.
///
/// :param Value value: The value.
/// :returns: The static result ID.
/// :rtype: Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn result_id(value: &Value) -> Option<u64> {
    values::result_id(unsafe { value.get() }.try_into().ok()?)
}

/// Creates an entry point.
///
/// :param Module module: The parent module.
/// :param str name: The entry point name.
/// :param int required_num_qubits: The number of qubits required by the entry point.
/// :param int required_num_results: The number of results required by the entry point.
/// :returns: An entry point.
/// :rtype: Function
#[pyfunction]
pub(crate) fn entry_point(
    py: Python,
    module: &Module,
    name: &str,
    required_num_qubits: u64,
    required_num_results: u64,
) -> PyResult<PyObject> {
    let entry_point = values::entry_point(
        unsafe { module.get() },
        name,
        required_num_qubits,
        required_num_results,
    );
    unsafe { Value::from_any(py, module.context().clone(), entry_point) }
}

/// Whether the function is an entry point.
///
/// :param Function function: The function.
/// :rtype: bool
/// :returns: True if the function is an entry point.
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn is_entry_point(function: &Function) -> bool {
    values::is_entry_point(unsafe { function.get() })
}

/// Whether the function is interop-friendly.
///
/// :param Function function: The function.
/// :rtype: bool
/// :returns: True if the function is interop-friendly.
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn is_interop_friendly(function: &Function) -> bool {
    values::is_interop_friendly(unsafe { function.get() })
}

/// If the function declares a required number of qubits, extracts it.
///
/// :param Function function: The function.
/// :rtype: Optional[int]
/// :returns: The required number of qubits.
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn required_num_qubits(function: &Function) -> Option<u64> {
    values::required_num_qubits(unsafe { function.get() })
}

/// If the function declares a required number of results, extracts it.
///
/// :param Function function: The function.
/// :rtype: Optional[int]
/// :returns: The required number of results.
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn required_num_results(function: &Function) -> Option<u64> {
    values::required_num_results(unsafe { function.get() })
}

/// If the value is a pointer to a constant byte array, extracts it.
///
/// :param Value value: The value.
/// :rtype: Optional[bytes]
/// :returns: The constant byte array.
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn extract_bytes<'p>(py: Python<'p>, value: &Value) -> Option<&'p PyBytes> {
    let bytes = values::extract_bytes(unsafe { value.get() }.try_into().ok()?)?;
    Some(PyBytes::new(py, bytes))
}

pub(crate) unsafe fn extract_any<'ctx>(
    ty: &impl AnyType<'ctx>,
    ob: &PyAny,
) -> PyResult<AnyValue<'ctx>> {
    ob.extract()
        .map(|v: Value| v.value)
        .or_else(|_| extract_constant(ty, ob))
}

fn extract_constant<'ctx>(ty: &impl AnyType<'ctx>, ob: &PyAny) -> PyResult<AnyValue<'ctx>> {
    match ty.as_any_type_enum() {
        AnyTypeEnum::IntType(int) => Ok(int.const_int(ob.extract()?, true).into()),
        AnyTypeEnum::FloatType(float) => Ok(float.const_float(ob.extract()?).into()),
        _ => Err(PyTypeError::new_err(
            "Can't convert Python value into this type.",
        )),
    }
}

pub(crate) fn extract_contexts<'a>(
    values: impl IntoIterator<Item = &'a PyAny> + 'a,
) -> impl Iterator<Item = Py<Context>> + 'a {
    values
        .into_iter()
        .filter_map(|v| Some(v.extract::<Value>().ok()?.context))
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    basic_block::BasicBlock,
    context::Context,
    module::Module,
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::{
        AnyValue as InkwellAnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum,
        CallableValue, FloatValue, FunctionValue, InstructionValue, IntValue, PointerValue,
    },
};
use pyo3::{
    exceptions::{PyOSError, PyTypeError, PyUnicodeDecodeError, PyValueError},
    prelude::*,
    PyClass,
};
use std::{
    borrow::Borrow,
    convert::{Into, TryFrom, TryInto},
    ffi::CStr,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub(crate) struct ConversionError {
    from: String,
    to: String,
}

impl ConversionError {
    fn new(from: impl Into<String>, to: impl Into<String>) -> ConversionError {
        ConversionError {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl From<ConversionError> for PyErr {
    fn from(error: ConversionError) -> Self {
        PyValueError::new_err(format!("Couldn't convert {} to {}.", error.from, error.to))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AnyValue<'ctx> {
    Any(AnyValueEnum<'ctx>),
    BasicBlock(BasicBlock<'ctx>),
}

impl<'ctx> AnyValue<'ctx> {
    pub(crate) fn ty(&self) -> AnyTypeEnum<'ctx> {
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
            Self::Any(AnyValueEnum::MetadataValue(m)) => m.as_any_value_enum().get_type(),
            Self::BasicBlock(b) => b.get_context().void_type().into(),
        }
    }

    pub(crate) fn name(&self) -> &CStr {
        match self {
            Self::Any(AnyValueEnum::ArrayValue(a)) => a.get_name(),
            Self::Any(AnyValueEnum::IntValue(i)) => i.get_name(),
            Self::Any(AnyValueEnum::FloatValue(f)) => f.get_name(),
            Self::Any(AnyValueEnum::PhiValue(p)) => p.get_name(),
            Self::Any(AnyValueEnum::FunctionValue(f)) => f.get_name(),
            Self::Any(AnyValueEnum::PointerValue(p)) => p.get_name(),
            Self::Any(AnyValueEnum::StructValue(s)) => s.get_name(),
            Self::Any(AnyValueEnum::VectorValue(v)) => v.get_name(),
            Self::Any(AnyValueEnum::InstructionValue(i)) => i
                .get_name()
                .unwrap_or_else(|| CStr::from_bytes_with_nul(b"\0").unwrap()),
            Self::Any(AnyValueEnum::MetadataValue(m)) => m.get_name(),
            Self::BasicBlock(b) => b.get_name(),
        }
    }

    pub(crate) fn is_const(&self) -> bool {
        match self {
            Self::Any(AnyValueEnum::ArrayValue(a)) => a.is_const(),
            Self::Any(AnyValueEnum::IntValue(i)) => i.is_const(),
            Self::Any(AnyValueEnum::FloatValue(f)) => f.is_const(),
            Self::Any(AnyValueEnum::PointerValue(p)) => p.is_const(),
            Self::Any(AnyValueEnum::StructValue(_)) => todo!(),
            Self::Any(AnyValueEnum::VectorValue(v)) => v.is_const(),
            Self::Any(AnyValueEnum::PhiValue(_) | AnyValueEnum::InstructionValue(_)) => false,
            Self::Any(AnyValueEnum::FunctionValue(_) | AnyValueEnum::MetadataValue(_))
            | AnyValue::BasicBlock(_) => true,
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        match self {
            Self::Any(AnyValueEnum::PointerValue(p)) => p.is_null(),
            Self::Any(_) | Self::BasicBlock(_) => false,
        }
    }
}

impl<'ctx> Display for AnyValue<'ctx> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Any(any) => write!(f, "{}", any.print_to_string()),
            Self::BasicBlock(_) => todo!(),
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

impl<'ctx> From<InstructionValue<'ctx>> for AnyValue<'ctx> {
    fn from(instruction: InstructionValue<'ctx>) -> Self {
        Self::Any(instruction.into())
    }
}

impl<'ctx> From<BasicBlock<'ctx>> for AnyValue<'ctx> {
    fn from(block: BasicBlock<'ctx>) -> Self {
        Self::BasicBlock(block)
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for AnyValueEnum<'ctx> {
    type Error = ConversionError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(a) => Ok(a),
            AnyValue::BasicBlock(_) => Err(ConversionError::new("value", "any value")),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for IntValue<'ctx> {
    type Error = ConversionError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        let err = || ConversionError::new("value", "integer value");
        match value {
            AnyValue::Any(AnyValueEnum::IntValue(i)) => Ok(i),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => i.try_into().map_err(|()| err()),
            _ => Err(err()),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for FloatValue<'ctx> {
    type Error = ConversionError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        let err = || ConversionError::new("value", "float value");
        match value {
            AnyValue::Any(AnyValueEnum::FloatValue(f)) => Ok(f),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => i.try_into().map_err(|()| err()),
            _ => Err(err()),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for PointerValue<'ctx> {
    type Error = ConversionError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        let err = || ConversionError::new("value", "pointer value");
        match value {
            AnyValue::Any(AnyValueEnum::PointerValue(p)) => Ok(p),
            AnyValue::Any(AnyValueEnum::InstructionValue(i)) => i.try_into().map_err(|()| err()),
            _ => Err(err()),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for InstructionValue<'ctx> {
    type Error = ConversionError;

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
        .ok_or_else(|| ConversionError::new("value", "instruction value"))
    }
}

pub(crate) fn extract_constant<'ctx>(
    ty: &impl AnyType<'ctx>,
    ob: &PyAny,
) -> PyResult<AnyValueEnum<'ctx>> {
    match ty.as_any_type_enum() {
        AnyTypeEnum::IntType(int) => Ok(int.const_int(ob.extract()?, true).into()),
        AnyTypeEnum::FloatType(float) => Ok(float.const_float(ob.extract()?).into()),
        _ => Err(PyTypeError::new_err(
            "Can't convert Python value into this type.",
        )),
    }
}

pub(crate) fn function_type<'ctx>(
    return_type: &impl AnyType<'ctx>,
    params: impl IntoIterator<Item = AnyTypeEnum<'ctx>>,
) -> Option<FunctionType<'ctx>> {
    let params = params
        .into_iter()
        .map(|ty| BasicTypeEnum::try_from(ty).map(Into::into).ok())
        .collect::<Option<Vec<_>>>()?;

    match return_type.as_any_type_enum() {
        AnyTypeEnum::VoidType(void) => Some(void.fn_type(&params, false)),
        any => BasicTypeEnum::try_from(any)
            .map(|basic| basic.fn_type(&params, false))
            .ok(),
    }
}

pub(crate) fn try_callable_value(value: AnyValue) -> Option<(CallableValue, Vec<BasicTypeEnum>)> {
    match value {
        AnyValue::Any(AnyValueEnum::FunctionValue(f)) => {
            Some((CallableValue::from(f), f.get_type().get_param_types()))
        }
        AnyValue::Any(AnyValueEnum::PointerValue(p)) => match p.get_type().get_element_type() {
            AnyTypeEnum::FunctionType(ty) => {
                Some((CallableValue::try_from(p).unwrap(), ty.get_param_types()))
            }
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn any_to_meta(value: AnyValueEnum) -> Option<BasicMetadataValueEnum> {
    match value {
        AnyValueEnum::ArrayValue(a) => Some(BasicMetadataValueEnum::ArrayValue(a)),
        AnyValueEnum::IntValue(i) => Some(BasicMetadataValueEnum::IntValue(i)),
        AnyValueEnum::FloatValue(f) => Some(BasicMetadataValueEnum::FloatValue(f)),
        AnyValueEnum::PointerValue(p) => Some(BasicMetadataValueEnum::PointerValue(p)),
        AnyValueEnum::StructValue(s) => Some(BasicMetadataValueEnum::StructValue(s)),
        AnyValueEnum::VectorValue(v) => Some(BasicMetadataValueEnum::VectorValue(v)),
        AnyValueEnum::MetadataValue(m) => Some(BasicMetadataValueEnum::MetadataValue(m)),
        AnyValueEnum::InstructionValue(i) => i
            .try_into()
            .map(BasicMetadataValueEnum::IntValue)
            .or_else(|()| i.try_into().map(BasicMetadataValueEnum::FloatValue))
            .or_else(|()| i.try_into().map(BasicMetadataValueEnum::PointerValue))
            .ok(),
        AnyValueEnum::PhiValue(_) | AnyValueEnum::FunctionValue(_) => None,
    }
}

pub(crate) fn basic_to_any(ty: BasicTypeEnum) -> AnyTypeEnum {
    match ty {
        BasicTypeEnum::ArrayType(a) => a.into(),
        BasicTypeEnum::FloatType(f) => f.into(),
        BasicTypeEnum::IntType(i) => i.into(),
        BasicTypeEnum::PointerType(p) => p.into(),
        BasicTypeEnum::StructType(s) => s.into(),
        BasicTypeEnum::VectorType(v) => v.into(),
    }
}

pub(crate) fn call_if_some(f: Option<&PyAny>) -> PyResult<()> {
    match f {
        Some(f) => f.call0().map(|_| ()),
        None => Ok(()),
    }
}

pub(crate) fn is_all_same<T>(
    py: Python,
    items: impl IntoIterator<Item = impl Borrow<Py<T>>>,
) -> bool
where
    T: Eq + PyClass,
{
    let mut items = items.into_iter();
    if let Some(mut prev) = items.next() {
        for item in items {
            if *item.borrow().borrow(py) != *prev.borrow().borrow(py) {
                return false;
            }
            prev = item;
        }
    };
    true
}

pub(crate) fn clone_module<'ctx>(
    module: &Module,
    context: &'ctx Context,
) -> PyResult<Module<'ctx>> {
    let bitcode = module.write_bitcode_to_memory();
    let new_module = Module::parse_bitcode_from_buffer(&bitcode, context).map_err(|e| {
        module.verify().err().map_or_else(
            || PyOSError::new_err(e.to_string()),
            |e| PyOSError::new_err(e.to_string()),
        )
    })?;
    let name = module
        .get_name()
        .to_str()
        .map_err(PyUnicodeDecodeError::new_err)?;
    new_module.set_name(name);
    Ok(new_module)
}

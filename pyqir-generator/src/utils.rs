// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    basic_block::BasicBlock,
    context::Context,
    module::Module,
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::{
        AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum, CallableValue, FunctionValue,
        InstructionValue, IntValue, PointerValue,
    },
};
use pyo3::{
    exceptions::{PyOSError, PyTypeError, PyUnicodeDecodeError, PyValueError},
    prelude::*,
    PyClass,
};
use std::{
    borrow::Borrow,
    convert::{Into, TryFrom},
};

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

#[derive(Clone, Copy)]
pub(crate) enum AnyValue<'ctx> {
    Any(AnyValueEnum<'ctx>),
    Instruction(InstructionValue<'ctx>),
    BasicBlock(BasicBlock<'ctx>),
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

impl<'ctx> From<BasicBlock<'ctx>> for AnyValue<'ctx> {
    fn from(block: BasicBlock<'ctx>) -> Self {
        Self::BasicBlock(block)
    }
}

impl<'ctx> From<InstructionValue<'ctx>> for AnyValue<'ctx> {
    fn from(instruction: InstructionValue<'ctx>) -> Self {
        Self::Instruction(instruction)
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for AnyValueEnum<'ctx> {
    type Error = ConversionError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(a) => Ok(a),
            _ => Err(ConversionError::new("value", "any value")),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for IntValue<'ctx> {
    type Error = ConversionError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::IntValue(i)) => Ok(i),
            _ => Err(ConversionError::new("value", "integer value")),
        }
    }
}

impl<'ctx> TryFrom<AnyValue<'ctx>> for PointerValue<'ctx> {
    type Error = ConversionError;

    fn try_from(value: AnyValue<'ctx>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::Any(AnyValueEnum::PointerValue(p)) => Ok(p),
            _ => Err(ConversionError::new("value", "pointer value")),
        }
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
        AnyValueEnum::PhiValue(_)
        | AnyValueEnum::FunctionValue(_)
        | AnyValueEnum::InstructionValue(_) => None,
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

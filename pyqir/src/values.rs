// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::used_underscore_binding)]

use crate::{
    core::Context,
    core::Message,
    instructions::Instruction,
    module::{Linkage, Module},
    types::{FunctionType, Type},
};
#[allow(clippy::wildcard_imports)]
use llvm_sys::{
    core::*, prelude::*, LLVMAttributeFunctionIndex, LLVMAttributeIndex, LLVMAttributeReturnIndex,
    LLVMTypeKind, LLVMValueKind,
};
use pyo3::{
    conversion::ToPyObject,
    exceptions::{PyKeyError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PyLong},
    PyRef,
};
use qirlib::values;
use std::{
    borrow::Borrow,
    convert::{Into, TryInto},
    ffi::CString,
    ops::Deref,
    ptr::NonNull,
    slice, str,
};

/// A value.
#[pyclass(subclass, unsendable)]
pub(crate) struct Value {
    value: LLVMValueRef,
    owner: Owner,
}

#[pymethods]
impl Value {
    /// The type of this value.
    ///
    /// :type: Type
    #[getter]
    fn r#type(&self, py: Python) -> PyResult<PyObject> {
        unsafe { Type::from_ptr(py, self.owner.context(py), LLVMTypeOf(self.value)) }
    }

    /// The name of this value or the empty string if this value is anonymous.
    #[getter]
    fn name(&self) -> &str {
        let mut len = 0;
        let name = unsafe {
            let name = LLVMGetValueName2(self.value, &mut len).cast();
            slice::from_raw_parts(name, len)
        };
        str::from_utf8(name).unwrap()
    }

    fn __str__(&self) -> String {
        unsafe {
            let message = Message::new(NonNull::new(LLVMPrintValueToString(self.value)).unwrap());
            message.to_str().unwrap().to_string()
        }
    }
}

impl Value {
    pub(crate) unsafe fn new(owner: Owner, value: LLVMValueRef) -> PyClassInitializer<Self> {
        PyClassInitializer::from(Self { value, owner })
    }

    pub(crate) unsafe fn from_ptr(
        py: Python,
        owner: Owner,
        value: LLVMValueRef,
    ) -> PyResult<PyObject> {
        match LLVMGetValueKind(value) {
            LLVMValueKind::LLVMInstructionValueKind => Instruction::from_ptr(py, owner, value),
            LLVMValueKind::LLVMBasicBlockValueKind => {
                let base = PyClassInitializer::from(Self { value, owner });
                let block = base.add_subclass(BasicBlock(LLVMValueAsBasicBlock(value)));
                Ok(Py::new(py, block)?.to_object(py))
            }
            _ if LLVMIsConstant(value) != 0 => Constant::from_ptr(py, owner, value),
            _ => Ok(Py::new(py, Self { value, owner })?.to_object(py)),
        }
    }

    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }
}

impl Deref for Value {
    type Target = LLVMValueRef;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// To store Inkwell values in Python classes, we transmute the lifetime to `'static`. You need to
/// be careful when using Inkwell types with unsafely extended lifetimes. Follow these rules:
///
/// 1. When storing in a data type, always include an `Owner` field containing the owning module, if
///    there is one, or the context otherwise.
/// 2. Before passing an LLVM object to an Inkwell function, call `Owner::merge` to ensure that the
///    owners of all of the objects are compatible.
pub(crate) enum Owner {
    Context(Py<Context>),
    Module(Py<Module>),
}

impl Owner {
    pub(crate) fn context(&self, py: Python) -> Py<Context> {
        match self {
            Self::Context(context) => context.clone_ref(py),
            Self::Module(module) => module.borrow(py).context().clone_ref(py),
        }
    }

    /// Merges a sequence of owners into a single owner that lives at least as long as every owner
    /// in the sequence.
    ///
    /// # Errors
    /// Fails if the the given owners use more than one distinct context or module.
    ///
    /// # Panics
    /// Panics if the sequence is empty.
    pub(crate) fn merge(
        py: Python,
        owners: impl IntoIterator<Item = impl Borrow<Self>>,
    ) -> PyResult<Self> {
        owners
            .into_iter()
            .try_fold(None, |o1, o2| match (o1, o2.borrow()) {
                (None, owner) => Ok(Some(owner.clone_ref(py))),
                (Some(Self::Context(c1)), Self::Context(c2))
                    if *c1.borrow(py) == *c2.borrow(py) =>
                {
                    Ok(Some(Self::Context(c1)))
                }
                (Some(Self::Context(c)), Self::Module(m))
                    if *c.borrow(py) == *m.borrow(py).context().borrow(py) =>
                {
                    Ok(Some(Self::Module(m.clone_ref(py))))
                }
                (Some(Self::Module(m)), Self::Context(c))
                    if *m.borrow(py).context().borrow(py) == *c.borrow(py) =>
                {
                    Ok(Some(Self::Module(m)))
                }
                (Some(Self::Module(m1)), Self::Module(m2)) if *m1.borrow(py) == *m2.borrow(py) => {
                    Ok(Some(Self::Module(m1)))
                }
                _ => Err(PyValueError::new_err(
                    "Some values are from different contexts or modules.",
                )),
            })
            .map(|o| o.expect("No owners were given."))
    }

    pub(crate) fn clone_ref(&self, py: Python) -> Owner {
        match self {
            Self::Context(context) => Self::Context(context.clone_ref(py)),
            Self::Module(module) => Self::Module(module.clone_ref(py)),
        }
    }
}

impl From<Py<Context>> for Owner {
    fn from(context: Py<Context>) -> Self {
        Self::Context(context)
    }
}

impl From<Py<Module>> for Owner {
    fn from(module: Py<Module>) -> Self {
        Self::Module(module)
    }
}

/// A basic block.
///
/// If the `before` block is given, this basic block is inserted directly before it. If no `before`
/// block is given, a `parent` function must be given, and this basic block is appended to the end
/// of that function.
///
/// :param Context context: The LLVM context.
/// :param str name: The block name.
/// :param typing.Optional[Function] parent: The parent function.
/// :param typing.Optional[BasicBlock] before: The block to insert this block before.
#[pyclass(extends = Value, unsendable)]
#[pyo3(text_signature = "(context, name, parent=None, before=None)")]
pub(crate) struct BasicBlock(LLVMBasicBlockRef);

#[pymethods]
impl BasicBlock {
    #[new]
    fn new(
        py: Python,
        context: Py<Context>,
        name: &str,
        parent: Option<PyRef<Function>>,
        before: Option<PyRef<BasicBlock>>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let parent = parent.map(|p| p.into_super().into_super());
        let owner = Owner::merge(
            py,
            [
                Some(&context.clone_ref(py).into()),
                parent.as_ref().map(|f| &f.owner),
                before.as_ref().map(|b| &b.as_ref().owner),
            ]
            .into_iter()
            .flatten(),
        )?;

        let block = {
            let context = context.borrow(py);
            let name = CString::new(name).unwrap();
            match (parent, before) {
                (None, None) => Err(PyValueError::new_err("Can't create block without parent.")),
                (Some(parent), None) => Ok(unsafe {
                    LLVMAppendBasicBlockInContext(context.as_ptr(), **parent, name.as_ptr())
                }),
                (Some(parent), Some(before))
                    if unsafe { LLVMGetBasicBlockParent(before.0) != **parent } =>
                {
                    Err(PyValueError::new_err(
                        "Insert before block isn't in parent function.",
                    ))
                }
                (_, Some(before)) => Ok(unsafe {
                    LLVMInsertBasicBlockInContext(context.as_ptr(), before.0, name.as_ptr())
                }),
            }
        }?;

        let value = unsafe { LLVMBasicBlockAsValue(block) };
        Ok(PyClassInitializer::from(Value { value, owner }).add_subclass(Self(block)))
    }

    /// The instructions in this basic block.
    ///
    /// :type: typing.List[Instruction]
    #[getter]
    fn instructions(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let mut insts = Vec::new();
        unsafe {
            let mut inst = LLVMGetFirstInstruction(slf.0);
            while !inst.is_null() {
                let owner = slf.as_ref().owner.clone_ref(py);
                insts.push(Instruction::from_ptr(py, owner, inst)?);
                inst = LLVMGetNextInstruction(inst);
            }
        }
        Ok(insts)
    }

    /// The terminating instruction of this basic block if there is one.
    ///
    /// :type: typing.Optional[Instruction]
    #[getter]
    fn terminator(slf: PyRef<Self>, py: Python) -> PyResult<Option<PyObject>> {
        unsafe {
            let term = LLVMGetBasicBlockTerminator(slf.0);
            if term.is_null() {
                Ok(None)
            } else {
                let owner = slf.into_super().owner.clone_ref(py);
                Instruction::from_ptr(py, owner, term).map(Some)
            }
        }
    }
}

impl Deref for BasicBlock {
    type Target = LLVMBasicBlockRef;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        unsafe { Value::from_ptr(py, ty.context().clone_ref(py).into(), LLVMConstNull(**ty)) }
    }

    /// Whether this value is the null value for its type.
    ///
    /// :type: bool
    #[getter]
    fn is_null(slf: PyRef<Self>) -> bool {
        unsafe { LLVMIsNull(slf.into_super().value) != 0 }
    }
}

impl Constant {
    unsafe fn from_ptr(py: Python, owner: Owner, value: LLVMValueRef) -> PyResult<PyObject> {
        if LLVMIsConstant(value) == 0 {
            Err(PyValueError::new_err("Value is not constant."))
        } else {
            let base = PyClassInitializer::from(Value { value, owner }).add_subclass(Constant);
            match LLVMGetValueKind(value) {
                LLVMValueKind::LLVMConstantIntValueKind => {
                    Ok(Py::new(py, base.add_subclass(IntConstant))?.to_object(py))
                }
                LLVMValueKind::LLVMConstantFPValueKind => {
                    Ok(Py::new(py, base.add_subclass(FloatConstant))?.to_object(py))
                }
                LLVMValueKind::LLVMFunctionValueKind => {
                    Ok(Py::new(py, base.add_subclass(Function))?.to_object(py))
                }
                _ => Ok(Py::new(py, base)?.to_object(py)),
            }
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
        unsafe { LLVMConstIntGetZExtValue(slf.into_super().into_super().value) }
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
        unsafe { LLVMConstRealGetDouble(slf.into_super().into_super().value, &mut 0) }
    }
}

/// A function value.
///
/// :param FunctionType ty: The function type.
/// :param Linkage linkage: The linkage kind.
/// :param str name: The function name.
/// :param Module module: The parent module.
#[pyclass(extends = Constant)]
pub(crate) struct Function;

#[pymethods]
impl Function {
    #[new]
    fn new(
        py: Python,
        ty: PyRef<FunctionType>,
        linkage: Linkage,
        name: &str,
        module: Py<Module>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let owner = Owner::merge(
            py,
            [
                Owner::Context(ty.as_ref().context().clone_ref(py)),
                Owner::Module(module.clone_ref(py)),
            ],
        )?;

        let name = CString::new(name).unwrap();
        unsafe {
            let function = LLVMAddFunction(**module.borrow(py), name.as_ptr(), **ty.into_super());
            LLVMSetLinkage(function, linkage.into());
            Ok(Value::new(owner, function)
                .add_subclass(Constant)
                .add_subclass(Self))
        }
    }

    /// The parameters to this function.
    ///
    /// :type: typing.List[Value]
    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let slf = slf.into_super().into_super();
        unsafe {
            let count = LLVMCountParams(slf.value).try_into().unwrap();
            let mut params = Vec::with_capacity(count);
            LLVMGetParams(slf.value, params.as_mut_ptr());
            params.set_len(count);
            params
                .into_iter()
                .map(|p| Value::from_ptr(py, slf.owner.clone_ref(py), p))
                .collect()
        }
    }

    /// The basic blocks in this function.
    ///
    /// :type: typing.List[BasicBlock]
    #[getter]
    fn basic_blocks(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let slf = slf.into_super().into_super();
        unsafe {
            let count = LLVMCountBasicBlocks(slf.value).try_into().unwrap();
            let mut blocks = Vec::with_capacity(count);
            LLVMGetBasicBlocks(slf.value, blocks.as_mut_ptr());
            blocks.set_len(count);
            blocks
                .into_iter()
                .map(|b| Value::from_ptr(py, slf.owner.clone_ref(py), LLVMBasicBlockAsValue(b)))
                .collect()
        }
    }

    /// The attributes for this function.
    #[getter]
    fn attributes(slf: Py<Function>) -> AttributeList {
        AttributeList(slf)
    }
}

/// An attribute.
#[pyclass(unsendable)]
pub(crate) struct Attribute(LLVMAttributeRef);

#[pymethods]
impl Attribute {
    /// The value of this attribute as a string, or `None` if this is not a string attribute.
    ///
    /// :type: typing.Optional[str]
    #[getter]
    fn string_value(&self) -> Option<&str> {
        unsafe {
            if LLVMIsStringAttribute(self.0) == 0 {
                None
            } else {
                let mut len = 0;
                let value = LLVMGetStringAttributeValue(self.0, &mut len).cast();
                let value = slice::from_raw_parts(value, len.try_into().unwrap());
                Some(str::from_utf8(value).unwrap())
            }
        }
    }
}

/// The attribute list for a function.
#[pyclass]
pub(crate) struct AttributeList(Py<Function>);

#[pymethods]
impl AttributeList {
    /// The attributes for a parameter.
    ///
    /// :param int n: The parameter number, starting from zero.
    /// :returns: The parameter attributes.
    /// :rtype: AttributeDict
    fn param(&self, py: Python, n: u32) -> AttributeSet {
        AttributeSet {
            function: self.0.clone_ref(py),
            index: n + 1,
        }
    }

    /// The attributes for the return type.
    ///
    /// :type: AttributeDict
    #[getter]
    fn ret(&self, py: Python) -> AttributeSet {
        AttributeSet {
            function: self.0.clone_ref(py),
            index: LLVMAttributeReturnIndex,
        }
    }

    /// The attributes for the function itself.
    ///
    /// :type: AttributeDict
    #[getter]
    fn func(&self, py: Python) -> AttributeSet {
        AttributeSet {
            function: self.0.clone_ref(py),
            index: LLVMAttributeFunctionIndex,
        }
    }
}

/// A set of attributes for a specific part of a function.
#[pyclass]
pub(crate) struct AttributeSet {
    function: Py<Function>,
    index: LLVMAttributeIndex,
}

#[pymethods]
impl AttributeSet {
    /// Tests if an attribute is a member of the set.
    ///
    /// :param str item: The attribute kind.
    /// :returns: True if the group has an attribute with the given kind.
    /// :rtype: bool
    fn __contains__(&self, py: Python, item: &str) -> bool {
        self.__getitem__(py, item).is_ok()
    }

    /// Gets an attribute based on its kind.
    ///
    /// :param str key: The attribute kind.
    /// :returns: The attribute.
    /// :rtype: Attribute
    fn __getitem__(&self, py: Python, key: &str) -> PyResult<Attribute> {
        let function = self.function.borrow(py).into_super().into_super();
        let kind = CString::new(key).unwrap();
        let attr = unsafe {
            LLVMGetStringAttributeAtIndex(
                function.value,
                self.index,
                kind.as_ptr(),
                key.len().try_into().unwrap(),
            )
        };

        if attr.is_null() {
            Err(PyKeyError::new_err(key.to_owned()))
        } else {
            Ok(Attribute(attr))
        }
    }
}

#[derive(FromPyObject)]
pub(crate) enum Literal<'py> {
    Bool(bool),
    Int(&'py PyLong),
    Float(f64),
}

impl Literal<'_> {
    pub(crate) unsafe fn to_value(&self, ty: LLVMTypeRef) -> PyResult<LLVMValueRef> {
        match (LLVMGetTypeKind(ty), self) {
            (LLVMTypeKind::LLVMIntegerTypeKind, &Self::Bool(b)) => {
                Ok(LLVMConstInt(ty, b.into(), 0))
            }
            (LLVMTypeKind::LLVMIntegerTypeKind, &Self::Int(i)) => {
                Ok(LLVMConstInt(ty, i.extract()?, 0))
            }
            (LLVMTypeKind::LLVMDoubleTypeKind, &Self::Float(f)) => Ok(LLVMConstReal(ty, f)),
            _ => Err(PyTypeError::new_err(
                "Can't convert Python value into this type.",
            )),
        }
    }
}

/// Creates a constant value.
///
/// :param Type ty: The type of the value.
/// :param typing.Union[bool, int, float] value: The value of the constant.
/// :returns: The constant value.
/// :rtype: Value
#[pyfunction]
#[pyo3(text_signature = "(ty, value)")]
pub(crate) fn r#const(py: Python, ty: &Type, value: Literal) -> PyResult<PyObject> {
    let owner = ty.context().clone_ref(py).into();
    unsafe { Value::from_ptr(py, owner, value.to_value(**ty)?) }
}

/// Creates a static qubit value.
///
/// :param Context context: The LLVM context.
/// :param int id: The static qubit ID.
/// :returns: A static qubit value.
/// :rtype: Value
#[pyfunction]
#[pyo3(text_signature = "(context, id)")]
pub(crate) fn qubit(py: Python, context: Py<Context>, id: u64) -> PyResult<PyObject> {
    let value = {
        let context = context.borrow(py);
        unsafe { values::qubit(context.as_ptr(), id) }
    };
    unsafe { Value::from_ptr(py, Owner::Context(context), value) }
}

/// If the value is a static qubit ID, extracts it.
///
/// :param Value value: The value.
/// :returns: The static qubit ID.
/// :rtype: typing.Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn qubit_id(value: &Value) -> Option<u64> {
    unsafe { values::qubit_id(**value) }
}

/// Creates a static result value.
///
/// :param Context context: The LLVM context.
/// :param int id: The static result ID.
/// :returns: A static result value.
/// :rtype: Value
#[pyfunction]
#[pyo3(text_signature = "(context, id)")]
pub(crate) fn result(py: Python, context: Py<Context>, id: u64) -> PyResult<PyObject> {
    let value = {
        let context = context.borrow(py);
        unsafe { values::result(context.as_ptr(), id) }
    };
    unsafe { Value::from_ptr(py, Owner::Context(context), value) }
}

/// If the value is a static result ID, extracts it.
///
/// :param Value value: The value.
/// :returns: The static result ID.
/// :rtype: typing.Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn result_id(value: &Value) -> Option<u64> {
    unsafe { values::result_id(**value) }
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
#[pyo3(text_signature = "(module, name, required_num_qubits, required_num_results)")]
pub(crate) fn entry_point(
    py: Python,
    module: Py<Module>,
    name: &str,
    required_num_qubits: u64,
    required_num_results: u64,
) -> PyResult<PyObject> {
    let name = CString::new(name).unwrap();
    let entry_point = unsafe {
        values::entry_point(
            **module.borrow(py),
            name.as_c_str(),
            required_num_qubits,
            required_num_results,
        )
    };
    unsafe { Value::from_ptr(py, Owner::Module(module), entry_point) }
}

/// Whether the function is an entry point.
///
/// :param Function function: The function.
/// :returns: True if the function is an entry point.
/// :rtype: bool
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn is_entry_point(function: PyRef<Function>) -> bool {
    unsafe { values::is_entry_point(function.into_super().into_super().value) }
}

/// Whether the function is interop-friendly.
///
/// :param Function function: The function.
/// :returns: True if the function is interop-friendly.
/// :rtype: bool
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn is_interop_friendly(function: PyRef<Function>) -> bool {
    unsafe { values::is_interop_friendly(function.into_super().into_super().value) }
}

/// If the function declares a required number of qubits, extracts it.
///
/// :param Function function: The function.
/// :returns: The required number of qubits.
/// :rtype: typing.Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn required_num_qubits(function: PyRef<Function>) -> Option<u64> {
    unsafe { values::required_num_qubits(function.into_super().into_super().value) }
}

/// If the function declares a required number of results, extracts it.
///
/// :param Function function: The function.
/// :returns: The required number of results.
/// :rtype: Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn required_num_results(function: PyRef<Function>) -> Option<u64> {
    unsafe { values::required_num_results(function.into_super().into_super().value) }
}

/// Creates a global null-terminated byte string constant in a module.
///
/// :param Module module: The parent module.
/// :param bytes value: The byte string value without a null terminator.
/// :returns: A pointer to the start of the null-terminated byte string.
/// :rtype: Constant
#[pyfunction]
#[pyo3(text_signature = "(module, value)")]
pub(crate) fn global_byte_string(py: Python, module: &Module, value: &[u8]) -> PyResult<PyObject> {
    let string = unsafe { values::global_string(**module, value) };
    unsafe { Value::from_ptr(py, module.context().clone_ref(py).into(), string) }
}

/// If the value is a pointer to a constant byte string, extracts it.
///
/// :param Value value: The value.
/// :returns: The constant byte string.
/// :rtype: typing.Optional[bytes]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn extract_byte_string<'py>(py: Python<'py>, value: &Value) -> Option<&'py PyBytes> {
    let string = unsafe { values::extract_string(**value)? };
    Some(PyBytes::new(py, &string))
}

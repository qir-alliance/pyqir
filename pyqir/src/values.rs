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
    LLVMBasicBlock, LLVMTypeKind, LLVMValue, LLVMValueKind,
};
use pyo3::{
    conversion::ToPyObject,
    exceptions::{PyKeyError, PyTypeError, PyValueError},
    prelude::*,
    pyclass::CompareOp,
    types::{PyBytes, PyLong, PyString},
    PyRef,
};
use qirlib::values::{self, get_string_attribute_kind, get_string_attribute_value};
use std::{
    borrow::Borrow,
    collections::hash_map::DefaultHasher,
    convert::{Into, TryInto},
    ffi::CString,
    hash::{Hash, Hasher},
    ops::Deref,
    ptr::NonNull,
    slice, str,
    vec::IntoIter,
};

/// A value.
#[pyclass(subclass, unsendable)]
pub(crate) struct Value {
    value: NonNull<LLVMValue>,
    owner: Owner,
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let x: LLVMValueRef = self.value.cast().as_ptr();
        x.hash(state);
    }
}

impl std::cmp::Eq for Value {}
impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.value.as_ptr() == other.value.cast().as_ptr()
    }
}

#[pymethods]
impl Value {
    // In order to implement the comparison operators, we have to do
    // it all in one impl of __richcmp__ for pyo3 to work.
    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
        match op {
            CompareOp::Eq => self.eq(other).into_py(py),
            CompareOp::Ne => (!self.eq(other)).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// The type of this value.
    ///
    /// :type: Type
    #[getter]
    fn r#type(&self, py: Python) -> PyResult<PyObject> {
        unsafe { Type::from_raw(py, self.owner.context(py), LLVMTypeOf(self.cast().as_ptr())) }
    }

    /// The name of this value or the empty string if this value is anonymous.
    #[getter]
    fn name(&self) -> &str {
        let mut len = 0;
        unsafe {
            let name = LLVMGetValueName2(self.cast().as_ptr(), &mut len).cast();
            str::from_utf8(slice::from_raw_parts(name, len)).unwrap()
        }
    }

    #[setter]
    fn set_name(&self, value: &str) {
        unsafe {
            let c_name = &CString::new(value).unwrap();
            LLVMSetValueName2(
                self.cast().as_ptr(),
                value.as_ptr().cast(),
                c_name.as_bytes().len(),
            );
        }
    }

    fn __str__(&self) -> String {
        unsafe {
            Message::from_raw(LLVMPrintValueToString(self.cast().as_ptr()))
                .to_str()
                .unwrap()
                .to_string()
        }
    }
}

impl Value {
    pub(crate) unsafe fn new(owner: Owner, value: NonNull<LLVMValue>) -> Self {
        Self { value, owner }
    }

    pub(crate) unsafe fn from_raw(
        py: Python,
        owner: Owner,
        value: LLVMValueRef,
    ) -> PyResult<PyObject> {
        match LLVMGetValueKind(value) {
            LLVMValueKind::LLVMInstructionValueKind => Instruction::from_raw(py, owner, value),
            LLVMValueKind::LLVMBasicBlockValueKind => {
                let block = BasicBlock::from_raw(owner, LLVMValueAsBasicBlock(value));
                Ok(Py::new(py, block)?.to_object(py))
            }
            _ if LLVMIsConstant(value) != 0 => Constant::from_raw(py, owner, value),
            _ => {
                let value = NonNull::new(value).expect("Value is null.");
                Ok(Py::new(py, Self { value, owner })?.to_object(py))
            }
        }
    }

    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }
}

impl Deref for Value {
    type Target = NonNull<LLVMValue>;

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
pub(crate) struct BasicBlock(NonNull<LLVMBasicBlock>);

#[pymethods]
impl BasicBlock {
    #[new]
    #[pyo3(text_signature = "(context, name, parent=None, before=None)")]
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
                    LLVMAppendBasicBlockInContext(
                        context.cast().as_ptr(),
                        parent.cast().as_ptr(),
                        name.as_ptr(),
                    )
                }),
                (Some(parent), Some(before))
                    if unsafe {
                        LLVMGetBasicBlockParent(before.cast().as_ptr()) != parent.cast().as_ptr()
                    } =>
                {
                    Err(PyValueError::new_err(
                        "Insert before block isn't in parent function.",
                    ))
                }
                (_, Some(before)) => Ok(unsafe {
                    LLVMInsertBasicBlockInContext(
                        context.cast().as_ptr(),
                        before.cast().as_ptr(),
                        name.as_ptr(),
                    )
                }),
            }
        }?;

        Ok(PyClassInitializer::from(Value {
            value: NonNull::new(unsafe { LLVMBasicBlockAsValue(block) }).unwrap(),
            owner,
        })
        .add_subclass(Self(NonNull::new(block).unwrap())))
    }

    /// The instructions in this basic block.
    ///
    /// :type: typing.List[Instruction]
    #[getter]
    fn instructions(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let mut insts = Vec::new();
        unsafe {
            let mut inst = LLVMGetFirstInstruction(slf.cast().as_ptr());
            while !inst.is_null() {
                let owner = slf.as_ref().owner.clone_ref(py);
                insts.push(Instruction::from_raw(py, owner, inst)?);
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
            let term = LLVMGetBasicBlockTerminator(slf.cast().as_ptr());
            if term.is_null() {
                Ok(None)
            } else {
                let owner = slf.into_super().owner.clone_ref(py);
                Instruction::from_raw(py, owner, term).map(Some)
            }
        }
    }
}

impl BasicBlock {
    pub(crate) unsafe fn from_raw(
        owner: Owner,
        block: LLVMBasicBlockRef,
    ) -> PyClassInitializer<Self> {
        let block = NonNull::new(block).expect("Block is null.");
        let value =
            NonNull::new(LLVMBasicBlockAsValue(block.cast().as_ptr())).expect("Value is null.");
        PyClassInitializer::from(Value { value, owner }).add_subclass(BasicBlock(block))
    }
}

impl Deref for BasicBlock {
    type Target = NonNull<LLVMBasicBlock>;

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
        let context = ty.context().clone_ref(py);
        unsafe { Value::from_raw(py, context.into(), LLVMConstNull(ty.cast().as_ptr())) }
    }

    /// Whether this value is the null value for its type.
    ///
    /// :type: bool
    #[getter]
    fn is_null(slf: PyRef<Self>) -> bool {
        unsafe { LLVMIsNull(slf.into_super().cast().as_ptr()) != 0 }
    }
}

impl Constant {
    pub(crate) unsafe fn from_raw(
        py: Python,
        owner: Owner,
        value: LLVMValueRef,
    ) -> PyResult<PyObject> {
        let value = NonNull::new(value).expect("Value is null.");
        if LLVMIsConstant(value.cast().as_ptr()) == 0 {
            Err(PyValueError::new_err("Value is not constant."))
        } else {
            let base = PyClassInitializer::from(Value { value, owner }).add_subclass(Constant);
            match LLVMGetValueKind(value.cast().as_ptr()) {
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
        unsafe { LLVMConstIntGetZExtValue(slf.into_super().into_super().cast().as_ptr()) }
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
        unsafe { LLVMConstRealGetDouble(slf.into_super().into_super().cast().as_ptr(), &mut 0) }
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
            let value = NonNull::new(LLVMAddFunction(
                module.borrow(py).cast().as_ptr(),
                name.as_ptr(),
                ty.into_super().cast().as_ptr(),
            ))
            .expect("Function is null.");
            LLVMSetLinkage(value.cast().as_ptr(), linkage.into());

            Ok(PyClassInitializer::from(Value { value, owner })
                .add_subclass(Constant)
                .add_subclass(Self))
        }
    }

    #[getter]
    fn r#type(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let slf = slf.into_super().into_super();
        unsafe {
            let ty = LLVMGetElementType(LLVMTypeOf(slf.cast().as_ptr()));
            Type::from_raw(py, slf.owner().context(py), ty)
        }
    }

    /// The parameters to this function.
    ///
    /// :type: typing.List[Value]
    #[getter]
    fn params(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let slf = slf.into_super().into_super();
        unsafe {
            let count = LLVMCountParams(slf.cast().as_ptr()).try_into().unwrap();
            let mut params = Vec::with_capacity(count);
            LLVMGetParams(slf.cast().as_ptr(), params.as_mut_ptr());
            params.set_len(count);
            params
                .into_iter()
                .map(|p| Value::from_raw(py, slf.owner.clone_ref(py), p))
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
            let count = LLVMCountBasicBlocks(slf.cast().as_ptr())
                .try_into()
                .unwrap();
            let mut blocks = Vec::with_capacity(count);
            LLVMGetBasicBlocks(slf.cast().as_ptr(), blocks.as_mut_ptr());
            blocks.set_len(count);
            blocks
                .into_iter()
                .map(|b| Value::from_raw(py, slf.owner.clone_ref(py), LLVMBasicBlockAsValue(b)))
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
    /// The id of this attribute as a string.
    ///
    /// :type: str
    #[getter]
    fn string_kind(&self) -> String {
        unsafe { get_string_attribute_kind(self.0) }
    }

    /// The value of this attribute as a string, or `None` if this is not a string attribute.
    ///
    /// :type: typing.Optional[str]
    #[getter]
    fn string_value(&self) -> Option<String> {
        unsafe { get_string_attribute_value(self.0) }
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
            index: n + 1, // Parameter indices start from one.
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

/// An iterator of attributes for a specific part of a function.
#[pyclass]
struct AttributeIterator {
    iter: IntoIter<Py<Attribute>>,
}

#[pymethods]
impl AttributeIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    // Returning `None` from `__next__` indicates that that there are no further items.
    // and maps to StopIteration
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<Attribute>> {
        slf.iter.next()
    }
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
                function.cast().as_ptr(),
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

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<AttributeIterator>> {
        let function = slf.function.borrow(slf.py()).into_super().into_super();

        unsafe {
            let attrs = qirlib::values::get_attributes(function.cast().as_ptr(), slf.index);
            let items = attrs
                .into_iter()
                .map(|a| Py::new(slf.py(), Attribute(a)).expect("msg"));
            Py::new(
                slf.py(),
                AttributeIterator {
                    iter: items.collect::<Vec<Py<Attribute>>>().into_iter(),
                },
            )
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
    unsafe { Value::from_raw(py, owner, value.to_value(ty.cast().as_ptr())?) }
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
    unsafe {
        let value = values::qubit(context.borrow(py).cast().as_ptr(), id);
        Value::from_raw(py, context.into(), value)
    }
}

/// If the value is a static qubit ID, extracts it.
///
/// :param Value value: The value.
/// :returns: The static qubit ID.
/// :rtype: typing.Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn qubit_id(value: &Value) -> Option<u64> {
    unsafe { values::qubit_id(value.cast().as_ptr()) }
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
    unsafe {
        let value = values::result(context.borrow(py).cast().as_ptr(), id);
        Value::from_raw(py, context.into(), value)
    }
}

/// If the value is a static result ID, extracts it.
///
/// :param Value value: The value.
/// :returns: The static result ID.
/// :rtype: typing.Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn result_id(value: &Value) -> Option<u64> {
    unsafe { values::result_id(value.cast().as_ptr()) }
}

/// Whether the function is an entry point.
///
/// :param Function function: The function.
/// :returns: True if the function is an entry point.
/// :rtype: bool
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn is_entry_point(function: PyRef<Function>) -> bool {
    unsafe { values::is_entry_point(function.into_super().into_super().cast().as_ptr()) }
}

/// Whether the function is interop-friendly.
///
/// :param Function function: The function.
/// :returns: True if the function is interop-friendly.
/// :rtype: bool
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn is_interop_friendly(function: PyRef<Function>) -> bool {
    unsafe { values::is_interop_friendly(function.into_super().into_super().cast().as_ptr()) }
}

/// If the function declares a required number of qubits, extracts it.
///
/// :param Function function: The function.
/// :returns: The required number of qubits.
/// :rtype: typing.Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn required_num_qubits(function: PyRef<Function>) -> Option<u64> {
    unsafe { values::required_num_qubits(function.into_super().into_super().cast().as_ptr()) }
}

/// If the function declares a required number of results, extracts it.
///
/// :param Function function: The function.
/// :returns: The required number of results.
/// :rtype: Optional[int]
#[pyfunction]
#[pyo3(text_signature = "(function)")]
pub(crate) fn required_num_results(function: PyRef<Function>) -> Option<u64> {
    unsafe { values::required_num_results(function.into_super().into_super().cast().as_ptr()) }
}

/// Creates a module with required QIR module flag metadata
///
/// :param Context context: The parent context.
/// :param str name: The module name.
/// :param int qir_major_version: The QIR major version this module is built for. Default 1.
/// :param int qir_minor_version: The QIR minor version this module is built for. Default 0.
/// :param bool dynamic_qubit_management: Whether this module supports dynamic qubit management. Default False.
/// :param bool dynamic_result_management: Whether this module supports dynamic result management. Default False.
/// :rtype: Module
#[pyfunction]
#[pyo3(
    text_signature = "(context, name, qir_major_version, qir_minor_version, dynamic_qubit_management, dynamic_result_management)"
)]
pub(crate) fn qir_module(
    py: Python,
    context: Py<Context>,
    name: &str,
    qir_major_version: Option<i32>,
    qir_minor_version: Option<i32>,
    dynamic_qubit_management: Option<bool>,
    dynamic_result_management: Option<bool>,
) -> PyResult<PyObject> {
    let module = crate::module::Module::new(py, context, name);
    let ptr = module.cast().as_ptr();
    unsafe {
        qirlib::module::set_qir_major_version(ptr, qir_major_version.unwrap_or(1));
    }
    unsafe {
        qirlib::module::set_qir_minor_version(ptr, qir_minor_version.unwrap_or(0));
    }
    unsafe {
        qirlib::module::set_dynamic_qubit_management(
            ptr,
            dynamic_qubit_management.unwrap_or(false),
        );
    }
    unsafe {
        qirlib::module::set_dynamic_result_management(
            ptr,
            dynamic_result_management.unwrap_or(false),
        );
    }
    Ok(Py::new(py, module)?.to_object(py))
}

/// The QIR major version this module is built for. None if unspecified.
#[pyfunction]
#[pyo3(text_signature = "(module)")]
pub(crate) fn qir_major_version(module: PyRef<Module>) -> Option<i32> {
    unsafe { qirlib::module::qir_major_version(module.cast().as_ptr()) }
}

/// The QIR minor version this module is built for. None if unspecified.
#[pyfunction]
#[pyo3(text_signature = "(module)")]
pub(crate) fn qir_minor_version(module: PyRef<Module>) -> Option<i32> {
    unsafe { qirlib::module::qir_minor_version(module.cast().as_ptr()) }
}

/// Whether this module supports dynamic qubit management. None if unspecified.
#[pyfunction]
#[pyo3(text_signature = "(module)")]
pub(crate) fn dynamic_qubit_management(module: PyRef<Module>) -> Option<bool> {
    unsafe { qirlib::module::dynamic_qubit_management(module.cast().as_ptr()) }
}

/// Whether this module supports dynamic result management. None if unspecified.
#[pyfunction]
#[pyo3(text_signature = "(module)")]
pub(crate) fn dynamic_result_management(module: PyRef<Module>) -> Option<bool> {
    unsafe { qirlib::module::dynamic_result_management(module.cast().as_ptr()) }
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
    let context = module.context().clone_ref(py);
    unsafe {
        let string = values::global_string(module.cast().as_ptr(), value);
        Value::from_raw(py, context.into(), string)
    }
}

/// If the value is a pointer to a constant byte string, extracts it.
///
/// :param Value value: The value.
/// :returns: The constant byte string.
/// :rtype: typing.Optional[bytes]
#[pyfunction]
#[pyo3(text_signature = "(value)")]
pub(crate) fn extract_byte_string<'py>(py: Python<'py>, value: &Value) -> Option<&'py PyBytes> {
    let string = unsafe { values::extract_string(value.cast().as_ptr())? };
    Some(PyBytes::new(py, &string))
}

// Adds a string attribute to the given function.

// :param function: The function.
// :param kind: The attribute kind.
// :param value: The attribute value.
// :param index: The optional attribute index, defaults to the function index.
#[pyfunction]
#[pyo3(text_signature = "(function, key, value, index)")]
pub(crate) fn add_string_attribute<'py>(
    function: PyRef<Function>,
    key: &'py PyString,
    value: Option<&'py PyString>,
    index: Option<u32>,
) {
    let function = function.into_super().into_super().cast().as_ptr();
    let key = key.to_string_lossy();
    let value = value.map(PyString::to_string_lossy);
    unsafe {
        values::add_string_attribute(
            function,
            key.as_bytes(),
            match value {
                Some(ref x) => x.as_bytes(),
                None => &[],
            },
            index.unwrap_or(LLVMAttributeFunctionIndex),
        );
    }
}

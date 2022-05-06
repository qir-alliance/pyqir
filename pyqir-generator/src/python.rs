// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pyo3::{
    basic::CompareOp,
    exceptions::{PyOSError, PyOverflowError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PySequence, PyString, PyUnicode},
    PyObjectProtocol,
};
use qirlib::generation::{
    emit,
    interop::{
        Call, ClassicalRegister, Controlled, FunctionType, If, Instruction, IntegerValue, Measured,
        QuantumRegister, ReturnType, Rotated, SemanticModel, Single, Value, ValueType,
    },
};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    vec,
};

#[pyfunction]
fn ir_to_bitcode<'a>(py: Python<'a>, value: &PyString, name: &PyString) -> PyResult<&'a PyBytes> {
    let bitcode = qirlib::generation::ir_to_bitcode(value.to_str()?, name.to_str()?)
        .map_err(PyOSError::new_err)?;
    Ok(PyBytes::new(py, &bitcode))
}

#[pyfunction]
fn bitcode_to_ir<'a>(py: Python<'a>, value: &PyBytes, name: &PyString) -> PyResult<&'a PyString> {
    let ir = qirlib::generation::bitcode_to_ir(value.as_bytes(), name.to_str()?)
        .map_err(PyOSError::new_err)?;
    Ok(PyUnicode::new(py, ir.as_str()))
}

#[pymodule]
#[pyo3(name = "_native")]
fn native_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Qubit>()?;
    m.add_class::<ResultRef>()?;
    m.add_class::<Function>()?;
    m.add_class::<Builder>()?;
    m.add_class::<SimpleModule>()?;
    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;
    m.add_class::<BasicQisBuilder>()
}

const TYPES_MODULE_NAME: &str = "pyqir.generator.types";
const RESULT_NAME: &str = "result";
const QUBIT_NAME: &str = "qubit";

struct PyVoidType;

impl<'source> FromPyObject<'source> for PyVoidType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        extract_sentinel(TYPES_MODULE_NAME, "Void", ob).map(|()| PyVoidType)
    }
}

#[derive(FromPyObject)]
struct PyIntegerType {
    width: u32,
}

struct PyDoubleType;

impl<'source> FromPyObject<'source> for PyDoubleType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        extract_sentinel(TYPES_MODULE_NAME, "Double", ob).map(|()| PyDoubleType)
    }
}

struct PyQubitType;

impl<'source> FromPyObject<'source> for PyQubitType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        extract_sentinel(TYPES_MODULE_NAME, "Qubit", ob).map(|()| PyQubitType)
    }
}

struct PyResultType;

impl<'source> FromPyObject<'source> for PyResultType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        extract_sentinel(TYPES_MODULE_NAME, "Result", ob).map(|()| PyResultType)
    }
}

fn extract_sentinel(module_name: &str, type_name: &str, ob: &PyAny) -> PyResult<()> {
    let module: &str = ob.get_type().getattr("__module__")?.extract()?;

    if module == module_name && ob.get_type().name()? == type_name {
        Ok(())
    } else {
        let message = format!("Expected {}.{}.", module_name, type_name);
        Err(PyErr::new::<PyTypeError, _>(message))
    }
}

#[derive(FromPyObject)]
enum PyValueType {
    Integer(PyIntegerType),
    Double(PyDoubleType),
    Qubit(PyQubitType),
    Result(PyResultType),
}

impl From<PyValueType> for ValueType {
    fn from(ty: PyValueType) -> Self {
        match ty {
            PyValueType::Integer(PyIntegerType { width }) => ValueType::Integer { width },
            PyValueType::Double(PyDoubleType) => ValueType::Double,
            PyValueType::Qubit(PyQubitType) => ValueType::Qubit,
            PyValueType::Result(PyResultType) => ValueType::Result,
        }
    }
}

#[derive(FromPyObject)]
enum PyReturnType {
    Void(PyVoidType),
    Value(PyValueType),
}

impl From<PyReturnType> for ReturnType {
    fn from(ty: PyReturnType) -> Self {
        match ty {
            PyReturnType::Void(PyVoidType) => ReturnType::Void,
            PyReturnType::Value(value_type) => ReturnType::Value(value_type.into()),
        }
    }
}

#[derive(FromPyObject)]
struct PyFunctionType {
    param_types: Vec<PyValueType>,
    return_type: PyReturnType,
}

impl From<PyFunctionType> for FunctionType {
    fn from(ty: PyFunctionType) -> Self {
        FunctionType {
            param_types: ty.param_types.into_iter().map(Into::into).collect(),
            return_type: ty.return_type.into(),
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
#[pyclass]
struct Qubit {
    index: u64,
}

impl Qubit {
    fn id(&self) -> String {
        format!("{}{}", QUBIT_NAME, self.index)
    }
}

#[pyproto]
impl PyObjectProtocol for Qubit {
    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!("<Qubit {}>", self.index)
    }

    fn __richcmp__(&self, other: Qubit, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self == &other),
            _ => Err(PyErr::new::<PyTypeError, _>("Only equality is supported.")),
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
#[pyclass]
struct ResultRef {
    index: u64,
}

impl ResultRef {
    fn id(&self) -> String {
        format!("{}{}", RESULT_NAME, self.index)
    }
}

#[pyproto]
impl PyObjectProtocol for ResultRef {
    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!("<ResultRef {}>", self.index)
    }

    fn __richcmp__(&self, other: ResultRef, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self == &other),
            _ => Err(PyErr::new::<PyTypeError, _>("Only equality is supported.")),
        }
    }
}

#[derive(Clone)]
#[pyclass]
struct Function {
    name: String,
}

#[pyclass]
struct Builder {
    frames: Vec<Vec<Instruction>>,
    external_functions: HashMap<String, FunctionType>,
}

#[pymethods]
impl Builder {
    #[new]
    fn new() -> Builder {
        Builder {
            frames: vec![vec![]],
            external_functions: HashMap::new(),
        }
    }

    fn call(&mut self, function: Function, args: &PySequence) -> PyResult<()> {
        let name = function.name;
        let ty = self.external_functions.get(&name).unwrap();
        let num_params = ty.param_types.len();
        let num_args = args.len()?;

        let typed_args = if num_args == num_params {
            args.iter()?.zip(&ty.param_types)
        } else {
            let message = format!("Expected {} arguments, got {}.", num_params, num_args);
            return Err(PyErr::new::<PyValueError, _>(message));
        };

        let args = typed_args
            .map(|(arg, &ty)| extract_value(arg?, ty))
            .collect::<PyResult<_>>()?;

        self.push_inst(Instruction::Call(Call { name, args }));
        Ok(())
    }
}

impl Builder {
    fn push_inst(&mut self, inst: Instruction) {
        self.frames.last_mut().unwrap().push(inst);
    }

    fn push_frame(&mut self) {
        self.frames.push(vec![]);
    }

    fn pop_frame(&mut self) -> Option<Vec<Instruction>> {
        self.frames.pop()
    }
}

#[pyclass]
struct SimpleModule {
    model: SemanticModel,
    builder: Py<Builder>,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(py: Python, name: String, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let registers = vec![ClassicalRegister::new(RESULT_NAME.to_string(), num_results)];

        let qubits = (0..num_qubits)
            .map(|i| QuantumRegister::new(QUBIT_NAME.to_string(), i))
            .collect();

        let model = SemanticModel {
            name,
            external_functions: HashMap::new(),
            registers,
            qubits,
            instructions: Vec::new(),
            use_static_qubit_alloc: true,
            use_static_result_alloc: true,
        };

        let builder = Py::new(py, Builder::new())?;
        Ok(SimpleModule { model, builder })
    }

    #[getter]
    fn qubits(&self) -> Vec<Qubit> {
        self.model
            .qubits
            .iter()
            .map(|q| Qubit { index: q.index })
            .collect()
    }

    #[getter]
    fn results(&self) -> Vec<ResultRef> {
        let size = self.model.registers.first().unwrap().size;
        (0..size).map(|index| ResultRef { index }).collect()
    }

    #[getter]
    fn builder(&self) -> Py<Builder> {
        self.builder.clone()
    }

    fn ir(&self, py: Python) -> PyResult<String> {
        let model = self.model_from_builder(py);
        emit::ir(&model).map_err(PyOSError::new_err)
    }

    fn bitcode<'a>(&self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        let model = self.model_from_builder(py);
        match emit::bitcode(&model) {
            Ok(bitcode) => Ok(PyBytes::new(py, &bitcode[..])),
            Err(err) => Err(PyOSError::new_err(err)),
        }
    }

    fn add_external_function(&mut self, py: Python, name: String, ty: PyFunctionType) -> Function {
        let mut builder = self.builder.as_ref(py).borrow_mut();
        builder.external_functions.insert(name.clone(), ty.into());
        Function { name }
    }

    fn use_static_qubit_alloc(&mut self, value: bool) {
        self.model.use_static_qubit_alloc = value;
    }

    fn use_static_result_alloc(&mut self, value: bool) {
        self.model.use_static_result_alloc = value;
    }
}

impl SimpleModule {
    fn model_from_builder(&self, py: Python) -> SemanticModel {
        let builder = self.builder.as_ref(py).borrow();

        match builder.frames[..] {
            [ref instructions] => SemanticModel {
                instructions: instructions.clone(),
                external_functions: builder.external_functions.clone(),
                ..self.model.clone()
            },
            _ => panic!("Builder does not contain exactly one stack frame."),
        }
    }
}

#[pyclass]
struct BasicQisBuilder {
    builder: Py<Builder>,
}

#[pymethods]
impl BasicQisBuilder {
    #[new]
    fn new(builder: Py<Builder>) -> Self {
        BasicQisBuilder { builder }
    }

    fn cx(&self, py: Python, control: &Qubit, target: &Qubit) {
        let controlled = Controlled::new(control.id(), target.id());
        self.push_inst(py, Instruction::Cx(controlled));
    }

    fn cz(&self, py: Python, control: &Qubit, target: &Qubit) {
        let controlled = Controlled::new(control.id(), target.id());
        self.push_inst(py, Instruction::Cz(controlled));
    }

    fn h(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::H(single));
    }

    fn m(&self, py: Python, qubit: &Qubit, result: &ResultRef) {
        let measured = Measured::new(qubit.id(), result.id());
        self.push_inst(py, Instruction::M(measured));
    }

    fn reset(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::Reset(single));
    }

    fn rx(&self, py: Python, theta: f64, qubit: &Qubit) {
        let rotated = Rotated::new(theta, qubit.id());
        self.push_inst(py, Instruction::Rx(rotated));
    }

    fn ry(&self, py: Python, theta: f64, qubit: &Qubit) {
        let rotated = Rotated::new(theta, qubit.id());
        self.push_inst(py, Instruction::Ry(rotated));
    }

    fn rz(&self, py: Python, theta: f64, qubit: &Qubit) {
        let rotated = Rotated::new(theta, qubit.id());
        self.push_inst(py, Instruction::Rz(rotated));
    }

    fn s(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::S(single));
    }

    fn s_adj(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::SAdj(single));
    }

    fn t(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::T(single));
    }

    fn t_adj(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::TAdj(single));
    }

    fn x(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::X(single));
    }

    fn y(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::Y(single));
    }

    fn z(&self, py: Python, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.push_inst(py, Instruction::Z(single));
    }

    fn if_result(
        &self,
        py: Python,
        result: &ResultRef,
        one: Option<&PyAny>,
        zero: Option<&PyAny>,
    ) -> PyResult<()> {
        let build_frame = |callback: Option<&PyAny>| -> PyResult<_> {
            self.push_frame(py);
            if let Some(callback) = callback {
                callback.call0()?;
            }

            Ok(self.pop_frame(py).unwrap())
        };

        let if_inst = If {
            condition: result.id(),
            then_insts: build_frame(one)?,
            else_insts: build_frame(zero)?,
        };

        self.push_inst(py, Instruction::If(if_inst));
        Ok(())
    }
}

impl BasicQisBuilder {
    fn push_inst(&self, py: Python, inst: Instruction) {
        let mut builder = self.builder.as_ref(py).borrow_mut();
        builder.push_inst(inst);
    }

    fn push_frame(&self, py: Python) {
        let mut builder = self.builder.as_ref(py).borrow_mut();
        builder.push_frame();
    }

    fn pop_frame(&self, py: Python) -> Option<Vec<Instruction>> {
        let mut builder = self.builder.as_ref(py).borrow_mut();
        builder.pop_frame()
    }
}

fn extract_value(ob: &PyAny, ty: ValueType) -> PyResult<Value> {
    match ty {
        ValueType::Integer { width } => IntegerValue::new(width, ob.extract()?)
            .map(Value::Integer)
            .ok_or_else(|| {
                let message = format!("Value too big for {}-bit integer.", width);
                PyErr::new::<PyOverflowError, _>(message)
            }),
        ValueType::Double => Ok(Value::Double(ob.extract()?)),
        ValueType::Qubit => Ok(Value::Qubit(ob.extract::<Qubit>()?.id())),
        ValueType::Result => Ok(Value::Result(ob.extract::<ResultRef>()?.id())),
    }
}

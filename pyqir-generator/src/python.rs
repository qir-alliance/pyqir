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
        self, Call, ClassicalRegister, Controlled, FunctionType, If, Instruction, Integer,
        Measured, QuantumRegister, ReturnType, Rotated, SemanticModel, Single, ValueType, Variable,
    },
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    vec,
};

#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn ir_to_bitcode<'a>(
    py: Python<'a>,
    value: &str,
    module_name: Option<String>,
    source_file_name: Option<String>,
) -> PyResult<&'a PyBytes> {
    let bitcode = qirlib::generation::ir_to_bitcode(value, &module_name, &source_file_name)
        .map_err(PyOSError::new_err)?;
    Ok(PyBytes::new(py, &bitcode))
}

#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn bitcode_to_ir<'a>(
    py: Python<'a>,
    value: &PyBytes,
    module_name: Option<String>,
    source_file_name: Option<String>,
) -> PyResult<&'a PyString> {
    let ir = qirlib::generation::bitcode_to_ir(value.as_bytes(), &module_name, &source_file_name)
        .map_err(PyOSError::new_err)?;
    Ok(PyUnicode::new(py, ir.as_str()))
}

#[pymodule]
#[pyo3(name = "_native")]
fn native_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ResultRef>()?;
    m.add_class::<Function>()?;
    m.add_class::<Builder>()?;
    m.add_class::<Value>()?;
    m.add_class::<SimpleModule>()?;
    m.add_class::<BasicQisBuilder>()?;

    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;

    Ok(())
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
    ty: FunctionType,
}

#[derive(Clone)]
#[pyclass]
struct Value(interop::Value);

#[pyproto]
impl PyObjectProtocol for Value {
    fn __repr__(&self) -> String {
        match &self.0 {
            interop::Value::Integer(int) => {
                format!(
                    "Value.integer(types.Integer({}), {})",
                    int.width(),
                    int.value()
                )
            }
            interop::Value::Double(value) => format!("Value.double({})", value),
            interop::Value::Qubit(name) | interop::Value::Result(name) => {
                format!("<Value {}>", name)
            }
            interop::Value::Variable(var) => format!("<Value {:?}>", var),
        }
    }
}

#[pymethods]
impl Value {
    #[staticmethod]
    fn integer(ty: PyIntegerType, value: u64) -> PyResult<Value> {
        let integer =
            interop::Integer::new(ty.width, value).ok_or(PyErr::new::<PyOverflowError, _>(
                "Value is too large for the type.",
            ))?;
        Ok(Value(interop::Value::Integer(integer)))
    }

    #[staticmethod]
    fn double(value: f64) -> Value {
        Value(interop::Value::Double(value))
    }
}

#[pyclass]
struct Builder {
    frames: Vec<Vec<Instruction>>,
    external_functions: Vec<Function>,
    next_variable: Variable,
}

#[pymethods]
impl Builder {
    #[new]
    fn new() -> Builder {
        Builder {
            frames: vec![vec![]],
            external_functions: vec![],
            next_variable: Variable::default(),
        }
    }

    fn call(&mut self, function: Function, args: &PySequence) -> PyResult<Option<Value>> {
        let num_params = function.ty.param_types.len();
        let num_args = args.len()?;
        if num_params != num_args {
            let message = format!("Expected {} arguments, got {}.", num_params, num_args);
            return Err(PyErr::new::<PyValueError, _>(message));
        }

        let args = args
            .iter()?
            .zip(&function.ty.param_types)
            .map(|(arg, &ty)| extract_value(arg?, ty))
            .collect::<PyResult<_>>()?;

        let result = match function.ty.return_type {
            ReturnType::Void => None,
            ReturnType::Value(_) => Some(self.fresh_variable()),
        };

        self.push_inst(Instruction::Call(Call {
            name: function.name,
            args,
            result,
        }));

        Ok(result.map(|v| Value(interop::Value::Variable(v))))
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

    fn fresh_variable(&mut self) -> Variable {
        let v = self.next_variable;
        self.next_variable = v.next();
        v
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
            external_functions: vec![],
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
    fn qubits(&self) -> Vec<Value> {
        self.model
            .qubits
            .iter()
            .map(|q| Value(interop::Value::Qubit(format!("qubit{}", q.index))))
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
        let ty = ty.into();
        let function = Function { name, ty };
        builder.external_functions.push(function.clone());
        function
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
        let external_functions = builder
            .external_functions
            .iter()
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect();

        match builder.frames[..] {
            [ref instructions] => SemanticModel {
                instructions: instructions.clone(),
                external_functions,
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

    fn cx(&self, py: Python, control: Value, target: Value) {
        let controlled = Controlled::new(control.0, target.0);
        self.push_inst(py, Instruction::Cx(controlled));
    }

    fn cz(&self, py: Python, control: Value, target: Value) {
        let controlled = Controlled::new(control.0, target.0);
        self.push_inst(py, Instruction::Cz(controlled));
    }

    fn h(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::H(single));
    }

    fn m(&self, py: Python, qubit: Value, result: &ResultRef) {
        let measured = Measured::new(qubit.0, result.id());
        self.push_inst(py, Instruction::M(measured));
    }

    fn reset(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::Reset(single));
    }

    fn rx(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(theta, ValueType::Double)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Rx(rotated));
        Ok(())
    }

    fn ry(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(theta, ValueType::Double)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Ry(rotated));
        Ok(())
    }

    fn rz(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(theta, ValueType::Double)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Rz(rotated));
        Ok(())
    }

    fn s(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::S(single));
    }

    fn s_adj(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::SAdj(single));
    }

    fn t(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::T(single));
    }

    fn t_adj(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::TAdj(single));
    }

    fn x(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::X(single));
    }

    fn y(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::Y(single));
    }

    fn z(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
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

fn extract_value(ob: &PyAny, ty: ValueType) -> PyResult<interop::Value> {
    match ob.extract::<Value>() {
        Ok(value) => Ok(value.0),
        Err(_) => match ty {
            ValueType::Integer { width } => Integer::new(width, ob.extract()?)
                .map(interop::Value::Integer)
                .ok_or_else(|| {
                    let message = format!("Value too big for {}-bit integer.", width);
                    PyErr::new::<PyOverflowError, _>(message)
                }),
            ValueType::Double => Ok(interop::Value::Double(ob.extract()?)),
            ValueType::Qubit => Err(PyErr::new::<PyTypeError, _>("Expected Qubit value.")),
            ValueType::Result => Ok(interop::Value::Result(ob.extract::<ResultRef>()?.id())),
        },
    }
}

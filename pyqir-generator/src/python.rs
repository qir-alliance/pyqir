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
        self, BinaryKind, BinaryOp, Call, ClassicalRegister, Controlled, If, Instruction, Int,
        IntPredicate, Measured, QuantumRegister, Rotated, SemanticModel, Single, Type, Variable,
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
    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;
    m.add_class::<ResultRef>()?;
    m.add_class::<Function>()?;
    m.add_class::<Builder>()?;
    m.add_class::<Value>()?;
    m.add("const", wrap_pyfunction!(constant, m)?)?;
    m.add_class::<SimpleModule>()?;
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

#[derive(Clone, Copy, FromPyObject)]
struct PyIntType {
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

struct PyFunctionType {
    params: Vec<PyType>,
    result: Box<PyType>,
}

impl<'source> FromPyObject<'source> for PyFunctionType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let params = ob.getattr("params")?.extract()?;
        let result = Box::new(ob.getattr("result")?.extract()?);
        Ok(PyFunctionType { params, result })
    }
}

#[derive(FromPyObject)]
enum PyType {
    Void(PyVoidType),
    Int(PyIntType),
    Double(PyDoubleType),
    Qubit(PyQubitType),
    Result(PyResultType),
    Function(PyFunctionType),
}

impl From<PyType> for Type {
    fn from(ty: PyType) -> Self {
        match ty {
            PyType::Void(PyVoidType) => Type::Void,
            PyType::Int(PyIntType { width }) => Type::Int { width },
            PyType::Double(PyDoubleType) => Type::Double,
            PyType::Qubit(PyQubitType) => Type::Qubit,
            PyType::Result(PyResultType) => Type::Result,
            PyType::Function(PyFunctionType { params, result }) => Type::Function {
                params: params.into_iter().map(Into::into).collect(),
                result: Box::new((*result).into()),
            },
        }
    }
}

struct PyIPredicate(IntPredicate);

impl<'source> FromPyObject<'source> for PyIPredicate {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let predicate = match ob.getattr("name")?.extract()? {
            "EQ" => Ok(IntPredicate::EQ),
            "NE" => Ok(IntPredicate::NE),
            "UGT" => Ok(IntPredicate::UGT),
            "UGE" => Ok(IntPredicate::UGE),
            "ULT" => Ok(IntPredicate::ULT),
            "ULE" => Ok(IntPredicate::ULE),
            "SGT" => Ok(IntPredicate::SGT),
            "SGE" => Ok(IntPredicate::SGE),
            "SLT" => Ok(IntPredicate::SLT),
            "SLE" => Ok(IntPredicate::SLE),
            _ => Err(PyValueError::new_err("Invalid predicate.")),
        }?;

        Ok(PyIPredicate(predicate))
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
            _ => Err(PyTypeError::new_err("Only equality is supported.")),
        }
    }
}

#[derive(Clone)]
#[pyclass]
struct Function {
    name: String,
    ty: Type,
}

#[derive(Clone)]
#[pyclass]
struct Value(interop::Value);

#[pyproto]
impl PyObjectProtocol for Value {
    fn __repr__(&self) -> String {
        match &self.0 {
            interop::Value::Int(int) => {
                format!("const(types.Int({}), {})", int.width(), int.value())
            }
            interop::Value::Double(value) => format!("const(types.DOUBLE, {})", value),
            interop::Value::Qubit(name) | interop::Value::Result(name) => {
                format!("<Value {}>", name)
            }
            interop::Value::Variable(var) => format!("<Value {:?}>", var),
        }
    }
}

#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn constant(ty: PyType, value: &PyAny) -> PyResult<Value> {
    match ty {
        PyType::Int(PyIntType { width }) => extract_value(&Type::Int { width }, value),
        PyType::Double(PyDoubleType) => extract_value(&Type::Double, value),
        _ => Err(PyTypeError::new_err(
            "Constant values are not supported for this type.",
        )),
    }
    .map(Value)
}

#[pyclass]
struct Builder {
    frames: Vec<Vec<Instruction>>,
    external_functions: Vec<Function>,
    last_variable: Option<Variable>,
}

#[pymethods]
impl Builder {
    #[new]
    fn new() -> Builder {
        Builder {
            frames: vec![vec![]],
            external_functions: vec![],
            last_variable: None,
        }
    }

    fn neg(&mut self, value: Value) -> PyResult<Value> {
        match value.0.type_of() {
            Type::Int { width, .. } => {
                let zero = interop::Value::Int(Int::new(width, 0).unwrap());
                Ok(self.push_binary_op(BinaryKind::Sub, zero, value.0))
            }
            _ => Err(PyTypeError::new_err("Value must be an integer.")),
        }
    }

    #[pyo3(name = "and_")]
    fn and(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::And, lhs.0, rhs.0)
    }

    #[pyo3(name = "or_")]
    fn or(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Or, lhs.0, rhs.0)
    }

    fn xor(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Xor, lhs.0, rhs.0)
    }

    fn add(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Add, lhs.0, rhs.0)
    }

    fn sub(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Sub, lhs.0, rhs.0)
    }

    fn mul(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Mul, lhs.0, rhs.0)
    }

    fn shl(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Shl, lhs.0, rhs.0)
    }

    fn lshr(&mut self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::LShr, lhs.0, rhs.0)
    }

    fn icmp(&mut self, predicate: PyIPredicate, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::ICmp(predicate.0), lhs.0, rhs.0)
    }

    fn call(&mut self, function: Function, args: &PySequence) -> PyResult<Option<Value>> {
        let (param_types, return_type) = match function.ty {
            Type::Function { params, result } => (params, result),
            _ => panic!("Invalid function type."),
        };

        let num_params = param_types.len();
        let num_args = args.len()?;
        if num_params != num_args {
            let message = format!("Expected {} arguments, got {}.", num_params, num_args);
            return Err(PyValueError::new_err(message));
        }

        let args = args
            .iter()?
            .zip(&param_types)
            .map(|(arg, ty)| extract_value(ty, arg?))
            .collect::<PyResult<_>>()?;

        let result = match *return_type {
            Type::Void => None,
            _ => Some(self.fresh_variable(*return_type)),
        };

        self.push_inst(Instruction::Call(Call {
            name: function.name,
            args,
            result: result.clone(),
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

    fn fresh_variable(&mut self, ty: Type) -> Variable {
        let v = match &self.last_variable {
            None => Variable::new(ty),
            Some(v) => v.next(ty),
        };
        self.last_variable = Some(v.clone());
        v
    }

    fn push_binary_op(
        &mut self,
        kind: BinaryKind,
        lhs: interop::Value,
        rhs: interop::Value,
    ) -> Value {
        // TODO: Check both types are equal.
        let result = self.fresh_variable(lhs.type_of());
        self.push_inst(Instruction::BinaryOp(BinaryOp {
            kind,
            lhs,
            rhs,
            result: result.clone(),
        }));

        Value(interop::Value::Variable(result))
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

    fn add_external_function(&mut self, py: Python, name: String, ty: PyType) -> Function {
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
        let theta = extract_value(&Type::Double, theta)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Rx(rotated));
        Ok(())
    }

    fn ry(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(&Type::Double, theta)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Ry(rotated));
        Ok(())
    }

    fn rz(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(&Type::Double, theta)?;
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

fn extract_sentinel(module_name: &str, type_name: &str, ob: &PyAny) -> PyResult<()> {
    let module: &str = ob.get_type().getattr("__module__")?.extract()?;

    if module == module_name && ob.get_type().name()? == type_name {
        Ok(())
    } else {
        let message = format!("Expected {}.{}.", module_name, type_name);
        Err(PyTypeError::new_err(message))
    }
}

fn extract_value(ty: &Type, ob: &PyAny) -> PyResult<interop::Value> {
    match ob.extract::<Value>() {
        Ok(value) => Ok(value.0),
        Err(_) => match ty {
            &Type::Int { width } => Int::new(width, ob.extract()?)
                .map(interop::Value::Int)
                .ok_or_else(|| {
                    let message = format!("Value too big for {}-bit integer.", width);
                    PyOverflowError::new_err(message)
                }),
            Type::Double => Ok(interop::Value::Double(ob.extract()?)),
            Type::Result => Ok(interop::Value::Result(ob.extract::<ResultRef>()?.id())),
            Type::Void | Type::Qubit | Type::Function { .. } => Err(PyTypeError::new_err(
                "Can't convert Python value into this type.",
            )),
        },
    }
}

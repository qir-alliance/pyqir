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
        self, BinaryKind, BinaryOp, Call, ClassicalRegister, Controlled, FunctionType, If,
        Instruction, IntPredicate, Integer, Measured, QuantumRegister, ReturnType, Rotated,
        SemanticModel, Single, ValueType, Variable,
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
    m.add_class::<Qubit>()?;
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

#[derive(Clone)]
#[pyclass]
struct Value(interop::Value);

#[pyclass]
struct Builder {
    frames: Vec<Vec<Instruction>>,
    external_functions: Vec<(String, FunctionType)>,
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
            ValueType::Integer { width, .. } => {
                let zero = interop::Value::Integer(Integer::new(width, 0).unwrap());
                Ok(self.push_binary_op(BinaryKind::Sub, zero, value.0))
            }
            _ => Err(PyErr::new::<PyTypeError, _>("Value must be an integer.")),
        }
    }

    #[pyo3(name = "and_")]
    fn and(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::And, lhs, rhs)
    }

    #[pyo3(name = "or_")]
    fn or(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::Or, lhs, rhs)
    }

    fn xor(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::Xor, lhs, rhs)
    }

    fn add(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::Add, lhs, rhs)
    }

    fn sub(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::Sub, lhs, rhs)
    }

    fn mul(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::Mul, lhs, rhs)
    }

    fn shl(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::Shl, lhs, rhs)
    }

    fn lshr(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::LShr, lhs, rhs)
    }

    fn icmp_eq(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::EQ), lhs, rhs)
    }

    fn icmp_neq(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::NE), lhs, rhs)
    }

    fn icmp_ugt(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::UGT), lhs, rhs)
    }

    fn icmp_uge(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::UGE), lhs, rhs)
    }

    fn icmp_ult(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::ULT), lhs, rhs)
    }

    fn icmp_ule(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::ULE), lhs, rhs)
    }

    fn icmp_sgt(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::SGT), lhs, rhs)
    }

    fn icmp_sge(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::SGE), lhs, rhs)
    }

    fn icmp_slt(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::SLT), lhs, rhs)
    }

    fn icmp_sle(&mut self, lhs: &PyAny, rhs: &PyAny) -> PyResult<Value> {
        self.push_binary_op_any(BinaryKind::ICmp(IntPredicate::SLE), lhs, rhs)
    }

    fn call(&mut self, function: Function, args: &PySequence) -> PyResult<Option<Value>> {
        let (_, ty) = self
            .external_functions
            .iter()
            .find(|f| f.0 == function.name)
            .expect("Function not found in module.");

        let num_params = ty.param_types.len();
        let num_args = args.len()?;
        if num_params != num_args {
            let message = format!("Expected {} arguments, got {}.", num_params, num_args);
            return Err(PyErr::new::<PyValueError, _>(message));
        }

        let args = args
            .iter()?
            .zip(&ty.param_types)
            .map(|(arg, &ty)| extract_value(arg?, ty))
            .collect::<PyResult<_>>()?;

        let result = match ty.return_type {
            ReturnType::Void => None,
            ReturnType::Value(ty) => Some(self.fresh_variable(ty)),
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

    fn fresh_variable(&mut self, ty: ValueType) -> Variable {
        let v = match self.last_variable {
            None => Variable::new(ty),
            Some(v) => v.next(ty),
        };
        self.last_variable = Some(v);
        v
    }

    fn push_binary_op_any(
        &mut self,
        kind: BinaryKind,
        lhs: &PyAny,
        rhs: &PyAny,
    ) -> PyResult<Value> {
        let (lhs, rhs) = extract_binary_operands(lhs, rhs)?;
        Ok(self.push_binary_op(kind, lhs, rhs))
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
            result,
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
        builder.external_functions.push((name.clone(), ty.into()));
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

    fn rx(&self, py: Python, theta: &PyAny, qubit: &Qubit) -> PyResult<()> {
        let theta = extract_value(theta, ValueType::Double)?;
        let rotated = Rotated::new(theta, qubit.id());
        self.push_inst(py, Instruction::Rx(rotated));
        Ok(())
    }

    fn ry(&self, py: Python, theta: &PyAny, qubit: &Qubit) -> PyResult<()> {
        let theta = extract_value(theta, ValueType::Double)?;
        let rotated = Rotated::new(theta, qubit.id());
        self.push_inst(py, Instruction::Ry(rotated));
        Ok(())
    }

    fn rz(&self, py: Python, theta: &PyAny, qubit: &Qubit) -> PyResult<()> {
        let theta = extract_value(theta, ValueType::Double)?;
        let rotated = Rotated::new(theta, qubit.id());
        self.push_inst(py, Instruction::Rz(rotated));
        Ok(())
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
            ValueType::Qubit => Ok(interop::Value::Qubit(ob.extract::<Qubit>()?.id())),
            ValueType::Result => Ok(interop::Value::Result(ob.extract::<ResultRef>()?.id())),
        },
    }
}

fn extract_binary_operands(lhs: &PyAny, rhs: &PyAny) -> PyResult<(interop::Value, interop::Value)> {
    match lhs.extract::<Value>() {
        Ok(Value(lhs)) => {
            let ty = lhs.type_of();
            let rhs = extract_value(rhs, ty)?;
            Ok((lhs, rhs))
        }
        Err(_) => match rhs.extract::<Value>() {
            Ok(Value(rhs)) => {
                let ty = rhs.type_of();
                let lhs = extract_value(lhs, ty)?;
                Ok((lhs, rhs))
            }
            Err(_) => Err(PyErr::new::<PyTypeError, _>(
                "At least one operand must be a Value.",
            )),
        },
    }
}

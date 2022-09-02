// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// pyo3 generates errors with _obj and _tmp values
#![allow(clippy::used_underscore_binding)]
// Some arguments get turned into Deref by PyO3 macros, which we can't control.
#![allow(clippy::borrow_deref_ref, clippy::needless_option_as_deref)]
// This was introduced in 1.62, but we can't update the dependency to
// to resolve it until we move to a newer version of python.
#![allow(clippy::format_push_string)]

use pyo3::{
    exceptions::{PyOSError, PyOverflowError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PySequence, PyString, PyUnicode},
    PyObjectProtocol,
};
use qirlib::generation::{
    emit,
    interop::{
        self, BinaryKind, BinaryOp, Call, Controlled, If, IfResult, Instruction, Int, IntPredicate,
        Measured, Rotated, SemanticModel, Single, Type, Variable,
    },
};
use std::{cell::RefCell, vec};

#[pymodule]
#[pyo3(name = "_native")]
fn native_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ir_to_bitcode, m)?)?;
    m.add_function(wrap_pyfunction!(bitcode_to_ir, m)?)?;
    m.add_class::<Function>()?;
    m.add_class::<Builder>()?;
    m.add_class::<Value>()?;
    m.add("const", wrap_pyfunction!(constant, m)?)?;
    m.add_class::<SimpleModule>()?;
    m.add_class::<BasicQisBuilder>()
}

const TYPES_MODULE_NAME: &str = "pyqir.generator.types";

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

struct PyIntPredicate(IntPredicate);

impl<'source> FromPyObject<'source> for PyIntPredicate {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        match ob.getattr("name")?.extract()? {
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
        }
        .map(Self)
    }
}

/// A QIR function.
#[derive(Clone)]
#[pyclass]
struct Function {
    name: String,
    ty: Type,
}

/// A QIR value.
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

/// Creates a constant QIR value.
///
/// :param Type ty: The type of the value.
/// :param Union[int, float] value: A Python value that will be converted into a QIR value.
/// :returns: The constant QIR value.
/// :rtype: Value
#[pyfunction]
#[pyo3(text_signature = "(ty, value)")]
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

/// An instruction builder.
#[pyclass]
struct Builder {
    external_functions: Vec<Function>,
    next_variable: RefCell<Variable>,
    frames: RefCell<Vec<Vec<Instruction>>>,
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
    fn and_(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::And, lhs.0, rhs.0)
    }

    /// Inserts a bitwise logical or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn or_(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Or, lhs.0, rhs.0)
    }

    /// Inserts a bitwise logical exclusive or instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn xor(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Xor, lhs.0, rhs.0)
    }

    /// Inserts an addition instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The sum.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn add(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Add, lhs.0, rhs.0)
    }

    /// Inserts a subtraction instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The difference.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn sub(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Sub, lhs.0, rhs.0)
    }

    /// Inserts a multiplication instruction.
    ///
    /// :param Value lhs: The left-hand side.
    /// :param Value rhs: The right-hand side.
    /// :returns: The product.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn mul(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Mul, lhs.0, rhs.0)
    }

    /// Inserts a shift left instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn shl(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::Shl, lhs.0, rhs.0)
    }

    /// Inserts a logical (zero fill) shift right instruction.
    ///
    /// :param Value lhs: The value to shift.
    /// :param Value rhs: The number of bits to shift by.
    /// :returns: The result.
    /// :rtype: Value
    #[pyo3(text_signature = "(self, lhs, rhs)")]
    fn lshr(&self, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::LShr, lhs.0, rhs.0)
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
    fn icmp(&self, pred: PyIntPredicate, lhs: Value, rhs: Value) -> Value {
        self.push_binary_op(BinaryKind::ICmp(pred.0), lhs.0, rhs.0)
    }

    /// Inserts a call instruction.
    ///
    /// :param Function function: The function to call.
    /// :param Sequence[Union[Value, bool, int, float]] args: The arguments to the function.
    /// :returns: The return value, or None if the function has a void return type.
    /// :rtype: Optional[Value]
    #[pyo3(text_signature = "(self, function, args)")]
    fn call(&self, function: Function, args: &PySequence) -> PyResult<Option<Value>> {
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
            _ => Some(self.fresh_variable()),
        };

        self.push_inst(Instruction::Call(Call {
            name: function.name,
            args,
            result,
        }));

        Ok(result.map(|v| Value(interop::Value::Variable(v))))
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
    fn if_(&self, cond: Value, r#true: Option<&PyAny>, r#false: Option<&PyAny>) -> PyResult<()> {
        let if_ = If {
            cond: cond.0,
            if_true: build_frame(self, r#true)?,
            if_false: build_frame(self, r#false)?,
        };
        self.push_inst(Instruction::If(if_));
        Ok(())
    }
}

impl Builder {
    fn new() -> Builder {
        Builder {
            external_functions: vec![],
            next_variable: RefCell::new(Variable::new()),
            frames: RefCell::new(vec![vec![]]),
        }
    }

    fn push_inst(&self, inst: Instruction) {
        self.frames.borrow_mut().last_mut().unwrap().push(inst);
    }

    fn push_frame(&self) {
        self.frames.borrow_mut().push(vec![]);
    }

    fn pop_frame(&self) -> Option<Vec<Instruction>> {
        self.frames.borrow_mut().pop()
    }

    fn fresh_variable(&self) -> Variable {
        let mut next_variable = self.next_variable.borrow_mut();
        let v = *next_variable;
        *next_variable = v.next();
        v
    }

    fn push_binary_op(&self, kind: BinaryKind, lhs: interop::Value, rhs: interop::Value) -> Value {
        let result = self.fresh_variable();
        self.push_inst(Instruction::BinaryOp(BinaryOp {
            kind,
            lhs,
            rhs,
            result,
        }));
        Value(interop::Value::Variable(result))
    }
}

/// A simple module represents an executable QIR program with these restrictions:
///
/// - There is one global qubit register and one global result register. Both are statically
///   allocated with a fixed size.
/// - There is only a single function that runs as the entry point.
///
/// :param str name: The name of the module.
/// :param int num_qubits: The number of statically allocated qubits.
/// :param int num_results: The number of statically allocated results.
#[pyclass]
#[pyo3(text_signature = "(name, num_qubits, num_results)")]
struct SimpleModule {
    model: SemanticModel,
    builder: Py<Builder>,
    num_qubits: u64,
    num_results: u64,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(py: Python, name: String, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let model = SemanticModel {
            name,
            external_functions: vec![],
            instructions: vec![],
        };

        Ok(SimpleModule {
            model,
            builder: Py::new(py, Builder::new())?,
            num_qubits,
            num_results,
        })
    }

    /// The global qubit register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn qubits(&self) -> Vec<Value> {
        (0..self.num_qubits)
            .map(|id| Value(interop::Value::Qubit(id)))
            .collect()
    }

    /// The global result register.
    ///
    /// :type: Tuple[Value, ...]
    #[getter]
    fn results(&self) -> Vec<Value> {
        (0..self.num_results)
            .map(|id| Value(interop::Value::Result(id)))
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
        let model = self.model_from_builder(py);
        emit::ir(&model).map_err(PyOSError::new_err)
    }

    /// Emits the LLVM bitcode for the module as a sequence of bytes.
    ///
    /// :rtype: bytes
    fn bitcode<'a>(&self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        let model = self.model_from_builder(py);
        match emit::bitcode(&model) {
            Ok(bitcode) => Ok(PyBytes::new(py, &bitcode[..])),
            Err(err) => Err(PyOSError::new_err(err)),
        }
    }

    /// Adds a declaration for an externally linked function to the module.
    ///
    /// :param str name: The name of the function.
    /// :param Type ty: The type of the function.
    /// :return: The function value.
    /// :rtype: Function
    #[pyo3(text_signature = "(self, name, ty)")]
    fn add_external_function(&mut self, py: Python, name: String, ty: PyType) -> Function {
        let mut builder = self.builder.as_ref(py).borrow_mut();
        let ty = ty.into();
        let function = Function { name, ty };
        builder.external_functions.push(function.clone());
        function
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

        let frames = builder.frames.borrow();
        match frames[..] {
            [ref instructions] => SemanticModel {
                external_functions,
                instructions: instructions.clone(),
                ..self.model.clone()
            },
            _ => panic!("Builder does not contain exactly one stack frame."),
        }
    }
}

fn build_frame(builder: &Builder, callback: Option<&PyAny>) -> PyResult<Vec<Instruction>> {
    builder.push_frame();
    if let Some(callback) = callback {
        callback.call0()?;
    }
    Ok(builder.pop_frame().unwrap())
}

/// An instruction builder that generates instructions from the basic quantum instruction set.
///
/// :param Builder builder: The underlying builder used to build QIS instructions.
#[pyclass]
#[pyo3(text_signature = "(builder)")]
struct BasicQisBuilder {
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
    fn cx(&self, py: Python, control: Value, target: Value) {
        let controlled = Controlled::new(control.0, target.0);
        self.push_inst(py, Instruction::Cx(controlled));
    }

    /// Inserts a controlled Pauli :math:`Z` gate.
    ///
    /// :param Value control: The control qubit.
    /// :param Value target: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, control, target)")]
    fn cz(&self, py: Python, control: Value, target: Value) {
        let controlled = Controlled::new(control.0, target.0);
        self.push_inst(py, Instruction::Cz(controlled));
    }

    /// Inserts a Hadamard gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn h(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::H(single));
    }

    /// Inserts a Z-basis measurement operation.
    ///
    /// :param Value qubit: The qubit to measure.
    /// :param Value result: A result where the measurement result will be written to.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit, result)")]
    fn mz(&self, py: Python, qubit: Value, result: Value) {
        let measured = Measured::new(qubit.0, result.0);
        self.push_inst(py, Instruction::M(measured));
    }

    /// Inserts a reset operation.
    ///
    /// :param Value qubit: The qubit to reset.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn reset(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::Reset(single));
    }

    /// Inserts a rotation gate about the :math:`x` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rx(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(&Type::Double, theta)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Rx(rotated));
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`y` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn ry(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(&Type::Double, theta)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Ry(rotated));
        Ok(())
    }

    /// Inserts a rotation gate about the :math:`z` axis.
    ///
    /// :param Union[Value, float] theta: The angle to rotate by.
    /// :param Value qubit: The qubit to rotate.
    /// :rtype: None
    #[pyo3(text_signature = "(self, theta, qubit)")]
    fn rz(&self, py: Python, theta: &PyAny, qubit: Value) -> PyResult<()> {
        let theta = extract_value(&Type::Double, theta)?;
        let rotated = Rotated::new(theta, qubit.0);
        self.push_inst(py, Instruction::Rz(rotated));
        Ok(())
    }

    /// Inserts an :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::S(single));
    }

    /// Inserts an adjoint :math:`S` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn s_adj(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::SAdj(single));
    }

    /// Inserts a :math:`T` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::T(single));
    }

    /// Inserts an adjoint :math:`T` gate.
    ///
    /// :param qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn t_adj(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::TAdj(single));
    }

    /// Inserts a Pauli :math:`X` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn x(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::X(single));
    }

    /// Inserts a Pauli :math:`Y` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn y(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::Y(single));
    }

    /// Inserts a Pauli :math:`Z` gate.
    ///
    /// :param Value qubit: The target qubit.
    /// :rtype: None
    #[pyo3(text_signature = "(self, qubit)")]
    fn z(&self, py: Python, qubit: Value) {
        let single = Single::new(qubit.0);
        self.push_inst(py, Instruction::Z(single));
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
        cond: Value,
        one: Option<&PyAny>,
        zero: Option<&PyAny>,
    ) -> PyResult<()> {
        let builder = self.builder.borrow(py);
        let if_result = IfResult {
            cond: cond.0,
            if_one: build_frame(&builder, one)?,
            if_zero: build_frame(&builder, zero)?,
        };
        builder.push_inst(Instruction::IfResult(if_result));
        Ok(())
    }
}

impl BasicQisBuilder {
    fn push_inst(&self, py: Python, inst: Instruction) {
        let builder = self.builder.as_ref(py).borrow();
        builder.push_inst(inst);
    }
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
#[allow(clippy::needless_pass_by_value)]
fn ir_to_bitcode<'a>(
    py: Python<'a>,
    ir: &str,
    module_name: Option<String>,
    source_file_name: Option<String>,
) -> PyResult<&'a PyBytes> {
    let bitcode = qirlib::generation::ir_to_bitcode(ir, &module_name, &source_file_name)
        .map_err(PyOSError::new_err)?;
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
#[allow(clippy::needless_pass_by_value)]
fn bitcode_to_ir<'a>(
    py: Python<'a>,
    bitcode: &PyBytes,
    module_name: Option<String>,
    source_file_name: Option<String>,
) -> PyResult<&'a PyString> {
    let ir = qirlib::generation::bitcode_to_ir(bitcode.as_bytes(), &module_name, &source_file_name)
        .map_err(PyOSError::new_err)?;
    Ok(PyUnicode::new(py, ir.as_str()))
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
            Type::Void | Type::Qubit | Type::Result | Type::Function { .. } => Err(
                PyTypeError::new_err("Can't convert Python value into this type."),
            ),
        },
    }
}

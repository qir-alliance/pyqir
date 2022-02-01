// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    emit::{get_bitcode_base64_string, get_ir_string, write_model_to_file},
    interop::{
        self, ClassicalRegister, Controlled, If, Measured, QuantumRegister, Rotated, SemanticModel,
        Single,
    },
};
use pyo3::{exceptions::PyOSError, prelude::*};

#[pymodule]
fn pyqir_generator(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Instruction>()?;
    m.add_function(wrap_pyfunction!(cx, m)?)?;
    m.add_function(wrap_pyfunction!(cz, m)?)?;
    m.add_function(wrap_pyfunction!(h, m)?)?;
    m.add_function(wrap_pyfunction!(m, m)?)?;
    m.add_function(wrap_pyfunction!(reset, m)?)?;
    m.add_function(wrap_pyfunction!(rx, m)?)?;
    m.add_function(wrap_pyfunction!(ry, m)?)?;
    m.add_function(wrap_pyfunction!(rz, m)?)?;
    m.add_function(wrap_pyfunction!(s, m)?)?;
    m.add_function(wrap_pyfunction!(s_adj, m)?)?;
    m.add_function(wrap_pyfunction!(t, m)?)?;
    m.add_function(wrap_pyfunction!(t_adj, m)?)?;
    m.add_function(wrap_pyfunction!(x, m)?)?;
    m.add_function(wrap_pyfunction!(y, m)?)?;
    m.add_function(wrap_pyfunction!(z, m)?)?;
    m.add(
        "dump_machine",
        Instruction(interop::Instruction::DumpMachine),
    )?;
    m.add("if_", wrap_pyfunction!(if_, m)?)?;

    m.add_class::<Register>()?;
    m.add_class::<Module>()?;

    m.add_function(wrap_pyfunction!(enable_logging, m)?)
}

#[pyclass]
#[derive(Clone)]
struct Instruction(interop::Instruction);

#[pyfunction]
fn cx(control: String, target: String) -> Instruction {
    Instruction(interop::Instruction::Cx(Controlled { control, target }))
}

#[pyfunction]
fn cz(control: String, target: String) -> Instruction {
    Instruction(interop::Instruction::Cz(Controlled { control, target }))
}

#[pyfunction]
fn h(qubit: String) -> Instruction {
    Instruction(interop::Instruction::H(Single { qubit }))
}

#[pyfunction]
fn m(qubit: String, target: String) -> Instruction {
    Instruction(interop::Instruction::M(Measured { qubit, target }))
}

#[pyfunction]
fn reset(qubit: String) -> Instruction {
    Instruction(interop::Instruction::Reset(Single { qubit }))
}

#[pyfunction]
fn rx(theta: f64, qubit: String) -> Instruction {
    Instruction(interop::Instruction::Rx(Rotated { theta, qubit }))
}

#[pyfunction]
fn ry(theta: f64, qubit: String) -> Instruction {
    Instruction(interop::Instruction::Ry(Rotated { theta, qubit }))
}

#[pyfunction]
fn rz(theta: f64, qubit: String) -> Instruction {
    Instruction(interop::Instruction::Rz(Rotated { theta, qubit }))
}

#[pyfunction]
fn s(qubit: String) -> Instruction {
    Instruction(interop::Instruction::S(Single { qubit }))
}

#[pyfunction]
fn s_adj(qubit: String) -> Instruction {
    Instruction(interop::Instruction::SAdj(Single { qubit }))
}

#[pyfunction]
fn t(qubit: String) -> Instruction {
    Instruction(interop::Instruction::T(Single { qubit }))
}

#[pyfunction]
fn t_adj(qubit: String) -> Instruction {
    Instruction(interop::Instruction::TAdj(Single { qubit }))
}

#[pyfunction]
fn x(qubit: String) -> Instruction {
    Instruction(interop::Instruction::X(Single { qubit }))
}

#[pyfunction]
fn y(qubit: String) -> Instruction {
    Instruction(interop::Instruction::Y(Single { qubit }))
}

#[pyfunction]
fn z(qubit: String) -> Instruction {
    Instruction(interop::Instruction::Z(Single { qubit }))
}

#[pyfunction]
fn if_(condition: String, r#true: Vec<Instruction>, r#false: Vec<Instruction>) -> Instruction {
    let true_insts = r#true.into_iter().map(|i| i.0).collect();
    let false_insts = r#false.into_iter().map(|i| i.0).collect();

    Instruction(interop::Instruction::If(If {
        condition,
        true_insts,
        false_insts,
    }))
}

#[pyclass]
#[derive(Clone)]
struct Register {
    name: String,
    size: u64,
}

#[pymethods]
impl Register {
    #[new]
    fn new(name: String, size: u64) -> Register {
        Register { name, size }
    }
}

impl Register {
    fn into_classical(self) -> ClassicalRegister {
        ClassicalRegister::new(self.name, self.size)
    }

    fn quantum(&self) -> impl Iterator<Item = QuantumRegister> + '_ {
        (0..self.size).map(move |i| QuantumRegister::new(self.name.clone(), i))
    }
}

#[pyclass]
struct Module(SemanticModel);

#[pymethods]
impl Module {
    #[allow(clippy::needless_pass_by_value)]
    #[new]
    fn new(
        name: String,
        bits: Vec<Register>,
        qubits: Vec<Register>,
        instructions: Vec<Instruction>,
    ) -> Module {
        let registers = bits.into_iter().map(Register::into_classical).collect();
        let qubits = qubits.iter().flat_map(Register::quantum).collect();
        let instructions = instructions.into_iter().map(|i| i.0).collect();

        Module(SemanticModel {
            name,
            registers,
            qubits,
            instructions,
        })
    }

    fn ir(&self) -> PyResult<String> {
        get_ir_string(&self.0).map_err(PyOSError::new_err)
    }

    fn write_ir(&self, path: &str) -> PyResult<()> {
        write_model_to_file(&self.0, path).map_err(PyOSError::new_err)
    }

    fn bitcode_base64(&self) -> PyResult<String> {
        get_bitcode_base64_string(&self.0).map_err(PyOSError::new_err)
    }
}

#[pyfunction]
fn enable_logging() -> PyResult<()> {
    env_logger::try_init().map_err(|e| PyOSError::new_err(e.to_string()))
}

const RESULT_NAME: &str = "result";
const QUBIT_NAME: &str = "qubit";

#[pyclass]
struct SimpleModule {
    builder: Py<Builder>,
}

#[pymethods]
impl SimpleModule {
    #[new]
    fn new(name: String, num_qubits: u64, num_results: u64) -> PyResult<SimpleModule> {
        let registers = vec![ClassicalRegister::new(RESULT_NAME.to_string(), num_results)];

        let qubits = (0..num_qubits)
            .map(|i| QuantumRegister::new(QUBIT_NAME.to_string(), i))
            .collect();

        let model = SemanticModel {
            name,
            registers,
            qubits,
            instructions: Vec::new(),
        };

        Python::with_gil(|py| {
            let builder = Py::new(py, Builder { model })?;
            Ok(SimpleModule { builder })
        })
    }

    #[getter]
    fn qubits(&self) -> PyResult<Vec<Qubit>> {
        Python::with_gil(|py| {
            let builder = self.builder.as_ref(py).try_borrow()?;
            Ok(builder
                .model
                .qubits
                .iter()
                .map(|q| Qubit { index: q.index })
                .collect())
        })
    }

    #[getter]
    fn results(&self) -> PyResult<Vec<Ref>> {
        Python::with_gil(|py| {
            let builder = self.builder.as_ref(py).try_borrow()?;
            let size = builder.model.registers.first().unwrap().size;

            Ok((0..size)
                .map(|index| Ref(RefKind::Result { index }))
                .collect())
        })
    }

    #[getter]
    fn builder(&self) -> Py<Builder> {
        self.builder.clone()
    }

    fn ir(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let builder = self.builder.as_ref(py).try_borrow()?;
            get_ir_string(&builder.model).map_err(PyOSError::new_err)
        })
    }

    fn bitcode(&self) -> &[u8] {
        todo!()
    }
}

#[pyclass]
struct Qubit {
    index: u64,
}

#[pyclass]
struct Ref(RefKind);

enum RefKind {
    Result { index: u64 },
}

#[pyclass]
struct Builder {
    model: SemanticModel,
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

    fn cx(&self, control: &Qubit, target: &Qubit) {
        todo!()
    }

    fn cz(&self, control: &Qubit, target: &Qubit) {
        todo!()
    }

    fn h(&self, qubit: &Qubit) {
        todo!()
    }

    fn m(&self, qubit: &Qubit, result: &Ref) {
        todo!()
    }

    fn reset(&self, qubit: &Qubit) {
        todo!()
    }

    fn rx(&self, theta: f64, qubit: &Qubit) {
        todo!()
    }

    fn ry(&self, theta: f64, qubit: &Qubit) {
        todo!()
    }

    fn rz(&self, theta: f64, qubit: &Qubit) {
        todo!()
    }

    fn s(&self, qubit: &Qubit) {
        todo!()
    }

    fn s_adj(&self, qubit: &Qubit) {
        todo!()
    }

    fn t(&self, qubit: &Qubit) {
        todo!()
    }

    fn t_adj(&self, qubit: &Qubit) {
        todo!()
    }

    fn x(&self, qubit: &Qubit) {
        todo!()
    }

    fn y(&self, qubit: &Qubit) {
        todo!()
    }

    fn z(&self, qubit: &Qubit) {
        todo!()
    }

    fn if_result(&self, result: &Ref, one: &PyAny, zero: &PyAny) {
        todo!()
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    emit::{get_bitcode_base64_string, get_ir_string, write_model_to_file},
    interop::{
        self, ClassicalRegister, Controlled, Measured, QuantumRegister, Rotated, SemanticModel,
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
    let if_true = r#true.into_iter().map(|i| i.0).collect();
    let if_false = r#false.into_iter().map(|i| i.0).collect();
    Instruction(interop::Instruction::If(condition, if_true, if_false))
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

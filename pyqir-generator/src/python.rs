// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    emit,
    interop::{
        ClassicalRegister, Controlled, If, Instruction, Measured, QuantumRegister, Rotated,
        SemanticModel, Single,
    },
};
use pyo3::{
    basic::CompareOp,
    exceptions::{PyOSError, PyTypeError},
    prelude::*,
    PyObjectProtocol,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SimpleModule>()?;
    m.add_class::<Qubit>()?;
    m.add_class::<Ref>()?;
    m.add_class::<Builder>()?;
    m.add_class::<BasicQisBuilder>()
}

const RESULT_NAME: &str = "result";
const QUBIT_NAME: &str = "qubit";

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
            registers,
            qubits,
            instructions: Vec::new(),
            static_alloc: true,
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
    fn results(&self) -> Vec<Ref> {
        let size = self.model.registers.first().unwrap().size;
        (0..size)
            .map(|index| Ref(RefKind::Result { index }))
            .collect()
    }

    #[getter]
    fn builder(&self) -> Py<Builder> {
        self.builder.clone()
    }

    fn ir(&self, py: Python) -> PyResult<String> {
        let model = self.model_with_builder_instructions(py);
        emit::ir(&model).map_err(PyOSError::new_err)
    }

    fn bitcode(&self, py: Python) -> PyResult<Vec<u8>> {
        let model = self.model_with_builder_instructions(py);
        emit::bitcode(&model).map_err(PyOSError::new_err)
    }
}

impl SimpleModule {
    fn model_with_builder_instructions(&self, py: Python) -> SemanticModel {
        let builder = self.builder.as_ref(py).borrow();

        match builder.frames[..] {
            [ref instructions] => SemanticModel {
                instructions: instructions.clone(),
                ..self.model.clone()
            },
            _ => panic!("Builder does not contain exactly one stack frame."),
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
    fn __hash__(&self) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish())
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
struct Ref(RefKind);

#[pyproto]
impl PyObjectProtocol for Ref {
    fn __hash__(&self) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn __richcmp__(&self, other: Ref, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self == &other),
            _ => Err(PyErr::new::<PyTypeError, _>("Only equality is supported.")),
        }
    }
}

impl Ref {
    fn id(&self) -> String {
        let Ref(RefKind::Result { index }) = self;
        format!("{}{}", RESULT_NAME, index)
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum RefKind {
    Result { index: u64 },
}

#[pyclass]
struct Builder {
    frames: Vec<Vec<Instruction>>,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            frames: vec![vec![]],
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

    fn m(&self, py: Python, qubit: &Qubit, result: &Ref) {
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
        result: &Ref,
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
        builder.frames.last_mut().unwrap().push(inst);
    }

    fn push_frame(&self, py: Python) {
        let mut builder = self.builder.as_ref(py).borrow_mut();
        builder.frames.push(vec![]);
    }

    fn pop_frame(&self, py: Python) -> Option<Vec<Instruction>> {
        let mut builder = self.builder.as_ref(py).borrow_mut();
        builder.frames.pop()
    }
}

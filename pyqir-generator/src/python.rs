// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    emit::get_ir_string,
    interop::{
        ClassicalRegister, Controlled, If, Instruction, Measured, QuantumRegister, Rotated,
        SemanticModel, Single,
    },
};
use pyo3::{exceptions::PyOSError, prelude::*};

#[pymodule]
fn pyqir_generator(_py: Python, m: &PyModule) -> PyResult<()> {
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
            let builder = Py::new(py, Builder::new())?;
            Ok(SimpleModule { model, builder })
        })
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

    fn ir(&mut self) -> PyResult<String> {
        Python::with_gil(|py| {
            let builder = self.builder.as_ref(py).try_borrow()?;
            self.model.instructions = builder.instructions.first().unwrap().clone();
            get_ir_string(&self.model).map_err(PyOSError::new_err)
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

impl Qubit {
    fn id(&self) -> String {
        format!("{}{}", QUBIT_NAME, self.index)
    }
}

#[pyclass]
struct Ref(RefKind);

impl Ref {
    fn id(&self) -> String {
        let Ref(RefKind::Result { index }) = self;
        format!("{}{}", RESULT_NAME, index)
    }
}

enum RefKind {
    Result { index: u64 },
}

#[pyclass]
struct Builder {
    instructions: Vec<Vec<Instruction>>,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            instructions: vec![vec![]],
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

    fn cx(&self, control: &Qubit, target: &Qubit) {
        let controlled = Controlled::new(control.id(), target.id());
        self.add_inst(Instruction::Cx(controlled));
    }

    fn cz(&self, control: &Qubit, target: &Qubit) {
        let controlled = Controlled::new(control.id(), target.id());
        self.add_inst(Instruction::Cz(controlled));
    }

    fn h(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::H(single));
    }

    fn m(&self, qubit: &Qubit, result: &Ref) {
        let measured = Measured::new(qubit.id(), result.id());
        self.add_inst(Instruction::M(measured));
    }

    fn reset(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::Reset(single));
    }

    fn rx(&self, theta: f64, qubit: &Qubit) {
        let rotated = Rotated::new(theta, qubit.id());
        self.add_inst(Instruction::Rx(rotated));
    }

    fn ry(&self, theta: f64, qubit: &Qubit) {
        let rotated = Rotated::new(theta, qubit.id());
        self.add_inst(Instruction::Ry(rotated));
    }

    fn rz(&self, theta: f64, qubit: &Qubit) {
        let rotated = Rotated::new(theta, qubit.id());
        self.add_inst(Instruction::Rz(rotated));
    }

    fn s(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::S(single));
    }

    fn s_adj(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::SAdj(single));
    }

    fn t(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::T(single));
    }

    fn t_adj(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::TAdj(single));
    }

    fn x(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::X(single));
    }

    fn y(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::Y(single));
    }

    fn z(&self, qubit: &Qubit) {
        let single = Single::new(qubit.id());
        self.add_inst(Instruction::Z(single));
    }

    fn if_result(&self, result: &Ref, one: &PyAny, zero: &PyAny) -> PyResult<()> {
        self.push_frame();
        one.call0()?;
        let then_insts = self.pop_frame().unwrap();

        self.push_frame();
        zero.call0()?;
        let else_insts = self.pop_frame().unwrap();

        let if_inst = If {
            condition: result.id(),
            then_insts,
            else_insts,
        };

        self.add_inst(Instruction::If(if_inst));
        Ok(())
    }
}

impl BasicQisBuilder {
    fn add_inst(&self, inst: Instruction) {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).borrow_mut();
            builder.instructions.last_mut().unwrap().push(inst);
        });
    }

    fn push_frame(&self) {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).borrow_mut();
            builder.instructions.push(vec![]);
        });
    }

    fn pop_frame(&self) -> Option<Vec<Instruction>> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).borrow_mut();
            builder.instructions.pop()
        })
    }
}

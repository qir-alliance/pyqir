// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    emit::get_ir_string,
    interop::{
        ClassicalRegister, Controlled, Instruction, Measured, QuantumRegister, Rotated,
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

    fn cx(&self, control: &Qubit, target: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let controlled = Controlled::new(control.id(), target.id());
            builder.model.add_inst(Instruction::Cx(controlled));
            Ok(())
        })
    }

    fn cz(&self, control: &Qubit, target: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let controlled = Controlled::new(control.id(), target.id());
            builder.model.add_inst(Instruction::Cz(controlled));
            Ok(())
        })
    }

    fn h(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::H(single));
            Ok(())
        })
    }

    fn m(&self, qubit: &Qubit, result: &Ref) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let measured = Measured::new(qubit.id(), result.id());
            builder.model.add_inst(Instruction::M(measured));
            Ok(())
        })
    }

    fn reset(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::Reset(single));
            Ok(())
        })
    }

    fn rx(&self, theta: f64, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let rotated = Rotated::new(theta, qubit.id());
            builder.model.add_inst(Instruction::Rx(rotated));
            Ok(())
        })
    }

    fn ry(&self, theta: f64, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let rotated = Rotated::new(theta, qubit.id());
            builder.model.add_inst(Instruction::Ry(rotated));
            Ok(())
        })
    }

    fn rz(&self, theta: f64, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let rotated = Rotated::new(theta, qubit.id());
            builder.model.add_inst(Instruction::Rz(rotated));
            Ok(())
        })
    }

    fn s(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::S(single));
            Ok(())
        })
    }

    fn s_adj(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::SAdj(single));
            Ok(())
        })
    }

    fn t(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::T(single));
            Ok(())
        })
    }

    fn t_adj(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::TAdj(single));
            Ok(())
        })
    }

    fn x(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::X(single));
            Ok(())
        })
    }

    fn y(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::Y(single));
            Ok(())
        })
    }

    fn z(&self, qubit: &Qubit) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut builder = self.builder.as_ref(py).try_borrow_mut()?;
            let single = Single::new(qubit.id());
            builder.model.add_inst(Instruction::Z(single));
            Ok(())
        })
    }

    fn if_result(&self, result: &Ref, one: &PyAny, zero: &PyAny) {
        todo!()
    }
}

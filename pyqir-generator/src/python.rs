// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::PyErr;

use crate::emit::{get_bitcode_base64_string, get_ir_string, write_model_to_file};
use crate::interop::{
    ClassicalRegister, Controlled, Instruction, Measured, QuantumRegister, Rotated, SemanticModel,
    Single,
};

#[pymodule]
fn pyqir_generator(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyQIR>()?;

    Ok(())
}

#[pyclass]
pub struct PyQIR {
    pub(super) model: SemanticModel,
}

#[pymethods]
impl PyQIR {
    #[new]
    fn new(name: String) -> Self {
        PyQIR {
            model: SemanticModel::new(name),
        }
    }

    fn add_measurement(&mut self, qubit: String, target: String) -> PyResult<()> {
        log::info!("measure {} => {}", qubit, target);
        Ok(())
    }

    fn cx(&mut self, control: String, target: String) -> PyResult<()> {
        log::info!("cx {} => {}", control, target);
        let controlled = Controlled::new(control, target);
        let inst = Instruction::Cx(controlled);
        self.model.add_inst(inst);
        Ok(())
    }

    fn cz(&mut self, control: String, target: String) -> PyResult<()> {
        log::info!("cz {} => {}", control, target);
        let controlled = Controlled::new(control, target);
        let inst = Instruction::Cz(controlled);
        self.model.add_inst(inst);
        Ok(())
    }

    fn h(&mut self, qubit: String) -> PyResult<()> {
        log::info!("h => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::H(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn m(&mut self, qubit: String, target: String) -> PyResult<()> {
        log::info!("m {}[{}]", qubit, target);
        let inst = Measured::new(qubit, target);
        let inst = Instruction::M(inst);
        self.model.add_inst(inst);
        Ok(())
    }

    fn reset(&mut self, qubit: String) -> PyResult<()> {
        log::info!("reset => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::Reset(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn rx(&mut self, theta: f64, qubit: String) -> PyResult<()> {
        log::info!("rx {} => {}", qubit, theta);
        let rotated = Rotated::new(theta, qubit);
        let inst = Instruction::Rx(rotated);
        self.model.add_inst(inst);
        Ok(())
    }

    fn ry(&mut self, theta: f64, qubit: String) -> PyResult<()> {
        log::info!("ry {} => {}", qubit, theta);
        let rotated = Rotated::new(theta, qubit);
        let inst = Instruction::Ry(rotated);
        self.model.add_inst(inst);
        Ok(())
    }

    fn rz(&mut self, theta: f64, qubit: String) -> PyResult<()> {
        log::info!("rz {} => {}", qubit, theta);
        let rotated = Rotated::new(theta, qubit);
        let inst = Instruction::Rz(rotated);
        self.model.add_inst(inst);
        Ok(())
    }

    fn s(&mut self, qubit: String) -> PyResult<()> {
        log::info!("s => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::S(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn s_adj(&mut self, qubit: String) -> PyResult<()> {
        log::info!("s_adj => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::SAdj(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn t(&mut self, qubit: String) -> PyResult<()> {
        log::info!("t => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::T(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn t_adj(&mut self, qubit: String) -> PyResult<()> {
        log::info!("t_adj => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::TAdj(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn x(&mut self, qubit: String) -> PyResult<()> {
        log::info!("x => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::X(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn y(&mut self, qubit: String) -> PyResult<()> {
        log::info!("y => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::Y(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn dump_machine(&mut self) -> PyResult<()> {
        log::info!("dump_machine");
        let inst = Instruction::DumpMachine;
        self.model.add_inst(inst);
        Ok(())
    }

    fn z(&mut self, qubit: String) -> PyResult<()> {
        log::info!("z => {}", qubit);
        let single = Single::new(qubit);
        let inst = Instruction::Z(single);
        self.model.add_inst(inst);
        Ok(())
    }

    fn add_quantum_register(&mut self, name: String, size: u64) -> PyResult<()> {
        let ns = name.as_str();
        for index in 0..size {
            let register_name = format!("{}[{}]", ns, index);
            log::info!("Adding {}", register_name);
            let reg = QuantumRegister {
                name: String::from(ns),
                index,
            };
            self.model.add_reg(reg.as_register());
        }
        Ok(())
    }

    fn add_classical_register(&mut self, name: String, size: u64) -> PyResult<()> {
        let ns = name.clone();
        let reg = ClassicalRegister { name, size };
        log::info!("Adding {}({})", ns, size);
        self.model.add_reg(reg.as_register());
        Ok(())
    }

    fn write(&self, file_name: &str) -> PyResult<()> {
        if let Err(msg) = write_model_to_file(&self.model, file_name) {
            let err: PyErr = PyOSError::new_err::<String>(msg);
            return Err(err);
        }
        Ok(())
    }

    fn get_ir_string(&self) -> PyResult<String> {
        match get_ir_string(&self.model) {
            Err(msg) => {
                let err: PyErr = PyOSError::new_err::<String>(msg);
                Err(err)
            }
            Ok(ir) => Ok(ir),
        }
    }

    fn get_bitcode_base64_string(&self) -> PyResult<String> {
        match get_bitcode_base64_string(&self.model) {
            Err(msg) => {
                let err: PyErr = PyOSError::new_err::<String>(msg);
                Err(err)
            }
            Ok(ir) => Ok(ir),
        }
    }

    fn enable_logging(&self) -> PyResult<()> {
        let _ = env_logger::try_init();
        Ok(())
    }
}

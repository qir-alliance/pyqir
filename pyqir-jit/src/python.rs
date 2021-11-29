// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::interop::Instruction;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyErr;

#[pymodule]
fn pyqir_jit(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNonadaptiveJit>()?;

    Ok(())
}

#[pyclass]
pub struct PyNonadaptiveJit {}

#[pymethods]
impl PyNonadaptiveJit {
    #[new]
    fn new() -> Self {
        PyNonadaptiveJit {}
    }

    fn controlled(
        &self,
        pyobj: &PyAny,
        gate: &str,
        control: String,
        target: String,
    ) -> PyResult<()> {
        let has_gate = pyobj.hasattr(gate)?;
        if has_gate {
            let func = pyobj.getattr(gate)?;
            let args = (control, target);
            func.call1(args)?;
        }
        Ok(())
    }

    fn measured(&self, pyobj: &PyAny, gate: &str, qubit: String, target: String) -> PyResult<()> {
        let has_gate = pyobj.hasattr(gate)?;
        if has_gate {
            let func = pyobj.getattr(gate)?;
            let args = (qubit, target);
            func.call1(args)?;
        }
        Ok(())
    }

    fn single(&self, pyobj: &PyAny, gate: &str, qubit: String) -> PyResult<()> {
        let has_gate = pyobj.hasattr(gate)?;
        if has_gate {
            let func = pyobj.getattr(gate)?;
            let args = (qubit,);
            func.call1(args)?;
        }
        Ok(())
    }

    fn rotated(&self, pyobj: &PyAny, gate: &str, theta: f64, qubit: String) -> PyResult<()> {
        let has_gate = pyobj.hasattr(gate)?;
        if has_gate {
            let func = pyobj.getattr(gate)?;
            let args = (theta, qubit);
            func.call1(args)?;
        }
        Ok(())
    }

    fn finish(&self, pyobj: &PyAny, dict: &PyDict) -> PyResult<()> {
        let has_gate = pyobj.hasattr("finish")?;
        if has_gate {
            let func = pyobj.getattr("finish")?;
            let args = (dict,);
            func.call1(args)?;
        }
        Ok(())
    }

    fn eval(&self, file: String, pyobj: &PyAny) -> PyResult<()> {
        // TODO: Add entry point parameter.
        let result = crate::jit::run_module(file, None);
        if let Err(msg) = result {
            let err: PyErr = PyOSError::new_err::<String>(msg);
            return Err(err);
        }
        let gen_model = result.unwrap();
        Python::with_gil(|py| -> PyResult<()> {
            for instruction in gen_model.instructions {
                match instruction {
                    Instruction::Cx(ins) => {
                        self.controlled(pyobj, "cx", ins.control, ins.target)?
                    }
                    Instruction::Cz(ins) => {
                        self.controlled(pyobj, "cz", ins.control, ins.target)?
                    }
                    Instruction::H(ins) => self.single(pyobj, "h", ins.qubit)?,
                    Instruction::M(ins) => self.measured(pyobj, "m", ins.qubit, ins.target)?,
                    Instruction::Reset(_ins) => {
                        todo!("Not Implemented")
                    }
                    Instruction::Rx(ins) => self.rotated(pyobj, "rx", ins.theta, ins.qubit)?,
                    Instruction::Ry(ins) => self.rotated(pyobj, "ry", ins.theta, ins.qubit)?,
                    Instruction::Rz(ins) => self.rotated(pyobj, "rz", ins.theta, ins.qubit)?,
                    Instruction::S(ins) => self.single(pyobj, "s", ins.qubit)?,
                    Instruction::SAdj(ins) => self.single(pyobj, "s_adj", ins.qubit)?,
                    Instruction::T(ins) => self.single(pyobj, "t", ins.qubit)?,
                    Instruction::TAdj(ins) => self.single(pyobj, "t_adj", ins.qubit)?,
                    Instruction::X(ins) => self.single(pyobj, "x", ins.qubit)?,
                    Instruction::Y(ins) => self.single(pyobj, "y", ins.qubit)?,
                    Instruction::Z(ins) => self.single(pyobj, "z", ins.qubit)?,
                    Instruction::DumpMachine => {
                        todo!("Not Implemented")
                    }
                }
            }
            let dict = PyDict::new(py);
            dict.set_item("number_of_qubits", gen_model.qubits.len())?;
            self.finish(pyobj, dict)?;
            Ok(())
        })?;
        Ok(())
    }
}

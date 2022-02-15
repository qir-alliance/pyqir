// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{interop::Instruction, jit::run_module_file};
use pyo3::{
    exceptions::PyOSError,
    prelude::*,
    types::{PyDict, PyList},
    PyAny,
};

#[pymodule]
fn _native(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
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

    #[allow(clippy::unused_self)]
    fn eval(
        &self,
        file: &str,
        pyobj: &PyAny,
        entry_point: Option<&str>,
        result_stream: Option<&PyList>,
    ) -> PyResult<()> {
        fn controlled(pyobj: &PyAny, gate: &str, control: String, target: String) -> PyResult<()> {
            let has_gate = pyobj.hasattr(gate)?;
            if has_gate {
                let func = pyobj.getattr(gate)?;
                let args = (control, target);
                func.call1(args)?;
            }
            Ok(())
        }

        fn measured(pyobj: &PyAny, gate: &str, qubit: String, target: String) -> PyResult<()> {
            let has_gate = pyobj.hasattr(gate)?;
            if has_gate {
                let func = pyobj.getattr(gate)?;
                let args = (qubit, target);
                func.call1(args)?;
            }
            Ok(())
        }

        fn single(pyobj: &PyAny, gate: &str, qubit: String) -> PyResult<()> {
            let has_gate = pyobj.hasattr(gate)?;
            if has_gate {
                let func = pyobj.getattr(gate)?;
                let args = (qubit,);
                func.call1(args)?;
            }
            Ok(())
        }

        fn reset(pyobj: &PyAny, qubit: String) -> PyResult<()> {
            let has_gate = pyobj.hasattr("reset")?;
            if has_gate {
                let func = pyobj.getattr("reset")?;
                let args = (qubit,);
                func.call1(args)?;
            }
            Ok(())
        }

        fn rotated(pyobj: &PyAny, gate: &str, theta: f64, qubit: String) -> PyResult<()> {
            let has_gate = pyobj.hasattr(gate)?;
            if has_gate {
                let func = pyobj.getattr(gate)?;
                let args = (theta, qubit);
                func.call1(args)?;
            }
            Ok(())
        }

        fn finish(pyobj: &PyAny, dict: &PyDict) -> PyResult<()> {
            let has_gate = pyobj.hasattr("finish")?;
            if has_gate {
                let func = pyobj.getattr("finish")?;
                let args = (dict,);
                func.call1(args)?;
            }
            Ok(())
        }

        let result_vec = result_stream
            .map(|rs| rs.iter().map(PyAny::extract::<bool>).collect())
            .transpose()?;

        let gen_model =
            run_module_file(file, entry_point, result_vec).map_err(PyOSError::new_err)?;

        Python::with_gil(|py| -> PyResult<()> {
            let mut current_register = 0;
            for instruction in gen_model.instructions {
                match instruction {
                    Instruction::Cx(ins) => controlled(pyobj, "cx", ins.control, ins.target)?,
                    Instruction::Cz(ins) => controlled(pyobj, "cz", ins.control, ins.target)?,
                    Instruction::H(ins) => single(pyobj, "h", ins.qubit)?,
                    Instruction::M(ins) => {
                        measured(pyobj, "m", ins.qubit, current_register.to_string())?;
                        current_register += 1;
                    }
                    Instruction::Reset(ins) => reset(pyobj, ins.qubit)?,
                    Instruction::Rx(ins) => rotated(pyobj, "rx", ins.theta, ins.qubit)?,
                    Instruction::Ry(ins) => rotated(pyobj, "ry", ins.theta, ins.qubit)?,
                    Instruction::Rz(ins) => rotated(pyobj, "rz", ins.theta, ins.qubit)?,
                    Instruction::S(ins) => single(pyobj, "s", ins.qubit)?,
                    Instruction::SAdj(ins) => single(pyobj, "s_adj", ins.qubit)?,
                    Instruction::T(ins) => single(pyobj, "t", ins.qubit)?,
                    Instruction::TAdj(ins) => single(pyobj, "t_adj", ins.qubit)?,
                    Instruction::X(ins) => single(pyobj, "x", ins.qubit)?,
                    Instruction::Y(ins) => single(pyobj, "y", ins.qubit)?,
                    Instruction::Z(ins) => single(pyobj, "z", ins.qubit)?,
                }
            }
            let dict = PyDict::new(py);
            dict.set_item("number_of_qubits", gen_model.qubits.len())?;
            finish(pyobj, dict)?;
            Ok(())
        })?;
        Ok(())
    }
}

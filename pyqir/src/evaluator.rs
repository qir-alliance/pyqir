// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pyo3::{
    exceptions::PyOSError,
    prelude::*,
    types::{PyDict, PyList, PyTuple},
    PyAny,
};
use qirlib::evaluation::jit::run_module_file;

#[pyclass]
pub(crate) struct PyNonadaptiveJit;

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
                let (name, args) = match instruction.name.to_lowercase().as_str() {
                    "m" => {
                        if instruction.args.len() == 1 {
                            let v = current_register.to_string();
                            current_register += 1;
                            ("m".to_string(), vec![instruction.args[0].clone(), v])
                        } else {
                            (
                                "mz".to_string(),
                                vec![instruction.args[0].clone(), instruction.args[1].clone()],
                            )
                        }
                    }
                    _ => (instruction.name, instruction.args),
                };
                let has_inst = pyobj.hasattr(name.as_str())?;
                if has_inst {
                    let func = pyobj.getattr(name.as_str())?;
                    let call_args = PyTuple::new(py, args);
                    func.call1(call_args)?;
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

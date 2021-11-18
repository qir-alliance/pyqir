# pyqir_generator

The `pyqir_generator` package provides the ability to generate [QIR](https://github.com/microsoft/qsharp-language/tree/main/Specifications/QIR#quantum-intermediate-representation-qir) using a [builder](https://en.wikipedia.org/wiki/Builder_pattern) API.

For example, this operation sets two qubits in superposition and returns the result of 
measuring them. The full output is sent to the terminal:

```python
from pyqir_generator import QirBuilder

"""
Whether the two qubits should be entangled.
    If "true", the two qubits are entangled such that the state of both
    is one of the Bell states: | x 〉| y 〉 = 1/sqrt(2) [|0〉|0〉 + |1〉|1〉]
    If "false", the two qubits are entangled such that the state is:
    | x 〉| y 〉 = 1/4 [|0〉|0〉 + |0〉|1〉 + |1〉|0〉 + |1〉|1〉]
"""
entangle: bool = True

builder = QirBuilder("Bell")
builder.add_quantum_register("qubit", 2)
builder.add_classical_register("output", 2)
builder.h("qubit0")

if(entangle):
    builder.cx("qubit0", "qubit1")
else:
    builder.h("qubit1")

builder.m("qubit0", "output0")
builder.m("qubit1", "output1")

print(builder.get_ir_string())
```

The corresponding piece in the QIR output will contain the generated function:

```
define internal %Array* @QuantumApplication__Run__body() {
entry:
  %qubit0 = call %Qubit* @__quantum__rt__qubit_allocate()
  %qubit1 = call %Qubit* @__quantum__rt__qubit_allocate()
  %results = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 1)
  %output = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 2)
  %output_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 0)
  %output_result_0 = bitcast i8* %output_0_raw to %Result**
  %zero_0 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_0, i32 1)
  store %Result* %zero_0, %Result** %output_result_0, align 8
  %output_1_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 1)
  %output_result_1 = bitcast i8* %output_1_raw to %Result**
  %zero_1 = call %Result* @__quantum__rt__result_get_zero()
  call void @__quantum__rt__result_update_reference_count(%Result* %zero_1, i32 1)
  store %Result* %zero_1, %Result** %output_result_1, align 8
  %results_result_tmp_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %results, i64 0)
  %results_result_tmp_result_0 = bitcast i8* %results_result_tmp_0_raw to %Array**
  store %Array* %output, %Array** %results_result_tmp_result_0, align 8
  call void @Microsoft__Quantum__Intrinsic__H__body(%Qubit* %qubit0)
  %__controlQubits__ = call %Array* @__quantum__rt__array_create_1d(i32 8, i64 1)
  %__controlQubits__0_result_tmp_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %__controlQubits__, i64 0)
  %__controlQubits__0_result_tmp_result_0 = bitcast i8* %__controlQubits__0_result_tmp_0_raw to %Qubit**
  store %Qubit* %qubit0, %Qubit** %__controlQubits__0_result_tmp_result_0, align 8
  call void @Microsoft__Quantum__Intrinsic__X__ctl(%Array* %__controlQubits__, %Qubit* %qubit1)
  call void @__quantum__rt__array_update_reference_count(%Array* %__controlQubits__, i32 -1)
  %measurement = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit0)
  %output0_0_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 0)
  %output0_result_0 = bitcast i8* %output0_0_raw to %Result**
  %existing_value = load %Result*, %Result** %output0_result_0, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement, i32 1)
  store %Result* %measurement, %Result** %output0_result_0, align 8
  %measurement1 = call %Result* @Microsoft__Quantum__Intrinsic__M__body(%Qubit* %qubit1)
  %output1_1_raw = call i8* @__quantum__rt__array_get_element_ptr_1d(%Array* %output, i64 1)
  %output1_result_1 = bitcast i8* %output1_1_raw to %Result**
  %existing_value2 = load %Result*, %Result** %output1_result_1, align 8
  call void @__quantum__rt__result_update_reference_count(%Result* %existing_value2, i32 -1)
  call void @__quantum__rt__result_update_reference_count(%Result* %measurement1, i32 1)
  store %Result* %measurement1, %Result** %output1_result_1, align 8
  call void @__quantum__rt__qubit_release(%Qubit* %qubit1)
  call void @__quantum__rt__qubit_release(%Qubit* %qubit0)
  ret %Array* %results
}
```

## Building and Testing

To build this package, first install `maturin`:

```shell
pip install maturin
```

To build and test use `maturin develop`:

```shell
pip install -r requirements-dev.txt
maturin develop && pytest
```

Alternatively, install tox and run the tests inside an isolated environment:

```shell
tox -e py
```
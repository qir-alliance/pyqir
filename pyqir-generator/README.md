# pyqir_generator

The `pyqir_generator` package provides the ability to generate
[QIR](https://github.com/qir-alliance/qir-spec) using a Python API.

It is intended to be used by code automating translation processes enabling the
conversion in some format to QIR via Python; i.e., this is a low level API
intended to be used as a bridge to existing Python frameworks enabling the
generation of QIR rather than directly consumed by an end-user. It is **not**
intended to be used as a framework for algorithm and application development.

## Examples

There are [generator
examples](https://github.com/qir-alliance/pyqir/tree/main/examples/generator) in
the repository.

Let's look at a short example. The following code creates QIR for an create Bell
pair before measuring each qubit and returning the result. The unoptimized QIR
is displayed in the terminal when executed:

```python
from pyqir_generator import QirBuilder

builder = QirBuilder("Bell")
builder.add_quantum_register("qubit", 2)
builder.add_classical_register("output", 2)

builder.h("qubit0")
builder.cx("qubit0", "qubit1")

builder.m("qubit0", "output0")
builder.m("qubit1", "output1")

print(builder.get_ir_string())
```

The corresponding piece in the QIR output will contain the generated function:

```llvm
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

See [Building](https://qir-alliance.github.io/pyqir/development-guide/building.html)

## Current Limitations

- Support for emitting classical computations and control flow constructs is not
  yet implemented, see also [this
  issue](https://github.com/qir-alliance/pyqir/issues/2)
- Using qubit/register names in gate calls that haven't been defined will cause
  an error during generation.

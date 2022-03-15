# pyqir-generator

The `pyqir-generator` package provides the ability to generate
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
from pyqir.generator import BasicQisBuilder, SimpleModule

module = SimpleModule("Bell", num_qubits=2, num_results=2)
qis = BasicQisBuilder(module.builder)

qis.h(module.qubits[0])
qis.cx(module.qubits[0], module.qubits[1])

qis.m(module.qubits[0], module.results[0])
qis.m(module.qubits[1], module.results[1])

print(module.ir())
```

The QIR output will look like:

```llvm
; ModuleID = 'Bell'
source_filename = "Bell"

%Qubit = type opaque
%Result = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__h__body(%Qubit* null)
  call void @__quantum__qis__cnot__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))
  %result0 = call %Result* @__quantum__qis__m__body(%Qubit* null)
  %result1 = call %Result* @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  ret void
}

declare void @__quantum__qis__h__body(%Qubit*)

declare void @__quantum__qis__cnot__body(%Qubit*, %Qubit*)

declare %Result* @__quantum__qis__m__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="2" }
```

## Building and Testing

See [Building](https://qir-alliance.github.io/pyqir/development-guide/building.html)

## Current Limitations

- Classical computation and control flow is not yet fully supported.
- Only branching based on measurement results is currently possible.
- See [issue #2: Support control flow and classical computation in PyQIR Generator](https://github.com/qir-alliance/pyqir/issues/2).

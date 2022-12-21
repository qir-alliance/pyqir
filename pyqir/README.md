# PyQIR

The `pyqir` package provides the ability to generate
[QIR](https://github.com/qir-alliance/qir-spec) as well as an easy way to
parse and analyze QIR.

QIR generation is intended to be used by code automating translation processes
enabling the conversion in some format to QIR via Python; i.e., this is a
low-level API intended to be used as a bridge to existing Python frameworks
enabling the generation of QIR rather than directly consumed by an end-user. It
is **not** intended to be used as a framework for algorithm and application
development.

## Installation

The package is released on PyPI and can be installed via pip:

```bash
pip install pyqir
```

## Examples

PyQIR examples can be found in this repository's [examples folder](../examples).

### Generation

The following code creates QIR for a Bell pair before measuring each qubit and
returning the result. The unoptimized QIR is displayed in the terminal when
executed:

```python
from pyqir.generator import BasicQisBuilder, SimpleModule

module = SimpleModule("Bell", num_qubits=2, num_results=2)
qis = BasicQisBuilder(module.builder)

qis.h(module.qubits[0])
qis.cx(module.qubits[0], module.qubits[1])

qis.mz(module.qubits[0], module.results[0])
qis.mz(module.qubits[1], module.results[1])

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
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)
  call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
  ret void
}

declare void @__quantum__qis__h__body(%Qubit*)

declare void @__quantum__qis__cnot__body(%Qubit*, %Qubit*)

declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly)

attributes #0 = { "EntryPoint" "requiredQubits"="2" "requiredResults"="2" }
```

## Contributing

There are many ways in which you can contribute to PyQIR, whether by
contributing a feature or by engaging in discussions; we value contributions in
all shapes and sizes! We refer to [this document](../CONTRIBUTING.md) for
guidelines and ideas for how you can get involved.

Contributing a pull request to this repo requires to agree to a [Contributor
License Agreement
(CLA)](https://en.wikipedia.org/wiki/Contributor_License_Agreement) declaring
that you have the right to, and actually do, grant us the rights to use your
contribution. A CLA-bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately. Simply follow the
instructions provided by the bot. You will only need to do this once.

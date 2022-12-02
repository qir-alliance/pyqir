# PyQIR

The `pyqir` package provides the ability to generate
[QIR](https://github.com/qir-alliance/qir-spec) as well as an easy way to
execute generated QIR.

QIR generation is intended to be used by code automating translation processes
enabling the conversion in some format to QIR via Python; i.e., this is a
low-level API intended to be used as a bridge to existing Python frameworks
enabling the generation of QIR rather than directly consumed by an end-user. It
is **not** intended to be used as a framework for algorithm and application
development.

QIR evaluation can be used for:

1. easily testing and experimenting with QIR code
2. connecting it to low-level Python-based lab software such as e.g.
   [QCoDeS](https://qcodes.github.io/Qcodes/examples/15_minutes_to_QCoDeS.html#Introduction)

It contains the necessary
[just-in-time compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
infrastructure as well as an extensibility mechanism to define what actions to
perform when a gate is applied in Python. Right now the evaluator does not have
a full runtime environment and can JIT QIR produced by the pyqir-generator, but
cannot use any external function calls.

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

### Evaluation

Let's look at how to log the gate sequence for the
[Bernstein-Vazirani](../examples/bernstein_vazirani.py) example.

We can evaluate the [generated bitcode](../examples/bernstein_vazirani.bc) with
`NonadaptiveEvaluator` and `GateLogger` to print out a simple log of the quantum
application:

```python
from pyqir.evaluator import NonadaptiveEvaluator, GateLogger

from pathlib import Path
import os

path = Path(__file__).parent
file = os.path.join(path, "bernstein_vazirani.bc")

evaluator = NonadaptiveEvaluator()
logger = GateLogger()

evaluator.eval(file, logger)

print("# output from GateLogger")
logger.print()
```

This would generate the following output:

```text
# output from GateLogger
qubits[9]
out[9]
x qubit[8]
h qubit[0]
h qubit[1]
h qubit[2]
h qubit[3]
h qubit[4]
h qubit[5]
h qubit[6]
h qubit[7]
h qubit[8]
cx qubit[2], qubit[8]
cx qubit[3], qubit[8]
h qubit[0]
h qubit[1]
h qubit[2]
h qubit[3]
h qubit[4]
h qubit[5]
h qubit[6]
h qubit[7]
measure qubits[0] -> out[0]
measure qubits[1] -> out[1]
measure qubits[2] -> out[2]
measure qubits[3] -> out[3]
measure qubits[4] -> out[4]
measure qubits[5] -> out[5]
measure qubits[6] -> out[6]
measure qubits[7] -> out[7]
measure qubits[8] -> out[8]
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

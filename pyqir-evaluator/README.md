# pyqir-evaluator

The `pyqir-evaluator` package provides an easy way to execute generated QIR for the
purpose of:

1. easily testing and experimenting with QIR code
2. connecting it to low-level Python-based lab software such as e.g.
   [QCoDeS](https://qcodes.github.io/Qcodes/examples/15_minutes_to_QCoDeS.html#Introduction)

It contains the necessary [just-in-time
compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
infrastructure as well as an extensibility mechanism to define what actions to
perform when a gate is applied in Python. Right now the evaluator does not have
a full runtime environment and can JIT QIR produced by the pyqir-generator, but
cannot use any external function calls.

## Installation

The package is released on PyPI and can be installed via pip:

```bash
pip install pyqir-evaluator
```

## Examples

There are [evaluator
examples](https://github.com/qir-alliance/pyqir/tree/main/examples/evaluator) in the
repository.

Let's look at how to log the gate sequence for the [Bernstein-Vazirani](https://github.com/qir-alliance/pyqir/tree/main/examples/evaluator/bernstein_vazirani.py) example.

We can evaluate the [generated
bitcode](https://github.com/qir-alliance/pyqir/tree/main/examples/evaluator/bernstein_vazirani.bc)
  with `NonadaptiveEvaluator` and `GateLogger` to print out a simple log of the
  quantum application:

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
all shapes and sizes! We refer to [this document](https://github.com/qir-alliance/pyqir/blob/main/CONTRIBUTING.md) for
guidelines and ideas for how you can get involved.

Contributing a pull request to this repo requires to agree to a [Contributor
License Agreement
(CLA)](https://en.wikipedia.org/wiki/Contributor_License_Agreement) declaring
that you have the right to, and actually do, grant us the rights to use your
contribution. A CLA-bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately. Simply follow the
instructions provided by the bot. You will only need to do this once.

## Building and Testing

See [Building](https://qir-alliance.github.io/pyqir/development-guide/building.html).

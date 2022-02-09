# pyqir_jit

The `pyqir_jit` package provides an easy way to execute generated QIR for the
purpose of

1. easily testing and experimenting with QIR code
2. connecting it to low-level Python-based lab software such as e.g.
   [QCoDeS](https://qcodes.github.io/Qcodes/examples/15_minutes_to_QCoDeS.html#Introduction)

It contains the necessary [just-in-time
compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
infrastructure as well an extensibility mechanism to define what actions to
perform when a gate is applied in Python.

## Examples

There are [JIT
examples](https://github.com/qir-alliance/pyqir/tree/main/examples/jit) in the
repository.

Let's look at how to log the gate sequence for the following example:

- [Bernstein-Vazirani](https://github.com/qir-alliance/pyqir/tree/main/examples/jit/bernstein_vazirani.py)
  We can evaluate the [generated
  bitcode](https://github.com/qir-alliance/pyqir/tree/main/examples/jit/bernstein_vazirani.bc)
  with the `NonadaptiveJit`, and `GateLogger` to print out a simple log of the
  quantum application.

```python
from pyqir_jit import NonadaptiveJit, GateLogger

from pathlib import Path
import os

path = Path(__file__).parent
file = os.path.join(path, "bernstein_vazirani.bc")

jit = NonadaptiveJit()
logger = GateLogger()

print("# output from NonadaptiveJit returning the uninitialized output")
jit.eval(file, logger)

print("# output from GateLogger")
logger.print()
```

Would generate the output:

```text
# output from NonadaptiveJit returning the uninitialized output
[[Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero]]
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

## Building and Testing

See [Building](https://qir-alliance.github.io/pyqir/development-guide/building.html)

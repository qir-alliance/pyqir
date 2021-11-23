# pyqir-jit

## Examples

There are JIT examples in the repository:
- [Bernstein-Vazirani](../examples/jit/bernstein_vazirani.py) ([Bernstein-Vazirani QIR](../examples/jit/bernstein_vazirani.ll))

Let's look at how to log the gate sequence for the following example:
- [Bernstein-Vazirani](../examples/jit/bernstein_vazirani.py)
 We can evaluate the [generated QIR](../examples/jit/bernstein_vazirani.ll) with the `NonadaptiveJit`, and `GateLogger` to print out a simple log of the quantum application.

```python
from pyqir_jit import NonadaptiveJit, GateLogger

from pathlib import Path
import os

path = Path(__file__).parent
file = os.path.join(path, "bernstein_vazirani.ll")

jit = NonadaptiveJit()
logger = GateLogger()

print("# output from NonadaptiveJit returning the uninitialized output")
jit.eval(file, logger)

print("# output from GateLogger")
logger.print()
```

Would generate the output:

```
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

See [Building](../docs/building.md)

## Limitations

- QIR entrypoint for JIT must be named `QuantumApplication__Run`
- QIR must contain the defined runtime in [module.ll](../qirlib/src/module.ll)

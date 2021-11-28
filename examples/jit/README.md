# Examples: pyqir_jit

The 'pyqir_jit' package provides an easy way to execute generated QIR for the
purpose of
1. easily testing and experimenting with QIR code
2. connecting it to low-level Python-based lab software such as e.g.
   [QCoDes.](https://qcodes.github.io/Qcodes/user/intro.html)

It contains the necessary [just-in-time
compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
infrastructure as well an extensibility mechanism to define what actions to
perform when a gate is applied in Python.

This folder contains the following examples for how to use the `pyqir_jit`
package:

- **Bernstein-Vazirani example**: <br/>
This example shows how to log the executed gate sequence
for a quantum program compiled to [LLVM bitcode](https://www.llvm.org/docs/BitCodeFormat.html).
It consists of a [Python
program](https://github.com/qir-alliance/pyqir/tree/main/examples/jit/bernstein_vazirani.py)
  that loads the [compiled
  bitcode](https://github.com/qir-alliance/pyqir/tree/main/examples/jit/bernstein_vazirani.bc)
  and then uses the `NonadaptiveJit`, and a custom `GateLogger` to print out a
  simple log of the quantum gates applied during execution.

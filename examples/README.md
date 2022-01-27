# PyQIR Examples

PyQIR is a set of APIs for generating, parsing, and evaluating [Quantum
Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec).

- [**Emitting
  QIR**](https://github.com/qir-alliance/pyqir/tree/main/examples/generator):
  <br/>
  The
  [`pyqir_generator`](https://github.com/qir-alliance/pyqir/tree/main/pyqir-generator)
  package provides a Python API for generating QIR. It is intended to easily
  integrate the QIR toolchain into existing Python-based frontends.

- [**Executing
  QIR**](https://github.com/qir-alliance/pyqir/tree/main/examples/jit): <br/>
  The [`pyqir-jit`](https://github.com/qir-alliance/pyqir/tree/main/pyqir-jit)
  package provides an easy way to execute generated QIR. It contains the
  necessary [just-in-time
  compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
  infrastructure as well an extensibility mechanism to define what actions to
  perform when a gate is applied in Python.

- **Analyzing QIR** (coming soon): <br/>
  The
  [`pyqir-parser`](https://github.com/qir-alliance/pyqir/tree/main/pyqir-parser)
  package provides a Python API for loading QIR for basic analysis and
  transformation. For more advanced scenarios, we recommend taking a look at the
  LLVM-based infrastructure provide by the [QAT
  tool](https://qir-alliance.github.io/qat/).

## Installation

For more information about how to install the PyQIR packages to run the
examples, see the
[docs](https://github.com/qir-alliance/pyqir/blob/main/getting-started/installing.md).

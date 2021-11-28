# PyQIR

PyQIR is a set of APIs for generating, parsing, and evaluating [Quantum
Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec). It
consists of the following components:

- [**pyqir_generator**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-generator)
  [[examples](https://github.com/qir-alliance/pyqir/tree/main/examples/generator)]:
  <br/>
  This package provides a Python API for generating QIR
  ([bitcode](https://www.llvm.org/docs/BitCodeFormat.html) and
  [IR](https://llvm.org/docs/LangRef.html)). It is intended to easily integrate
  the QIR toolchain into existing Python-based frontends.

- [**pyqir-jit**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-jit)
  [[examples](https://github.com/qir-alliance/pyqir/tree/main/examples/jit)]:
  <br/>
  This package provides an easy way to execute generated QIR. It contains the
  necessary [just-in-time
  compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
  infrastructure as well an extensibility mechanism to define what actions to
  perform when a gate is applied in Python.

- [**pyqir-parser**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-parser):
  <br/>
  This package provides a Python API for loading QIR for basic analysis and
  transformation. For more advanced scenarios, we recommend taking a look at the
  LLVM-based infrastructure provide by the [QAT
  tool](https://qir-alliance.github.io/qat/).

This repository furthermore contains the
[qirlib](https://github.com/qir-alliance/pyqir/tree/main/qirlib); a Rust library
wrapping [LLVM](https://llvm.org/) libraries for working with QIR that is used
by the above Python packages.

## Documentation

- [Installing
  PyQIR](https://github.com/qir-alliance/pyqir/blob/main/docs/installing.md)
- [Building PyQIR from
  source](https://github.com/qir-alliance/pyqir/blob/main/docs/building.md)
- [Compatibility](https://github.com/qir-alliance/pyqir/blob/main/docs/compatibility.md)

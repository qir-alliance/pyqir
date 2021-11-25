# PyQIR

PyQIR is a set of APIs for generating, parsing, and evaluating [Quantum Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec).

- [pyqir-generator](https://github.com/qir-alliance/pyqir/tree/main/pyqir-generator): Python API for generating QIR ([bitcode](https://www.llvm.org/docs/BitCodeFormat.html#id10) and [IR](https://llvm.org/docs/LangRef.html)).
  - [Examples](https://github.com/qir-alliance/pyqir/tree/main/examples/generator)
- [pyqir-jit](https://github.com/qir-alliance/pyqir/tree/main/pyqir-jit): Python API for evaluating QIR using [JIT compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation).
  - [Examples](https://github.com/qir-alliance/pyqir/tree/main/examples/jit)
- [pyqir-parser](https://github.com/qir-alliance/pyqir/tree/main/pyqir-parser): Python API for parsing QIR into an object model for analysis.
- [qirlib](https://github.com/qir-alliance/pyqir/tree/main/qirlib): Rust library wrapping [LLVM](https://llvm.org/) libraries for working with QIR.

## Documentation

- [Installing PyQIR](https://github.com/qir-alliance/pyqir/blob/main/docs/installing.md)
- [Building PyQIR from source](https://github.com/qir-alliance/pyqir/blob/main/docs/building.md)
- [Compatibility](https://github.com/qir-alliance/pyqir/blob/main/docs/compatibility.md)

## Installation
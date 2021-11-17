# PyQIR

PyQIR is a set of APIs for generating, parsing, and evaluating [Quantum Intermediate Representation (QIR)](https://github.com/microsoft/qsharp-language/tree/main/Specifications/QIR#quantum-intermediate-representation-qir).

- [pyqir-generator](./pyqir-generator/README.md) : Python API for generating QIR ([bitcode](https://www.llvm.org/docs/BitCodeFormat.html#id10) and [IR](https://llvm.org/docs/LangRef.html)).
- [pyqir-jit](./pyqir-jit/README.md) : Python API for evaluating QIR using [JIT compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation).
- [pyqir-parser](./pyqir-parser/README.md) : Python API for parsing QIR into an object model for analysis.
- [qirlib](./qirlib/README.md): Rust library wrapping [LLVM](https://llvm.org/) libraries for working with QIR.

## Documentation


## Installation
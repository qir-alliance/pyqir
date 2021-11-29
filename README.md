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

## Feedback

If you have feedback about the content in this repository, please let us know by
filing a [new issue](https://github.com/qir-alliance/pyqir/issues/new)!

## Contributing

There are many ways in which you can contribute to PyQIR, whether by
contributing a feature or by engaging in discussions; we value contributions in
all shapes and sizes! We refer to [this document](CONTRIBUTING.md) for
guidelines and ideas for how you can get involved.

Contributing a pull request to this repo requires to agree to a [Contributor
License Agreement
(CLA)](https://en.wikipedia.org/wiki/Contributor_License_Agreement) declaring
that you have the right to, and actually do, grant us the rights to use your
contribution. We are still working on setting up a suitable CLA-bot to automate
this process. A CLA-bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately. Once it is set up, simply follow the
instructions provided by the bot. You will only need to do this once.

## Code of Conduct

This project has adopted the community covenant [Code of
Conduct](https://github.com/qir-alliance/.github/blob/main/Code_of_Conduct.md#contributor-covenant-code-of-conduct).
Please contact [qiralliance@mail.com](mailto:qiralliance@mail.com) for Code of
Conduct issues or inquires.

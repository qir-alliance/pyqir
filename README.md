# iqm_pyqir

iqm_pyqir is a set of APIs for generating, parsing, and evaluating [Quantum Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec).
This repository contains:

- [**iqm_pyqir**](iqm_pyqir) ([examples](examples))

  This package provides a Python API for parsing and generating QIR.
  It is intended to easily integrate the QIR toolchain into existing Python-based frontends.
  For more advanced scenarios, we recommend taking a look at the LLVM-based infrastructure provided by [QAT](https://qir-alliance.github.io/qat/).

  It also provides an easy way to run generated QIR. It contains the necessary [just-in-time compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation) infrastructure as well an extensibility mechanism to define what actions to perform when a gate is applied in Python.

- [**qirlib**](qirlib)

  This is a Rust library wrapping [LLVM](https://llvm.org/) for working with QIR that is used by iqm_pyqir.

## Documentation

- [Getting Started](https://www.qir-alliance.org/iqm_pyqir/)
- [Building from Source](https://www.qir-alliance.org/iqm_pyqir/building.html)

## Feedback

If you have feedback about the content in this repository, please let us know by
filing a [new issue](https://github.com/qir-alliance/iqm_pyqir/issues/new)!

## Contributing

There are many ways in which you can contribute to iqm_pyqir, whether by
contributing a feature or by engaging in discussions; we value contributions in
all shapes and sizes! We refer to [this document](CONTRIBUTING.md) for
guidelines and ideas for how you can get involved.

Contributing a pull request to this repo requires to agree to a [Contributor
License Agreement
(CLA)](https://en.wikipedia.org/wiki/Contributor_License_Agreement) declaring
that you have the right to, and actually do, grant us the rights to use your
contribution. A CLA-bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately. Simply follow the
instructions provided by the bot. You will only need to do this once.

## Code of Conduct

This project has adopted the community covenant [Code of
Conduct](https://github.com/qir-alliance/.github/blob/main/Code_of_Conduct.md#contributor-covenant-code-of-conduct).
Please contact [qiralliance@mail.com](mailto:qiralliance@mail.com) for Code of
Conduct issues or inquiries.

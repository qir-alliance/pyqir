# pyqir-parser

The pyqir-parser provides utilities for parsing QIR from bitcode files into convenience objects in Python.
It includes mechanisms for walking the functions and blocks in a given QIR program, with additional support for
QIR-specific conventions like getting static qubit identifiers or measurement result identifiers. It is not designed
for output of QIR, and does not provide any functionality for modifying or transforming the parsed QIR.

## Installation

The package is released on PyPI and can be installed via pip:

```bash
pip install pyqir-parser
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

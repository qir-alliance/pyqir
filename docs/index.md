# PyQIR

PyQIR makes it easy to work with [Quantum Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec) using Python.
You can use it to parse and evaluate existing QIR modules or generate new ones.

```{toctree}
:maxdepth: 1
:hidden:

upgrade
api/index
building
```

## Installation

PyQIR can be installed using pip:

```sh
pip install pyqir
```

You can also {doc}`build PyQIR from source <building>`.

## Supported Systems

PyQIR runs on most 64-bit x86 systems with Python 3.7 or newer.

There are two tiers of support.
Tier 1 systems are guaranteed to work and we publish official binaries for them.
Tier 2 systems are likely to work and may be compatible with the binaries published for a tier 1 system, but may need to be built from source instead.

### Tier 1

- Ubuntu 20.04
- Debian 9
- macOS 11
- Windows Server 2019

### Tier 2

- Ubuntu 22.04
- Debian 11, 10
- macOS 12, 10.7-10.15
- Windows 11, 10

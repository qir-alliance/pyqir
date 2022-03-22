# Installing PyQIR

Releases are published to both Python Package Index (PyPI) and GitHub releases. To install via pip:

```bash
pip install pyqir-generator
pip install pyqir-evaluator
pip install pyqir-parser
```

### Install the Packages

Alternatively, you can run the installation script which will install them all for you:

- Windows
  - Command Prompt: `install.cmd`
  - PowerShell: `install.ps1`
- Linux
  - Bash/sh: `install.sh`
  - PowerShell: `install.ps1`

## Supported Platforms

For more information about what platforms are supported, please see
{doc}`Compatibility </development-guide/compatibility>`. Alternatively, instructions for how to build
PyQIR from source can be found at {doc}`/development-guide/building`.

## Uninstalling the Packages

To uninstall the PyQIR packages, run

```bash
python -m pip uninstall -y pyqir-generator pyqir-parser pyqir-evaluator
```

## Troubleshooting

### Error when running the examples

- `ModuleNotFoundError: No module named '(pyqir|pyqir.generator|pyqir.parser|pyqir.evaluator)'`:
  Install the library by running one of the `install.(sh|ps1|cmd)` scripts.

### Other errors

- `Python was not found`: If you are running `python3` in a terminal window, run
  `python --version`. If the output starts `Python 3` then use `python` instead.
  This can happen on some platforms that don't create an alias.

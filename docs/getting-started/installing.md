# Installing PyQIR

Currently, pre-built packages are available in the form of GitHub releases only.
We are working on making them available via other common distribution channels.

## Prerequisites

- [Python 3.6+](https://www.python.org)

## Installing PyQIR from GitHub Releases

### Install the GitHub CLI

To install the pre-built package from GitHub, please install the [GitHub
CLI](https://cli.github.com/) either directly from the webpage, or alternatively
use the following [conda](https://docs.conda.io/en/latest/) command from within
your conda environment:

```bash
conda install -c conda-forge gh
```

### Download the Release

From within the repository root folder, run the following command to download
the desired version for your platform, e.g. for `v0.2.0a1`:

- on Linux:

  ```bash
  gh release download v0.2.0a1 -D wheelhouse -R qir-alliance/pyqir --pattern "*-manylinux*_x86_64.whl"
  ```

- on Mac OS:

  ```bash
  gh release download v0.2.0a1 -D wheelhouse -R qir-alliance/pyqir --pattern "*-macosx_*_x86_64.whl"
  ```

- on Windows:

  ```bash
  gh release download v0.2.0a1 -D wheelhouse -R qir-alliance/pyqir --pattern "*-win_amd64.whl"
  ```

### Install the Packages

Then run the installation script:

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
python -m pip uninstall -y pyqir-generator pyqir-parser pyqir-jit
```

## Troubleshooting

### Error when running the examples

- `ModuleNotFoundError: No module named '(pyqir|pyqir.generator|pyqir.parser|pyqir.jit)'`:
  Install the library by running one of the `install.(sh|ps1|cmd)` scripts.

### Other errors

- `Python was not found`: If you are running `python3` in a terminal window, run
  `python --version`. If the output starts `Python 3` then use `python` instead.
  This can happen on some platforms that don't create an alias.

# Building from Source

## Local Environment

### Requirements

- [Rust 1.64+](https://rustup.rs/)
- [Python 3.7+](https://www.python.org)
- [PowerShell 7+
  (Core)](https://github.com/powershell/powershell#get-powershell)
- [LLVM/Clang 13.0.1](https://llvm.org/) - See [](#installing-llvm)
- If compiling LLVM from source:
  - [CMake 3.10+](https://github.com/Kitware/CMake/releases/tag/v3.10.3)
  - [Ninja 1.10.0+](https://ninja-build.org/)

### Optional

- If building LLVM from source, either of these is highly recommended:
  - [sccache](https://github.com/mozilla/sccache)
  - [ccache](https://github.com/ccache/ccache)

### Linux (Ubuntu)

Install python and libs:

```bash
sudo apt-get install -y --no-install-recommends python3-dev python3-pip
python3 -m pip install --user -U pip
python3 -m pip install --user maturin~=1.7.8
```

Install Rust from [rustup](https://rustup.rs/).

### Windows

Install Python 3.7+ from one of the following and make sure it is added to the
path.

- [Miniconda](https://docs.conda.io/en/latest/miniconda.html#latest-miniconda-installer-links)
- [Python.org](https://www.python.org/downloads/)

In a command prompt:

```bash
python -m pip install --user maturin~=1.7.8
```

Install Rust from [rustup](https://rustup.rs/).

### MacOS

Install Python 3.7+ from [Python.org](https://www.python.org/downloads/macos/).

or brew:

```bash
brew install 'python@3.9'
python -m pip install --user maturin~=1.7.8
```

Install Rust from [rustup](https://rustup.rs/).

### Installing Clang and Ninja

You can install Clang manually:

- Linux (Ubuntu 22.04)

  ```bash
  apt-get update
  apt-get install -y clang-13 lldb-13 lld-11 clangd-13
  apt-get install -y --no-install-recommends ninja-build clang-tidy-13 build-essential
  ```

- Windows
  - Download and install the `LLVM-13.0.1-win64.exe` from the [13.0.1
    Release](https://github.com/llvm/llvm-project/releases/tag/llvmorg-13.0.1)
    page.
  - This package only contains the Clang components. There is no package that
    contains Clang and LLVM.
- MacOS
  - Should be preinstalled.

### Installing LLVM

The build scripts will automatically download an LLVM toolchain which is
detailed in the [](#development) section. The build installs the toolchain to
`target/llvm-<version>`.

## Development

To initialize your local environment and build
the solution, run

```bash
./build.ps1
```

This will compile `qirlib` and its dependencies with the appropriate environment
variables set for their build scripts. After this is run, the build commands
below can be used instead of `build.ps1`.

The [](#environment-variables) section details ways to change this behavior.

Within each project folder, the build can be run specifically for that project.

For any of these commands, the LLVM version must be added via features.

- `<features>` is a placeholder for `--features (llvm11-0 | llvm12-0 | llvm13-0 | llvm14-0)`

Build commands:

- `maturin build <features>`: Build the crate into python packages
- `maturin build --release <features>`: Build and pass --release to cargo
- `maturin build --help`: to view more options
- `maturin develop <features>`: Installs the crate as module in the current virtualenv
- `maturin develop <features> && pytest`: Installs the crate as module in the current
  virtualenv and runs the Python tests

If you do not wish to package and test the Python wheels, `cargo` can be used to
build the project and run Rust tests.

- `cargo build <features>`: Build the Rust cdylib
- `cargo build --release <features>`: Build the Rust cdylib in release mode
- `cargo test <features>`: Build and run the Rust cdylib tests
- `cargo test --release <features>`: Build and run the Rust cdylib tests in release mode

### Environment Variables

For those directly consuming `qirlib`, refer to the
[Environment variables](https://github.com/qir-alliance/pyqir/blob/main/qirlib/README.md#environment-variables)
section of it's README as the following constraints do not apply.

The Python PyQIR projects require LLVM to already be installed prior to build.
The PowerShell scripts will look at the the `qirlib`
[Environment variables](https://github.com/qir-alliance/pyqir/blob/main/qirlib/README.md#environment-variables)
and locate, build, or install LLVM as specified by the environment. The
default order is:

- Use specific LLVM installation if specified
- Locate existing LLVM installation on `PATH`
- Build LLVM from source
- Download LLVM if allowed from specified source

Afterward, the build configures the `LLVM_SYS_*_PREFIX` environment variable
according to what the environment has configured. This will allow LLVM to
be linked into the rest of the build.

### Packaging

The `build.ps1`, `maturin` builds all generate Python wheels to the
`target/wheels` folder. The default Python3 installation will be used targeting
Python ABI 3.7.

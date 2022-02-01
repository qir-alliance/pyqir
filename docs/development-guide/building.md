# Building PyQIR from Source

## Local Environment

### Requirements

- [Rust 1.56+](https://rustup.rs/)
- [Python 3.6+](https://www.python.org)
- [PowerShell 7+
  (Core)](https://github.com/powershell/powershell#get-powershell)
- [LLVM/Clang 11.1.0](https://llvm.org/) - See [Installing
  LLVM](#installing-llvm)
- [CMake 3.20+](https://github.com/Kitware/CMake/releases/tag/v3.20.5)
- [Ninja 1.10.0+](https://ninja-build.org/)

### Optional
- [sccache](https://github.com/mozilla/sccache)

### Linux (Ubuntu)

Install python and libs:

```bash
sudo apt-get install -y --no-install-recommends python3-dev python3-pip
python3 -m pip install --user -U pip
python3 -m pip install --user maturin tox
```

Install Rust from [rustup](https://rustup.rs/).

### Windows

Install Python 3.6+ from one of the following and make sure it is added to the
path.

- [Windows store](https://docs.microsoft.com/en-us/windows/python/beginners#install-python)
- [Miniconda](https://docs.conda.io/en/latest/miniconda.html#latest-miniconda-installer-links)
- [Python.org](https://www.python.org/downloads/)

In a command prompt:

```bash
python -m pip install --user maturin tox
```

Install Rust from [rustup](https://rustup.rs/).

### MacOS

Install Python 3.6+ from [Python.org](https://www.python.org/downloads/macos/).

or brew:

```bash
brew install 'python@3.9'
python -m pip install --user maturin tox
```

Install Rust from [rustup](https://rustup.rs/).

### Installing Clang and Ninja

If you have a working installation of LLVM and [Clang](https://clang.llvm.org/),
each project can be built by running `cargo build` in the project directory. If
not, you can install Clang manually:

- Linux (Ubuntu)

  ```bash
  apt-get update
  apt-get install -y clang-11 lldb-11 lld-11 clangd-11
  apt-get install -y --no-install-recommends ninja-build clang-tidy-11 build-essential
  ```

- Windows
  - Download and install the `LLVM-11.1.0-win64.exe` from the [11.1.0
    Release](https://github.com/llvm/llvm-project/releases/tag/llvmorg-11.1.0)
    page.
  - This package only contains the Clang components. There is no package that
    contains Clang and LLVM.
  - MacOS
  - Should be preinstalled.

### Installing LLVM

The build scripts will automatically download an LLVM toolchain which is
detailed in the [Development](#development) section. The build installs the
toolchain to `$HOME/.pyqir` (Windows: `$HOME\.pyqir`) and configures Rust to use
this installation by setting the `LLVM_SYS_110_PREFIX` environment variable in
the root `.cargo/config.toml`

### Installing CMake

- Linux (Ubuntu)

Using `apt-get` will install 3.16, but compiling the QIR runtime requires 3.20. To install the latest version on Ubuntu, install directly from the CMake releases from GitHub:

```bash
curl -SsL https://github.com/Kitware/CMake/releases/download/v3.20.5/cmake-3.20.5-linux-x86_64.sh -o cmakeinstall.sh
echo "f582e02696ceee81818dc3378531804b2213ed41c2a8bc566253d16d894cefab cmakeinstall.sh" | sha256sum -c --strict -
chmod +x cmakeinstall.sh
./cmakeinstall.sh --prefix=/usr/local --exclude-subdir
rm cmakeinstall.sh
```

## Development

To initialize your local environment and build
the solution, run

```bash
./build.ps1
```

Alternatively, you can use `build.sh` or `build.cmd`.

The {ref}`building/environment-variables` section
details ways to change this behavior.

Within each project folder, the build can be run specifically for that project.

Build commands:

- `maturin build`: Build the crate into python packages
- `maturin build --release`: Build and pass --release to cargo
- `maturin build --help`: to view more options
- `maturin develop`: Installs the crate as module in the current virtualenv
- `maturin develop && pytest`: Installs the crate as module in the current
  virtualenv and runs the Python tests

If you do not wish to package and test the Python wheels, `cargo` can be used to
build the project and run Rust tests.

- `cargo build`: Build the Rust cdylib
- `cargo build --release`: Build the Rust cdylib in release mode
- `cargo test`: Build and run the Rust cdylib tests
- `cargo test --release`: Build and run the Rust cdylib tests in release mode

[Tox](https://tox.readthedocs.io/) can be used as well:

Two targets are available for tox:

- `python -m tox -e test`
- Runs the python tests in an isolated environment
- `python -m tox -e pack`
- Packages all wheels in an isolated environment

(building/environment-variables)=

### Environment Variables

- `PYQIR_LLVM_EXTERNAL_DIR`
- Path to where LLVM is already installed by user. Useful if you want to use
  your own LLVM builds for testing.
- `PYQIR_DOWNLOAD_LLVM`
- Indicator to whether the build should download LLVM cached builds.
- Build will download LLVM if needed unless this variable is defined and set to
  `false`
- `PYQIR_LLVM_BUILDS_URL`
- Url from where LLVM builds will be downloaded.
- Default: `https://msquantumpublic.blob.core.windows.net/llvm-builds`
- `PYQIR_CACHE_DIR`
- Root insallation path for LLVM builds
- Default if not specified:
  - Linux/Mac: `$HOME/.pyqir`
  - Windows: `$HOME\.pyqir`
- `LLVM_SYS_110_PREFIX`
- Required by `llvm-sys` and will be set to the version of LLVM used for
  configuration.
- Version dependent and will change as LLVM is updated. (`LLVM_SYS_120_PREFIX`,
  `LLVM_SYS_130_PREFIX`, etc)
- Not needed if you have a working LLVM installation on the path.

### Packaging

The `build.(ps1|sh|cmd)`, `maturin`, and `tox` builds all generate Python wheels
to the `target/wheels` folder. The default Python3 installation will be used
targeting Python ABI 3.6.

The manylinux support uses a Docker image in the build scripts to run the builds
in the CI environment.

The Windows packaging will look for python installations available and build for
them. More information on [supporting multiple python versions on
Windows](https://tox.readthedocs.io/en/latest/developers.html?highlight=windows#multiple-python-versions-on-windows)

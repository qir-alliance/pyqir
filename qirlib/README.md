# qirlib

`qirlib` is a set of APIs for generating, parsing, and evaluating [Quantum
Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec). It
is also the native implementation behind the PyQIR packages:

- [**pyqir-generator**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-generator)
- [**pyqir-evaluator**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-evaluator)
- [**pyqir-parser**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-parser)

## Requirements

- Rust 1.57+
- LLVM 11.x, 12.x, 13.x, or LLVM 14.x (see [Usage](#usage) below)

## Usage

### Summary

`qirlib` requires a working LLVM installation to link against. By default, `qirlib`
assumes a valid LLVM installation is available. This default is to align with the
[`Inkwell`](https://github.com/thedan64/inkwell) and
[`llvm-sys`](https://github.com/tari/llvm-sys.rs) defaults which are leveraged by
`qirlib`. This default can be changed and `qirlib` provides features to
bootstrap itself if desired. Building `qirlib` can be done with feature flags which will either:

- (default) Use a local LLVM installation. This is either detected on the path or
specified with an environment variable.
- Build LLVM from source
- Download a preexisting LLVM build

Supported cargo features are following:

- The use `<llvm version>` is a placeholder for `llvm11-0`, `llvm12-0`, `llvm13-0`, or `llvm14-0`.
- The use of `<llvm major>` is a placeholder for `110`, `120`, `130`, or `140` corresponding to the LLVM releases.

| Feature                 | Link type               | Requirements        | Description                                            |
|:------------------------|:------------------------|:--------------------|:-------------------------------------------------------|
| `default`               | external | gcc/clang           | Includes `external-llvm-linking`     |
| `external-llvm-linking` | external | gcc/clang           | Uses llvm-sys/inkwell for LLVM linking                 |
| `<llvm version>-no-llvm-linking` |  none    |                     | Disable all LLVM linking. Used for local installation or packaging of LLVM. Includes `<llvm version>`, `llvm-sys-<llvm major>/disable-alltargets-init` and `inkwell/<llvm version>-no-llvm-linking`. Additionally adds `no-llvm-linking` as a marker used in the code. |
| `<llvm version>-qirlib-llvm-linking`  | internal  | `build-llvm` or `download-llvm` |  let `qirlib` do the LLVM linking. Includes `<llvm version>`, `llvm-sys-<llvm major>/disable-alltargets-init` and `inkwell/<llvm version>-no-llvm-linking`.  Additionally adds `qirlib-llvm-linking` as a marker used in the code. |
| `llvm11-0` | NA | `inkwell/llvm11-0` | Target LLVM 11.1 |
| `llvm12-0` | NA | `inkwell/llvm12-0` | Target LLVM 12.0 |
| `llvm13-0` | NA | `inkwell/llvm13-0` | Target LLVM 13.0 |
| `llvm14-0` | NA | `inkwell/llvm14-0` | Target LLVM 14.0 |
| `download-llvm` | internal/none | cmake | dowload a precompiled version of LLVM|
| `build-llvm`| internal/none | gcc/clang, cmake, ninja, git | Build LLVM from source. Installation defaults to `OUT_DIR/llvm` but can be overridden via the `QIRLIB_CACHE_DIR` environment variable.
| `package-llvm` | none | cc/clang, cmake, ninja, git | *Dev use only* for packaging LLVM builds. Includes `build-llvm` and `no-llvm-linking`

- Exactly one of the linking features is required:
  - `<llvm version>-qirlib-llvm-linking`
  - `external-llvm-linking`
  - `<llvm version>-no-llvm-linking`
- `build-llvm` and `download-llvm` cannot be used with `external-llvm-linking`
- Exactly one of the LLVM version features is required:
  - `llvm11-0`
  - `llvm12-0`
  - `llvm13-0`
  - `llvm14-0`

### Environment variables

Building `qirlib` can be done with crate features which will either download
(and install) a preexisting LLVM build configured via environment variables
(the
[Cargo.toml [env] section](https://doc.rust-lang.org/nightly/cargo/reference/config.html#env)
can also be used), build LLVM from source and install it, or build LLVM from
source and package it

- `QIRLIB_LLVM_EXTERNAL_DIR`
  - Path to where LLVM is already installed by user. Useful if you want to use
  your own LLVM builds for testing.
- `QIRLIB_DOWNLOAD_LLVM`
  - Indicator to whether the build should download LLVM cached builds.
  - Build will build LLVM from source if needed unless this variable is
  defined and set to `true`
- `QIRLIB_LLVM_BUILDS_URL`
  - Url from where LLVM builds will be downloaded.
  - There is no default value provided. You can use the `package-llvm` feature
    to create an archive and upload it along with SHA256 checksum file. The
    checksum file must have the same file name with a file of the same name
    with `.sha256` appended. The build will download both files and verify
    that the checksum in the the file matches the downloaded archive.
- `QIRLIB_CACHE_DIR`
  - Root installation path for LLVM builds (mapped to `CMAKE_INSTALL_PREFIX`)
  - Default if not specified when building from source:
    - `qirlib`'s target `OUT_DIR`/llvm
  - Default if not specified when installing:
    - Linux/Mac: `target/<llvm version>`
- `QIRLIB_LLVM_TAG`
  - LLVM repo tag to fetch when building LLVM from source.
  - Default values are:
    - `llvm11-0`: `llvmorg-11.1.0`
    - `llvm12-0`: `llvmorg-12.0.1`
    - `llvm13-0`: `llvmorg-13.0.1`
    - `llvm14-0`: `llvmorg-14.0.3`
- `QIRLIB_LLVM_PKG_NAME`
  - Optional name of package to be downloaded/created.
- `LLVM_SYS_*_PREFIX`
  - Required by `llvm-sys` and must be set to the version of LLVM used for
  configuration if compiling against an external LLVM installation.
  - Version dependent and will change as LLVM is updated. (`LLVM_SYS_120_PREFIX`,
  `LLVM_SYS_130_PREFIX`, etc)
  - Not needed if you have a working LLVM installation on the path and wish
  to use the installed system version.

### Using existing LLVM installation

Using an existing (external) LLVM installation is the default and effects
the linking that is used. The `default` feature enables the `external-llvm-linking` feature.

```toml
[dependencies]
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", features = "llvm11-0" }
# or 
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", features = "llvm12-0" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", features = "llvm13-0" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", features = "llvm14-0" }
```

[`llvm-sys`](https://github.com/tari/llvm-sys.rs) leveraged by `qirlib` and [`Inkwell`](https://github.com/thedan64/inkwell) will look for `llvm-config` on the path in order to determine how to link against LLVM. If this application is not found on the path, then the the `LLVM_SYS_<version>_PREFIX` environment variable is used to locate `llvm-config`.

This environment variable can be set in your `Cargo.toml` in the `[env]` section or in the build environment.

### Building (and linking) LLVM from source

In order to build and link LLVM from source, we must also tell
[`Inkwell`](https://github.com/thedan64/inkwell) and
[`llvm-sys`](https://github.com/tari/llvm-sys.rs) to disable
their own LLVM linking

To do this, we must disable the default behavior (`external-llvm-linking`)  using:

- The `--no-default-features` command-line flag disables the default features of the package.
- The `default-features = false` option can be specified in a dependency declaration.

```toml
[dependencies]
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm11-0-qirlib-llvm-linking,build-llvm" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm12-0-qirlib-llvm-linking,build-llvm" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm13-0-qirlib-llvm-linking,build-llvm" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm14-0-qirlib-llvm-linking,build-llvm" }
```

Or via the terminal (adding -vv so we can see build progress of LLVM)

```bash
qirlib> cargo build --release --no-default-features --features "llvm11-0-qirlib-llvm-linking,build-llvm" -vv
# or
qirlib> cargo build --release --no-default-features --features "llvm12-0-qirlib-llvm-linking,build-llvm" -vv
# or
qirlib> cargo build --release --no-default-features --features "llvm13-0-qirlib-llvm-linking,build-llvm" -vv
# or
qirlib> cargo build --release --no-default-features --features "llvm14-0-qirlib-llvm-linking,build-llvm" -vv
```

### Downloading (and linking) LLVM from pre-compiled binaries

Downloading an existing LLVM package and installing it can be configured via
[Environment variables]($Environment-variables). This is an advanced feature
and requires more effort to use.

In order to build and link LLVM from source, we must also tell
[`Inkwell`](https://github.com/thedan64/inkwell) and
[`llvm-sys`](https://github.com/tari/llvm-sys.rs) to disable
their own LLVM linking.

To do this, we must disable the default behavior (`external-llvm-linking`)  using:

- The `--no-default-features` command-line flag disables the default features of the package.
- The `default-features = false` option can be specified in a dependency declaration.

```toml
[dependencies]
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm11-0-qirlib-llvm-linking,download-llvm" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm12-0-qirlib-llvm-linking,download-llvm" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm13-0-qirlib-llvm-linking,download-llvm" }
# or
qirlib = { git = "https://github.com/qir-alliance/pyqir", branch = "main", default-features = false, features = "llvm14-0-qirlib-llvm-linking,download-llvm" }
```

Or via the terminal (adding -vv so we can see download progress of LLVM)

```bash
qirlib> cargo build --release --no-default-features --features "llvm11-0-qirlib-llvm-linking,download-llvm" -vv
# or
qirlib> cargo build --release --no-default-features --features "llvm12-0-qirlib-llvm-linking,download-llvm" -vv
# or
qirlib> cargo build --release --no-default-features --features "llvm13-0-qirlib-llvm-linking,download-llvm" -vv
# or
qirlib> cargo build --release --no-default-features --features "llvm14-0-qirlib-llvm-linking,download-llvm" -vv  
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

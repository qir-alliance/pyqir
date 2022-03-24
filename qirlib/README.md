# qirlib

`qirlib` is a set of APIs for generating, parsing, and evaluating [Quantum
Intermediate Representation (QIR)](https://github.com/qir-alliance/qir-spec). It
is also the native implementation behind the PyQIR packages:

- [**pyqir-generator**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-generator)
- [**pyqir-evaluator**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-evaluator)
- [**pyqir-parser**](https://github.com/qir-alliance/pyqir/tree/main/pyqir-parser):

## Requirements

- Rust 1.57+
- LLVM 11.x (see [Usage](#usage) below)

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

Supported features are following:

| Feature                 | Link type               | Requirements        | Description                                            |
|:------------------------|:------------------------|:--------------------|:-------------------------------------------------------|
| `default`               | external | gcc/clang           | Includes `external-llvm-linking`     |
| `external-llvm-linking` | external | gcc/clang           | Uses llvm-sys/inkwell for LLVM linking                 |
| `no-llvm-linking`       |  none    |                     | Disable all LLVM linking. Used for local installation or packaging of LLVM. Includes `llvm-sys/disable-alltargets-init` and `inkwell/llvm11-0-no-llvm-linking` |
| `qirlib-llvm-linking`  | internal  | `build-llvm` or `download-llvm` |  let qirlib do the LLVM linking. Includes `llvm-sys/disable-alltargets-init` and `inkwell/llvm11-0-no-llvm-linking` |
| `download-llvm` | internal/none | cmake | dowload a precompiled version of LLVM|
| `build-llvm`| internal/none | gcc/clang, cmake, ninja, git | Build LLVM from source. Installation defaults to `OUT_DIR/llvm` but can be overridden via the `QIRLIB_CACHE_DIR` environment variable.
| `package-llvm` | none | cc/clang, cmake, ninja, git | *Dev use only* for packaging LLVM builds. Includes `build-llvm` and `no-llvm-linking`

- Exactly one of the linking features is required:
  - `qirlib-llvm-linking`
  - `external-llvm-linking`
  - `no-llvm-linking`
- `build-llvm` and `download-llvm` cannot be used with `external-llvm-linking`


### Using existing LLVM installation

Using an existing (external) LLVM installation is the default and effects
the linking that is used. The `default` feature enables the `external-llvm-linking` feature.

```toml
[features]
# default to use llvm-sys/inkwell for llvm linking
default = ["external-llvm-linking"]
external-llvm-linking = []
```

[`llvm-sys`](https://github.com/tari/llvm-sys.rs) leveraged by `qirlib` and [`Inkwell`](https://github.com/thedan64/inkwell) will look for `llvm-config` on the path in order to determine how to link against LLVM. If this application is not found on the path, then the the `LLVM_SYS_<version>_PREFIX` environment variable is used to locate `llvm-config`. Only LLVM 11.x is supported at this time, so the exact environment variable name is `LLVM_SYS_110_PREFIX`.

This environment variable can be set in your `Cargo.toml` in the `[env]` section or in the build environment.

### Building (and linking) LLVM from source

In order to build and link LLVM from source, we must also tell
[`Inkwell`](https://github.com/thedan64/inkwell) and
[`llvm-sys`](https://github.com/tari/llvm-sys.rs) to disable
their own LLVM linking:

```toml
[features]
# let qirlib do the llvm linking
qirlib-llvm-linking = ["llvm-sys/disable-alltargets-init", "inkwell/llvm11-0-no-llvm-linking"]
```

Do to this, we must disable the default behavior (`external-llvm-linking`)  using:

- The `--no-default-features` command-line flag disables the default features of the package.
- The `default-features = false` option can be specified in a dependency declaration.

```toml
[dependencies]
qirlib = { version = "0.3.0", default-features = false, features = "qirlib-llvm-linking,build-llvm" }
```

Or via the terminal (adding -vv so we can see build progress of LLVM)

```bash
qirlib> cargo build --release --no-default-features --features "qirlib-llvm-linking,build-llvm" -vv   
```

### Downloading (and linking) LLVM from pre-compiled binaries

In order to build and link LLVM from source, we must also tell
[`Inkwell`](https://github.com/thedan64/inkwell) and
[`llvm-sys`](https://github.com/tari/llvm-sys.rs) to disable
their own LLVM linking:

```toml
[features]
# let qirlib do the llvm linking
qirlib-llvm-linking = ["llvm-sys/disable-alltargets-init", "inkwell/llvm11-0-no-llvm-linking"]
```

Do to this, we must disable the default behavior (`external-llvm-linking`)  using:

- The `--no-default-features` command-line flag disables the default features of the package.
- The `default-features = false` option can be specified in a dependency declaration.

```toml
[dependencies]
qirlib = { version = "0.3.0", default-features = false, features = "qirlib-llvm-linking,download-llvm" }
```

Or via the terminal (adding -vv so we can see download progress of LLVM)

```bash
qirlib> cargo build --release --no-default-features --features "qirlib-llvm-linking,download-llvm" -vv   
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

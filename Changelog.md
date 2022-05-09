# Changelog

## [Unreleased]

## [0.4.1a1] - 2022-05-09

- Adding pyqir-evaluator example with result stream usage. by @idavis in https://github.com/qir-alliance/pyqir/pull/97
- Support for "ret void" in the parser by @LaurentAjdnik in https://github.com/qir-alliance/pyqir/pull/113
- Add bitcode/qir conversion functions to generator by @idavis in https://github.com/qir-alliance/pyqir/pull/115

## [0.4.0a1] - 2022-04-18

- Adding static result allocation by @idavis in https://github.com/qir-alliance/pyqir/pull/103
- Bumping version to `0.4.0a1` by @idavis in https://github.com/qir-alliance/pyqir/pull/105
- Changing default value of `use_static_result_alloc` to `True` by @idavis in https://github.com/qir-alliance/pyqir/pull/106

PR [#106](https://github.com/qir-alliance/pyqir/pull/106) changes
the default way `Result`s are emitted in emitted QIR.

The previous approach
used `call %Result* @__quantum__qis__m__body(%Qubit* null)` for
measurement saving to a `Result` variable. Along with this, branching
used `__quantum__rt__result_equal(%Result*, %Result*)` along with
`__quantum__rt__result_get_zero` and `__quantum__rt__result_get_one`
to perform branching.

The new default usage in `0.4.0a1` uses static `Result` identifiers
instead of dynamic `Result` values. With that, measurement now uses
`"call void @__quantum__qis__mz__body(%Qubit*, %Result*)`
which accepts the static `Result` identifier. As a result, the
`pyqir-evaluator` can now see which `Result` is being written to.
Branching has also changed to use
`call i1 @__quantum__qir__read_result(%Result*)` which returns the
boolean value of the `Result`.

For the `pyqir-generator`, the use of static and dynamic `Qubit` and `Result` can now be configured via two new methods on the `SimpleModule`
class:

- `use_static_qubit_alloc(bool)`
- `use_static_result_alloc(bool)`

## [0.3.2a1] - 2022-04-05

- Add licensing checks and notice file by @idavis in https://github.com/qir-alliance/pyqir/pull/94
- Updating version to 0.3.2a1 by @idavis in https://github.com/qir-alliance/pyqir/pull/95
- Lock wheel package dependencies by @idavis in https://github.com/qir-alliance/pyqir/pull/96

## [0.3.1a1] - 2022-03-24

- Correct a few more typos and update docs by @LaurentAjdnik in https://github.com/qir-alliance/pyqir/pull/86
- Support external functions in generator by @samarsha in https://github.com/qir-alliance/pyqir/pull/76
- Improve error handling for invalid external function calls by @samarsha in https://github.com/qir-alliance/pyqir/pull/89
- Update PyPI package metadata by @idavis in https://github.com/qir-alliance/pyqir/pull/87
- Create pyqir metapackage for PyPI by @idavis in https://github.com/qir-alliance/pyqir/pull/90

## [0.3.0a1] - 2022-03-14

- Adding issue template for bugs, feature requests, and GitHub releases by @bettinaheim in https://github.com/qir-alliance/pyqir/pull/66
- Qirlib code consolidation by @idavis in https://github.com/qir-alliance/pyqir/pull/71
- Update _parser.py docs by @LaurentAjdnik in https://github.com/qir-alliance/pyqir/pull/72
- Rename `Ref` to `ResultRef` by @samarsha in https://github.com/qir-alliance/pyqir/pull/78
- Wheel/Project Renames by @idavis in https://github.com/qir-alliance/pyqir/pull/79
- Moving LLVM build downloads to GitHub PyQIR releases by @idavis in https://github.com/qir-alliance/pyqir/pull/83
- Cleaning up docs before publishing by @idavis in https://github.com/qir-alliance/pyqir/pull/84

## [0.2.0a1] - 2022-02-25

- Cache parser values for better hashing in Python [#41](https://github.com/qir-alliance/pyqir/pull/41)
- Moving to minimal API and eliminating bitcode template [#46](https://github.com/qir-alliance/pyqir/pull/46)
- PyQIR generator API redesign with classical if statement on Result values [#43](https://github.com/qir-alliance/pyqir/pull/43)
- QIR generation now uses static qubit allocation by default [#43](https://github.com/qir-alliance/pyqir/pull/43)

## [0.1.1a1] - 2022-01-31

### Core

- Enhance error messages when accessing qubit and registers [#11](https://github.com/qir-alliance/pyqir/pull/11)
- Updating `microsoft-quantum-qir-runtime-sys` to [f4f281236](https://github.com/microsoft/qsharp-runtime/commit/f4f28123601d8372a5fe120bdab1f2be25b51522) on main [#14](https://github.com/qir-alliance/pyqir/pull/14)
- Updating `pyo3` and `maturin` to latest versions. [#16](https://github.com/qir-alliance/pyqir/pull/16)
- Creating bare context/module/builder for JIT'ing [#17](https://github.com/qir-alliance/pyqir/pull/17)
- Adding cargo fmt/clippy checks on build with associated code fixes. [#21](https://github.com/qir-alliance/pyqir/pull/21)
- Support running modules with a custom entry point name or multiple entry points [#23](https://github.com/qir-alliance/pyqir/pull/23)
- Support loading modules from memory and clean up module loading API [#24](https://github.com/qir-alliance/pyqir/pull/24)

### Infrastructure

- Documentation and Cleanup for Release [#1](https://github.com/qir-alliance/pyqir/pull/1)
- Adding Mock language for Bernstein-Vazirani generation example [#4](https://github.com/qir-alliance/pyqir/pull/4)
- Properly picking up markdown lint config [#5](https://github.com/qir-alliance/pyqir/pull/5)
- Removed markdown lint workflow to take the org template instead [#6](https://github.com/qir-alliance/pyqir/pull/6)
- Create link and spell check validations [#7](https://github.com/qir-alliance/pyqir/pull/7)
- Added and updates readmes [#8](https://github.com/qir-alliance/pyqir/pull/8)
- Adding a contribution guide and a link to the code of conduct [#9](https://github.com/qir-alliance/pyqir/pull/9)
- Run generator and jit examples as part of the CI. [#10](https://github.com/qir-alliance/pyqir/pull/10)
- Update Instructions for Examples [#13](https://github.com/qir-alliance/pyqir/pull/13)
- Create Linux container smoke tests during CI [#22](https://github.com/qir-alliance/pyqir/pull/22)
- Build conceptual and API docs with Sphinx [#30](https://github.com/qir-alliance/pyqir/pull/30)
- Fix documentation links that couldn't be checked as part of the first docs PR [#37](https://github.com/qir-alliance/pyqir/pull/37)

## 0.1.0a1 - 2021-11-24

- Initial Release

[Unreleased]: https://github.com/qir-alliance/pyqir/compare/v0.4.1a1...HEAD
[0.4.1a1]: https://github.com/qir-alliance/pyqir/compare/v0.4.0a1...v0.4.1a1
[0.4.0a1]: https://github.com/qir-alliance/pyqir/compare/v0.3.2a1...v0.4.0a1
[0.3.2a1]: https://github.com/qir-alliance/pyqir/compare/v0.3.1a1...v0.3.2a1
[0.3.1a1]: https://github.com/qir-alliance/pyqir/compare/v0.3.0a1...v0.3.1a1
[0.3.0a1]: https://github.com/qir-alliance/pyqir/compare/v0.2.0a1...v0.3.0a1
[0.2.0a1]: https://github.com/qir-alliance/pyqir/compare/v0.1.0a1...v0.2.0a1
[0.1.1a1]: https://github.com/qir-alliance/pyqir/compare/v0.1.0a1...v0.1.1a1

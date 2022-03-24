# Changelog

## [Unreleased]

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

[Unreleased]: https://github.com/qir-alliance/pyqir/compare/v0.3.1a1...HEAD
[0.3.1a1]: https://github.com/qir-alliance/pyqir/compare/v0.3.0a1...v0.3.1a1
[0.3.0a1]: https://github.com/qir-alliance/pyqir/compare/v0.2.0a1...v0.3.0a1
[0.2.0a1]: https://github.com/qir-alliance/pyqir/compare/v0.1.0a1...v0.2.0a1
[0.1.1a1]: https://github.com/qir-alliance/pyqir/compare/v0.1.0a1...v0.1.1a1

# Upgrading iqm_pyqir

## iqm_pyqir 0.8

iqm_pyqir 0.7 was the last version of iqm_pyqir to support QIR evaluation. Simulation of QIR is now available via the [`qir-runner`](https://github.com/qir-alliance/qir-runner) sparse simulator.

## iqm_pyqir 0.7

### Packages

iqm_pyqir 0.6 was the last version of iqm_pyqir to use three packages (`iqm_pyqir-evaluator`, `iqm_pyqir-generator`, and `iqm_pyqir-parser`) and a metapackage (`iqm_pyqir`).
iqm_pyqir 0.7 instead uses only a single package (`iqm_pyqir`) that has the functionality of all previous packages.

If you imported the `iqm_pyqir.generator` or `iqm_pyqir.parser` modules, then the same or an equivalent API is available in the `iqm_pyqir` module.
If you imported the `iqm_pyqir.evaluator` module, it is still available under the same name with no API changes.

## Generator

### IR and bitcode conversion

The functions `bitcode_to_ir` and `ir_to_bitcode` were removed because the new `Module` class has the same functionality.
`Module` supports both parsing and generating QIR.
For example, instead of:

```python
from iqm_pyqir.generator import bitcode_to_ir, ir_to_bitcode

ir = bitcode_to_ir(bitcode, "module_name")
bitcode = ir_to_bitcode(ir, "module_name")

# Or with a source filename:
ir = bitcode_to_ir(bitcode, "module_name", "source_filename")
```

Use this:

```python
from iqm_pyqir import Context, Module

ir = str(Module.from_bitcode(Context(), bitcode, "name"))
bitcode = Module.from_ir(Context(), ir, "name").bitcode

# Or with a source filename:
m = Module.from_bitcode(Context(), bitcode, "module_name")
m.source_filename = "source_filename"
ir = str(m)
```

### Types

If you generated programs with externally-linked functions, then you used the `iqm_pyqir.generator.types` module to describe the type of the functions.
This module has been removed.
Types need to be created differently because they now directly contain LLVM type objects, which require an LLVM context.

| iqm_pyqir 0.6                                     | iqm_pyqir 0.7                         |
| --------------------------------------------- | --------------------------------- |
| `iqm_pyqir.generator.types.VOID`                  | `iqm_pyqir.Type.void(context)`        |
| `iqm_pyqir.generator.types.BOOL`                  | `iqm_pyqir.IntType(context, 1)`       |
| `iqm_pyqir.generator.types.Int(width)`            | `iqm_pyqir.IntType(context, width)`   |
| `iqm_pyqir.generator.types.DOUBLE`                | `iqm_pyqir.Type.double(context)`      |
| `iqm_pyqir.generator.types.QUBIT`                 | `iqm_pyqir.qubit_type(context)`       |
| `iqm_pyqir.generator.types.RESULT`                | `iqm_pyqir.result_type(context)`      |
| `iqm_pyqir.generator.types.Function(params, ret)` | `iqm_pyqir.FunctionType(ret, params)` |

There are two ways to get a `context` object:

1. Use the `context` property on `SimpleModule`.
   For example:

   ```python
   from iqm_pyqir import SimpleModule, Type
   module = SimpleModule("name", num_qubits=1, num_results=1)
   void = Type.void(module.context)
   ```

2. Create one yourself.
   But you also need to give the context you created to `SimpleModule`.
   For example:

   ```python
   from iqm_pyqir import Context, SimpleModule, Type
   context = Context()
   module = SimpleModule("name", num_qubits=1, num_results=1, context=context)
   void = Type.void(context)
   ```

## Parser

iqm_pyqir 0.7 unified parsing and code generation into a single API that is designed to support both.
This makes iqm_pyqir much more powerful and will enable workflows that involve inspecting, running passes on, or otherwise transforming QIR that is parsed or generated using iqm_pyqir.
This means that the API for `iqm_pyqir-parser` had to be completely redesigned, which unfortunately makes upgrading challenging.
Here are some tips.

### Modules

Use `Module.from_bitcode` or `Module.from_ir` instead of the `QirModule` constructor.
See [IR and bitcode conversion](#ir-and-bitcode-conversion).

### Entry points and interop-friendly functions

Instead of `QirModule.entrypoint_funcs`, `QirModule.interop_funcs`, or `QirModule.get_funcs_by_attr`, filter the `Module.functions` list instead.
For example:

```python
entry_point = next(filter(iqm_pyqir.is_entry_point, module.functions))
interops = filter(iqm_pyqir.is_interop_friendly, module.functions)
```

### Instructions

The instruction class hierarchy was trimmed down significantly.
Most subclasses of `QirInstr` were removed.
The surviving subclasses are `Call`, `FCmp`, `ICmp`, `Phi` and `Switch`.

For any other instruction, it should be possible to use the base `Instruction` class to do anything that was previously possible.
Use the `opcode` property to check what kind of instruction it is, and the `operands` property to read all of the values that the instruction references.

The `successors` property is a subset of `operands` and contains just the values that are basic blocks, which can be useful to follow control flow with `br` instructions.
For example, if you have a terminator instruction `term`, then you can get the instructions of its first successor with `term.successors[0].instructions`.

### Qubit and result IDs

In iqm_pyqir 0.6, `QirQubitConstant` and `QirResultConstant` were subclasses of `QirOperand`.
Instead, you can try to extract a static qubit or result ID from any value using `iqm_pyqir.qubit_id(value)` and `iqm_pyqir.result_id(value)`.
If the value isn't the right kind, it will return `None`.

### Strings

In iqm_pyqir 0.6, you could read a global string constant from a string value using `QirModule.get_global_bytes_value(value)`.
This is possible now using `iqm_pyqir.extract_byte_string(value)`.

### Examples

To see examples of how the new parser API can be used, take a look at [test_parser.py](https://github.com/qir-alliance/iqm_pyqir/blob/53e4aebfdb456e9603fae28543a8391075021a9f/iqm_pyqir/tests/test_parser.py).
You can also compare it with [test_parser_api.py](https://github.com/qir-alliance/iqm_pyqir/blob/v0.6.2/iqm_pyqir-parser/tests/test_parser_api.py) from iqm_pyqir 0.6 for a before-and-after view of the same test cases using both the old and new APIs.

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from iqm_pyqir import BasicQisBuilder, Context, SimpleModule, Module


def get_module() -> SimpleModule:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.mz(mod.qubits[0], mod.results[0])
    return mod


def test_ir_round_trip_is_identical() -> None:
    expected_ir = get_module().ir()
    bitcode = Module.from_ir(Context(), expected_ir, "test").bitcode
    converted_ir = str(Module.from_bitcode(Context(), bitcode, "test"))
    assert expected_ir == converted_ir


def test_ir_round_trip_is_not_identical_when_module_name_is_not_supplied() -> None:
    expected_ir = get_module().ir()
    bitcode = Module.from_ir(Context(), expected_ir).bitcode
    converted_ir = str(Module.from_bitcode(Context(), bitcode))
    assert expected_ir != converted_ir


def test_module_name_persists_in_conversion() -> None:
    expected_ir = get_module().ir()
    bitcode = Module.from_ir(Context(), expected_ir, "test").bitcode
    converted_ir = str(Module.from_bitcode(Context(), bitcode, "test2"))
    assert expected_ir != converted_ir
    assert "; ModuleID = 'test2'" in converted_ir


def test_file_name_persists_in_conversion() -> None:
    expected_ir = get_module().ir()
    module1 = Module.from_ir(Context(), expected_ir, "test")
    module1.source_filename = "some file"
    module2 = Module.from_bitcode(Context(), module1.bitcode, "test")
    module2.source_filename = "some other file"
    converted_ir = str(module2)
    assert expected_ir != converted_ir
    assert 'source_filename = "some other file"' in converted_ir


def test_ir_to_bitcode_returns_bytes_type() -> None:
    expected_ir = get_module().ir()
    bitcode = Module.from_ir(Context(), expected_ir, "test").bitcode
    assert isinstance(bitcode, bytes)


def test_bitcode_to_ir_returns_str_type() -> None:
    expected_ir = get_module().ir()
    bitcode = Module.from_ir(Context(), expected_ir, "test").bitcode
    converted_ir = str(Module.from_bitcode(Context(), bitcode, "test"))
    assert isinstance(converted_ir, str)


def test_bitcode_returns_bytes_type() -> None:
    bitcode = get_module().bitcode()
    assert isinstance(bitcode, bytes)


def test_ir_returns_str_type() -> None:
    expected_ir = get_module().ir()
    assert isinstance(expected_ir, str)

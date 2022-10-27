# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, ir_to_bitcode, bitcode_to_ir


def get_module() -> SimpleModule:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.mz(mod.qubits[0], mod.results[0])
    return mod


def test_ir_round_trip_is_identical() -> None:
    expected_ir = get_module().ir()
    bitcode = ir_to_bitcode(expected_ir, "test")
    converted_ir = bitcode_to_ir(bitcode, "test")
    assert expected_ir == converted_ir


def test_ir_round_trip_is_not_identical_when_module_name_isnot_supplied() -> None:
    expected_ir = get_module().ir()
    bitcode = ir_to_bitcode(expected_ir)
    converted_ir = bitcode_to_ir(bitcode)
    assert expected_ir != converted_ir


def test_module_name_persists_in_conversion() -> None:
    expected_ir = get_module().ir()
    bitcode = ir_to_bitcode(expected_ir, "test")
    converted_ir = bitcode_to_ir(bitcode, "test2")
    assert expected_ir != converted_ir
    assert "; ModuleID = 'test2'" in converted_ir


def test_file_name_persists_in_conversion() -> None:
    expected_ir = get_module().ir()
    bitcode = ir_to_bitcode(expected_ir, "test", "some file")
    converted_ir = bitcode_to_ir(bitcode, "test", "some other file")
    assert expected_ir != converted_ir
    assert 'source_filename = "some other file"' in converted_ir


def test_ir_to_bitcode_returns_bytes_type() -> None:
    expected_ir = get_module().ir()
    bitcode = ir_to_bitcode(expected_ir, "test")
    assert isinstance(bitcode, bytes)


def test_bitcode_to_ir_returns_str_type() -> None:
    expected_ir = get_module().ir()
    bitcode = ir_to_bitcode(expected_ir, "test")
    converted_ir = bitcode_to_ir(bitcode, "test")
    assert isinstance(converted_ir, str)


def test_bitcode_returns_bytes_type() -> None:
    bitcode = get_module().bitcode()
    assert isinstance(bitcode, bytes)


def test_ir_returns_str_type() -> None:
    expected_ir = get_module().ir()
    assert isinstance(expected_ir, str)

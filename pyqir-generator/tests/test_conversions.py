# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, ir_to_bitcode, bitcode_to_ir


def test_ir_round_trip_is_identical() -> None:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.m(mod.qubits[0], mod.results[0])
    expected_ir = mod.ir()
    bitcode = ir_to_bitcode(expected_ir, "test")
    converted_ir = bitcode_to_ir(bitcode, "test")
    assert expected_ir == converted_ir

def test_ir_to_bitcode_returns_bytes_type() -> None:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.m(mod.qubits[0], mod.results[0])
    expected_ir = mod.ir()
    bitcode = ir_to_bitcode(expected_ir, "test")
    assert isinstance(bitcode, bytes)

def test_bitcode_to_ir_returns_str_type() -> None:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.m(mod.qubits[0], mod.results[0])
    expected_ir = mod.ir()
    bitcode = ir_to_bitcode(expected_ir, "test")
    converted_ir = bitcode_to_ir(bitcode, "test")
    assert isinstance(converted_ir, str)

def test_bitcode_returns_bytes_type() -> None:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.m(mod.qubits[0], mod.results[0])
    bitcode = mod.bitcode()
    assert isinstance(bitcode, bytes)

def test_ir_returns_str_type() -> None:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.m(mod.qubits[0], mod.results[0])
    expected_ir = mod.ir()
    assert isinstance(expected_ir, str)

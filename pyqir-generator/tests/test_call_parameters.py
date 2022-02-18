# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator.module import SimpleModule
from pyqir.generator.qis import BasicQisBuilder


def create_context(num_qubits=0, num_results=0):
    module = SimpleModule("test", num_qubits, num_results)
    qis = BasicQisBuilder(module.builder)
    return (qis, module, module.qubits, module.results)


def test_call_no_params() -> None:
    (_, module, _, _) = create_context()

    module.builder.call("__test_call")

    ir = module.ir()
    assert 'call void @__test_call()' in ir


def test_call_single_qubit() -> None:
    (_, module, qubits, _) = create_context(1)

    module.builder.call("__test_call", qubits[0])

    ir = module.ir()
    assert 'call void @__test_call(%Qubit* null)' in ir


def test_call_two_qubits() -> None:
    (_, module, qubits, _) = create_context(2)
    module.builder.call("__test_call", qubits[0], qubits[1])

    ir = module.ir()
    assert 'call void @__test_call(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))' in ir


def test_call_float() -> None:
    (_, module, _, _) = create_context()
    module.builder.call("__test_call", 23.25)

    ir = module.ir()
    assert 'call void @__test_call(double 2.325000e+01)' in ir


def test_call_int() -> None:
    (_, module, _, _) = create_context()

    module.builder.call("__test_call", 42)

    ir = module.ir()
    assert 'call void @__test_call(i64 42)' in ir


def test_call_bool_true() -> None:
    (_, module, _, _) = create_context()
    module.builder.call("__test_call", True)

    ir = module.ir()
    assert 'call void @__test_call(i1 true)' in ir


def test_call_bool_false() -> None:
    (_, module, _, _) = create_context()
    module.builder.call("__test_call", False)

    ir = module.ir()
    assert 'call void @__test_call(i1 false)' in ir


def test_call_single_result() -> None:
    (qis, module, qubits, results) = create_context(1, 1)
    qis.m(qubits[0], results[0])
    module.builder.call("__test_call", results[0])

    ir = module.ir()
    assert 'call void @__test_call(%Result* %result0)' in ir


def test_call_two_results() -> None:
    (qis, module, qubits, results) = create_context(1, 2)
    qis.m(qubits[0], results[0])
    qis.m(qubits[0], results[1])
    module.builder.call("__test_call", results[0], results[1])

    ir = module.ir()
    assert 'call void @__test_call(%Result* %result0, %Result* %result1)' in ir


def test_call_number_extraction() -> None:
    (_, module, _, _) = create_context(0, 0)

    i = 42
    d = 42.42
    b = True
    int_rep = f"i64 {i}"
    double_rep = "double 4.242000e+01"
    bool_rep = f"i1 {b}".lower()

    builder = module.builder
    builder.call("__callidb", i, d, b)
    builder.call("__callibd", i, b, d)
    builder.call("__calldib", d, i, b)
    builder.call("__calldbi", d, b, i)
    builder.call("__callbid", b, i, d)
    builder.call("__callbdi", b, d, i)

    ir = module.ir()

    assert f'call void @__callidb({int_rep}, {double_rep}, {bool_rep})' in ir
    assert f'call void @__callibd({int_rep}, {bool_rep}, {double_rep})' in ir
    assert f'call void @__calldib({double_rep}, {int_rep}, {bool_rep})' in ir
    assert f'call void @__calldbi({double_rep}, {bool_rep}, {int_rep})' in ir
    assert f'call void @__callbid({bool_rep}, {int_rep}, {double_rep})' in ir
    assert f'call void @__callbdi({bool_rep}, {double_rep}, {int_rep})' in ir

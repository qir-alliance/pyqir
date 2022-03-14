# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Tuple
from pyqir.generator import BasicQisBuilder, FunctionType, Qubit, ResultRef, SimpleModule, Type, ValueType

Context = Tuple[
    BasicQisBuilder,
    SimpleModule,
    Tuple[Qubit, ...],
    Tuple[ResultRef, ...]
]


def create_context(num_qubits: int = 0, num_results: int = 0) -> Context:
    module = SimpleModule("test", num_qubits, num_results)
    qis = BasicQisBuilder(module.builder)
    return qis, module, module.qubits, module.results


def test_call_no_params() -> None:
    _, module, _, _ = create_context()

    callable = module.add_external_function(
        "__test_call", FunctionType([], Type.VOID)
    )
    module.builder.call(callable, [])

    ir = module.ir()
    assert 'call void @__test_call()' in ir


def test_call_single_qubit() -> None:
    _, module, qubits, _ = create_context(1)

    callable = module.add_external_function(
        "__test_call", FunctionType([ValueType.QUBIT], Type.VOID)
    )
    module.builder.call(callable, [qubits[0]])

    ir = module.ir()
    assert 'call void @__test_call(%Qubit* null)' in ir


def test_call_two_qubits() -> None:
    _, module, qubits, _ = create_context(2)

    callable = module.add_external_function(
        "__test_call",
        FunctionType([ValueType.QUBIT, ValueType.QUBIT], Type.VOID)
    )
    module.builder.call(callable, [qubits[0], qubits[1]])

    ir = module.ir()
    assert 'call void @__test_call(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))' in ir


def test_call_float() -> None:
    _, module, _, _ = create_context()

    callable = module.add_external_function(
        "__test_call", FunctionType([ValueType.DOUBLE], Type.VOID)
    )
    module.builder.call(callable, [23.25])

    ir = module.ir()
    assert 'call void @__test_call(double 2.325000e+01)' in ir


def test_call_int() -> None:
    _, module, _, _ = create_context()

    callable = module.add_external_function(
        "__test_call", FunctionType([ValueType.INT], Type.VOID)
    )
    module.builder.call(callable, [42])

    ir = module.ir()
    assert 'call void @__test_call(i64 42)' in ir


def test_call_bool_true() -> None:
    _, module, _, _ = create_context()

    callable = module.add_external_function(
        "__test_call", FunctionType([ValueType.BOOL], Type.VOID)
    )
    module.builder.call(callable, [True])

    ir = module.ir()
    assert 'call void @__test_call(i1 true)' in ir


def test_call_bool_false() -> None:
    _, module, _, _ = create_context()

    callable = module.add_external_function(
        "__test_call", FunctionType([ValueType.BOOL], Type.VOID)
    )
    module.builder.call(callable, [False])

    ir = module.ir()
    assert 'call void @__test_call(i1 false)' in ir


def test_call_single_result() -> None:
    qis, module, qubits, results = create_context(1, 1)
    qis.m(qubits[0], results[0])

    callable = module.add_external_function(
        "__test_call", FunctionType([ValueType.RESULT], Type.VOID)
    )
    module.builder.call(callable, [results[0]])

    ir = module.ir()
    assert 'call void @__test_call(%Result* %result0)' in ir


def test_call_two_results() -> None:
    qis, module, qubits, results = create_context(1, 2)
    qis.m(qubits[0], results[0])
    qis.m(qubits[0], results[1])

    callable = module.add_external_function(
        "__test_call",
        FunctionType([ValueType.RESULT, ValueType.RESULT], Type.VOID)
    )
    module.builder.call(callable, [results[0], results[1]])

    ir = module.ir()
    assert 'call void @__test_call(%Result* %result0, %Result* %result1)' in ir


def test_call_number_extraction() -> None:
    _, module, _, _ = create_context(0, 0)

    i = 42
    d = 42.42
    b = True
    int_rep = f"i64 {i}"
    double_rep = "double 4.242000e+01"
    bool_rep = f"i1 {b}".lower()

    idb = module.add_external_function("__callidb", FunctionType(
        [ValueType.INT, ValueType.DOUBLE, ValueType.BOOL], Type.VOID
    ))
    module.builder.call(idb, [i, d, b])

    ibd = module.add_external_function("__callibd", FunctionType(
        [ValueType.INT, ValueType.BOOL, ValueType.DOUBLE], Type.VOID
    ))
    module.builder.call(ibd, [i, b, d])

    dib = module.add_external_function("__calldib", FunctionType(
        [ValueType.DOUBLE, ValueType.INT, ValueType.BOOL], Type.VOID
    ))
    module.builder.call(dib, [d, i, b])

    dbi = module.add_external_function("__calldbi", FunctionType(
        [ValueType.DOUBLE, ValueType.BOOL, ValueType.INT], Type.VOID
    ))
    module.builder.call(dbi, [d, b, i])

    bid = module.add_external_function("__callbid", FunctionType(
        [ValueType.BOOL, ValueType.INT, ValueType.DOUBLE], Type.VOID
    ))
    module.builder.call(bid, [b, i, d])

    bdi = module.add_external_function("__callbdi", FunctionType(
        [ValueType.BOOL, ValueType.DOUBLE, ValueType.INT], Type.VOID
    ))
    module.builder.call(bdi, [b, d, i])

    ir = module.ir()
    assert f'call void @__callidb({int_rep}, {double_rep}, {bool_rep})' in ir
    assert f'call void @__callibd({int_rep}, {bool_rep}, {double_rep})' in ir
    assert f'call void @__calldib({double_rep}, {int_rep}, {bool_rep})' in ir
    assert f'call void @__calldbi({double_rep}, {bool_rep}, {int_rep})' in ir
    assert f'call void @__callbid({bool_rep}, {int_rep}, {double_rep})' in ir
    assert f'call void @__callbdi({bool_rep}, {double_rep}, {int_rep})' in ir

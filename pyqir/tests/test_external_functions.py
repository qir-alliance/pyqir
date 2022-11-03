# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import (
    BasicQisBuilder,
    SimpleModule,
    Type,
    TypeFactory,
    Value,
    const,
)
import re
import pytest
from typing import Any, Callable, List, Union


def test_call_no_params() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function("test_function", types.function(types.void, []))
    mod.builder.call(f, [])
    assert "call void @test_function()" in mod.ir()


def test_call_single_qubit() -> None:
    mod = SimpleModule("test", 1, 0)
    types = mod.types
    f = mod.add_external_function(
        "test_function", types.function(types.void, [types.qubit])
    )
    mod.builder.call(f, [mod.qubits[0]])
    assert "call void @test_function(%Qubit* null)" in mod.ir()


def test_call_two_qubits() -> None:
    mod = SimpleModule("test", 2, 0)
    types = mod.types
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, [types.qubit, types.qubit]),
    )
    mod.builder.call(f, [mod.qubits[0], mod.qubits[1]])
    assert (
        "call void @test_function(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))"
        in mod.ir()
    )


@pytest.mark.parametrize(
    "get_value", [lambda types: const(types.double, 23.25), lambda _: 23.25]
)
def test_call_double(get_value: Callable[[TypeFactory], Union[Value, float]]) -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, [types.double]),
    )
    mod.builder.call(f, [get_value(types)])
    assert "call void @test_function(double 2.325000e+01)" in mod.ir()


@pytest.mark.parametrize(
    "get_value", [lambda types: const(types.int(64), 42), lambda _: 42]
)
def test_call_int(get_value: Callable[[TypeFactory], Union[Value, int]]) -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, [types.int(64)]),
    )
    mod.builder.call(f, [get_value(types)])
    assert "call void @test_function(i64 42)" in mod.ir()


@pytest.mark.parametrize(
    "get_value", [lambda types: const(types.bool, True), lambda _: True]
)
def test_call_bool_true(get_value: Callable[[TypeFactory], Union[Value, bool]]) -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, [types.bool]),
    )
    mod.builder.call(f, [get_value(types)])
    assert "call void @test_function(i1 true)" in mod.ir()


@pytest.mark.parametrize(
    "get_value", [lambda types: const(types.bool, False), lambda _: False]
)
def test_call_bool_false(
    get_value: Callable[[TypeFactory], Union[Value, bool]]
) -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, [types.bool]),
    )
    mod.builder.call(f, [get_value(types)])
    assert "call void @test_function(i1 false)" in mod.ir()


def test_call_single_result() -> None:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.mz(mod.qubits[0], mod.results[0])

    types = mod.types
    f = mod.add_external_function(
        "test_function", types.function(types.void, [types.result])
    )
    mod.builder.call(f, [mod.results[0]])
    assert "call void @test_function(%Result* null)" in mod.ir()


def test_call_two_results() -> None:
    mod = SimpleModule("test", 1, 2)
    qis = BasicQisBuilder(mod.builder)
    qis.mz(mod.qubits[0], mod.results[0])
    qis.mz(mod.qubits[0], mod.results[1])

    types = mod.types
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, [types.result, types.result]),
    )
    mod.builder.call(f, [mod.results[1], mod.results[0]])

    assert (
        "call void @test_function(%Result* inttoptr (i64 1 to %Result*), %Result* null)"
        in mod.ir()
    )


def test_call_numbers() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types

    b = const(types.bool, True)
    bool_rep = f"i1 true"
    i = const(types.int(64), 42)
    int_rep = f"i64 42"
    d = const(types.double, 42.42)
    double_rep = "double 4.242000e+01"

    f = mod.add_external_function(
        "test_function",
        types.function(
            types.void,
            [types.bool, types.int(64), types.double],
        ),
    )
    mod.builder.call(f, [b, i, d])
    assert f"call void @test_function({bool_rep}, {int_rep}, {double_rep})" in mod.ir()


@pytest.mark.parametrize(
    "get_types, args",
    [
        (lambda types: [types.bool], ["true"]),
        (lambda types: [types.int(64)], [1.23]),
        (lambda types: [types.int(64)], ["123"]),
        (lambda types: [types.double], ["1.23"]),
    ],
)
def test_wrong_type_conversion(
    get_types: Callable[[TypeFactory], List[Type]], args: List[Any]
) -> None:
    mod = SimpleModule("test", 1, 1)
    types = mod.types
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, get_types(types)),
    )
    with pytest.raises(TypeError):
        mod.builder.call(f, args)


def test_overflow_bool_value() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function("f", types.function(types.void, [types.bool]))
    b = const(types.bool, 123)
    mod.builder.call(f, [b])
    assert "call void @f(i1 true)" in mod.ir()


def test_underflow_bool_value() -> None:
    mod = SimpleModule("test", 0, 0)
    with pytest.raises(OverflowError):
        const(mod.types.bool, -123)


def test_overflow_bool_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function("f", types.function(types.void, [types.bool]))
    mod.builder.call(f, [123])
    assert "call void @f(i1 true)" in mod.ir()


def test_underflow_bool_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function("f", types.function(types.void, [types.bool]))
    with pytest.raises(OverflowError):
        mod.builder.call(f, [-123])


def test_overflow_int_value() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function("f", types.function(types.void, [types.int(32)]))
    i = const(types.int(32), 2**32 + 123)
    mod.builder.call(f, [i])
    assert "call void @f(i32 123)" in mod.ir()


def test_underflow_int_value() -> None:
    mod = SimpleModule("test", 0, 0)
    with pytest.raises(OverflowError):
        const(mod.types.int(32), -(2**32) - 123)


def test_overflow_int_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function("f", types.function(types.void, [types.int(32)]))
    mod.builder.call(f, [2**32 + 123])
    assert "call void @f(i32 123)" in mod.ir()


def test_underflow_int_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    f = mod.add_external_function("f", types.function(types.void, [types.int(32)]))
    with pytest.raises(OverflowError):
        mod.builder.call(f, [-(2**32) - 123])


def test_64_bit_overflow() -> None:
    mod = SimpleModule("test", 0, 0)
    with pytest.raises(OverflowError):
        const(mod.types.int(128), 2**64)


@pytest.mark.parametrize(
    "get_args",
    [
        lambda _: [],
        lambda types: [const(types.double, 1.23)],
        lambda types: [
            const(types.double, 1.23),
            const(types.bool, True),
            const(types.bool, False),
        ],
    ],
)
def test_wrong_number_of_args(get_args: Callable[[TypeFactory], List[Value]]) -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    args = get_args(types)

    param_types: List[Type] = [types.double, types.bool]
    f = mod.add_external_function(
        "test_function",
        types.function(types.void, param_types),
    )

    message = f"Expected {len(param_types)} arguments, got {len(args)}."
    with pytest.raises(ValueError, match="^" + re.escape(message) + "$"):
        mod.builder.call(f, args)


def test_variable() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    foo = mod.add_external_function("foo", types.function(types.int(64), []))
    bar = mod.add_external_function(
        "bar",
        types.function(types.void, [types.int(64)]),
    )

    x = mod.builder.call(foo, [])
    assert x is not None
    mod.builder.call(bar, [x])

    ir = mod.ir()
    assert "%0 = call i64 @foo()" in ir
    assert "call void @bar(i64 %0)" in ir


def test_variable_wrong_external_type() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    foo = mod.add_external_function("foo", types.function(types.int(64), []))
    bar = mod.add_external_function("bar", types.function(types.void, [types.qubit]))

    x = mod.builder.call(foo, [])
    assert x is not None
    mod.builder.call(bar, [x])

    with pytest.raises(
        OSError, match="^Call parameter type does not match function signature!"
    ):
        mod.ir()


def test_variable_wrong_angle_type() -> None:
    mod = SimpleModule("test", 1, 0)
    types = mod.types
    qis = BasicQisBuilder(mod.builder)
    foo = mod.add_external_function("foo", types.function(types.int(64), []))

    x = mod.builder.call(foo, [])
    assert x is not None
    with pytest.raises(BaseException, match="^Found IntValue"):
        qis.rz(x, mod.qubits[0])


def test_two_variables() -> None:
    mod = SimpleModule("test", 0, 0)
    types = mod.types
    foo = mod.add_external_function("foo", types.function(types.int(64), []))
    bar = mod.add_external_function(
        "bar",
        types.function(
            types.void,
            [types.int(64), types.int(64)],
        ),
    )

    x = mod.builder.call(foo, [])
    assert x is not None
    y = mod.builder.call(foo, [])
    assert y is not None
    mod.builder.call(bar, [x, y])

    ir = mod.ir()
    assert "%0 = call i64 @foo()" in ir
    assert "%1 = call i64 @foo()" in ir
    assert "call void @bar(i64 %0, i64 %1)" in ir


def test_computed_rotation() -> None:
    mod = SimpleModule("test", 1, 0)
    types = mod.types
    qis = BasicQisBuilder(mod.builder)
    foo = mod.add_external_function("foo", types.function(types.double, []))

    theta = mod.builder.call(foo, [])
    assert theta is not None
    qis.rx(theta, mod.qubits[0])
    qis.ry(theta, mod.qubits[0])
    qis.rz(theta, mod.qubits[0])

    ir = mod.ir()
    assert "%0 = call double @foo()" in ir
    assert "call void @__quantum__qis__rx__body(double %0, %Qubit* null)" in ir
    assert "call void @__quantum__qis__ry__body(double %0, %Qubit* null)" in ir
    assert "call void @__quantum__qis__rz__body(double %0, %Qubit* null)" in ir

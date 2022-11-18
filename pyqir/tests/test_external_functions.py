# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import re
from typing import Any, Callable, List, Union

import pytest

import pyqir
from pyqir import (
    BasicQisBuilder,
    Context,
    FunctionType,
    IntType,
    PointerType,
    SimpleModule,
    Type,
    Value,
)


def test_call_no_params() -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "test_function", FunctionType(Type.void(mod.context), [])
    )
    mod.builder.call(f, [])
    assert "call void @test_function()" in mod.ir()


def test_call_single_qubit() -> None:
    mod = SimpleModule("test", 1, 0)
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), [pyqir.qubit_type(mod.context)]),
    )
    mod.builder.call(f, [mod.qubits[0]])
    assert "call void @test_function(%Qubit* null)" in mod.ir()


def test_call_two_qubits() -> None:
    mod = SimpleModule("test", 2, 0)
    qubit = pyqir.qubit_type(mod.context)
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), [qubit, qubit]),
    )
    mod.builder.call(f, [mod.qubits[0], mod.qubits[1]])
    assert (
        "call void @test_function(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))"
        in mod.ir()
    )


@pytest.mark.parametrize(
    "get_value",
    [lambda context: pyqir.const(Type.double(context), 23.25), lambda _: 23.25],
)
def test_call_double(get_value: Callable[[Context], Union[Value, float]]) -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), [Type.double(mod.context)]),
    )
    mod.builder.call(f, [get_value(mod.context)])
    assert "call void @test_function(double 2.325000e+01)" in mod.ir()


@pytest.mark.parametrize(
    "get_value",
    [lambda context: pyqir.const(IntType(context, 64), 42), lambda _: 42],
)
def test_call_int(get_value: Callable[[Context], Union[Value, int]]) -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), [IntType(mod.context, 64)]),
    )
    mod.builder.call(f, [get_value(mod.context)])
    assert "call void @test_function(i64 42)" in mod.ir()


@pytest.mark.parametrize(
    "get_value",
    [lambda context: pyqir.const(IntType(context, 1), True), lambda _: True],
)
def test_call_bool_true(get_value: Callable[[Context], Union[Value, bool]]) -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), [IntType(mod.context, 1)]),
    )
    mod.builder.call(f, [get_value(mod.context)])
    assert "call void @test_function(i1 true)" in mod.ir()


@pytest.mark.parametrize(
    "get_value",
    [lambda context: pyqir.const(IntType(context, 1), False), lambda _: False],
)
def test_call_bool_false(get_value: Callable[[Context], Union[Value, bool]]) -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), [IntType(mod.context, 1)]),
    )
    mod.builder.call(f, [get_value(mod.context)])
    assert "call void @test_function(i1 false)" in mod.ir()


def test_call_single_result() -> None:
    mod = SimpleModule("test", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.mz(mod.qubits[0], mod.results[0])
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), [pyqir.result_type(mod.context)]),
    )
    mod.builder.call(f, [mod.results[0]])
    assert "call void @test_function(%Result* null)" in mod.ir()


def test_call_two_results() -> None:
    mod = SimpleModule("test", 1, 2)
    qis = BasicQisBuilder(mod.builder)
    qis.mz(mod.qubits[0], mod.results[0])
    qis.mz(mod.qubits[0], mod.results[1])

    result = pyqir.result_type(mod.context)
    f = mod.add_external_function(
        "test_function", FunctionType(Type.void(mod.context), [result, result])
    )
    mod.builder.call(f, [mod.results[1], mod.results[0]])

    assert (
        "call void @test_function(%Result* inttoptr (i64 1 to %Result*), %Result* null)"
        in mod.ir()
    )


def test_call_numbers() -> None:
    mod = SimpleModule("test", 0, 0)
    void = Type.void(mod.context)
    i1 = IntType(mod.context, 1)
    i64 = IntType(mod.context, 64)
    double = Type.double(mod.context)

    b = pyqir.const(i1, True)
    bool_rep = f"i1 true"
    i = pyqir.const(i64, 42)
    int_rep = f"i64 42"
    d = pyqir.const(double, 42.42)
    double_rep = "double 4.242000e+01"

    f = mod.add_external_function(
        "test_function", FunctionType(void, [i1, i64, double])
    )
    mod.builder.call(f, [b, i, d])
    assert f"call void @test_function({bool_rep}, {int_rep}, {double_rep})" in mod.ir()


@pytest.mark.parametrize(
    "get_types, args",
    [
        (lambda context: [IntType(context, 1)], ["true"]),
        (lambda context: [IntType(context, 64)], [1.23]),
        (lambda context: [IntType(context, 64)], ["123"]),
        (lambda context: [Type.double(context)], ["1.23"]),
    ],
)
def test_wrong_type_conversion(
    get_types: Callable[[Context], List[Type]], args: List[Any]
) -> None:
    mod = SimpleModule("test", 1, 1)
    f = mod.add_external_function(
        "test_function",
        FunctionType(Type.void(mod.context), get_types(mod.context)),
    )
    with pytest.raises(TypeError):
        mod.builder.call(f, args)


def test_overflow_bool_value() -> None:
    mod = SimpleModule("test", 0, 0)
    i1 = IntType(mod.context, 1)
    f = mod.add_external_function("f", FunctionType(Type.void(mod.context), [i1]))
    b = pyqir.const(i1, 123)
    mod.builder.call(f, [b])
    assert "call void @f(i1 true)" in mod.ir()


def test_underflow_bool_value() -> None:
    mod = SimpleModule("test", 0, 0)
    with pytest.raises(OverflowError):
        pyqir.const(IntType(mod.context, 1), -123)


def test_overflow_bool_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "f", FunctionType(Type.void(mod.context), [IntType(mod.context, 1)])
    )
    mod.builder.call(f, [123])
    assert "call void @f(i1 true)" in mod.ir()


def test_underflow_bool_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "f", FunctionType(Type.void(mod.context), [IntType(mod.context, 1)])
    )
    with pytest.raises(OverflowError):
        mod.builder.call(f, [-123])


def test_overflow_int_value() -> None:
    mod = SimpleModule("test", 0, 0)
    i32 = IntType(mod.context, 32)
    f = mod.add_external_function("f", FunctionType(Type.void(mod.context), [i32]))
    i = pyqir.const(i32, 2**32 + 123)
    mod.builder.call(f, [i])
    assert "call void @f(i32 123)" in mod.ir()


def test_underflow_int_value() -> None:
    mod = SimpleModule("test", 0, 0)
    with pytest.raises(OverflowError):
        pyqir.const(IntType(mod.context, 32), -(2**32) - 123)


def test_overflow_int_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "f", FunctionType(Type.void(mod.context), [IntType(mod.context, 32)])
    )
    mod.builder.call(f, [2**32 + 123])
    assert "call void @f(i32 123)" in mod.ir()


def test_underflow_int_literal() -> None:
    mod = SimpleModule("test", 0, 0)
    f = mod.add_external_function(
        "f", FunctionType(Type.void(mod.context), [IntType(mod.context, 32)])
    )
    with pytest.raises(OverflowError):
        mod.builder.call(f, [-(2**32) - 123])


def test_64_bit_overflow() -> None:
    mod = SimpleModule("test", 0, 0)
    with pytest.raises(OverflowError):
        pyqir.const(IntType(mod.context, 128), 2**64)


@pytest.mark.parametrize(
    "get_args",
    [
        lambda _: [],
        lambda context: [pyqir.const(Type.double(context), 1.23)],
        lambda context: [
            pyqir.const(Type.double(context), 1.23),
            pyqir.const(IntType(context, 1), True),
            pyqir.const(IntType(context, 1), False),
        ],
    ],
)
def test_wrong_number_of_args(get_args: Callable[[Context], List[Value]]) -> None:
    mod = SimpleModule("test", 0, 0)
    args = get_args(mod.context)

    param_types: List[Type] = [Type.double(mod.context), IntType(mod.context, 1)]
    f = mod.add_external_function(
        "test_function", FunctionType(Type.void(mod.context), param_types)
    )

    message = f"Expected {len(param_types)} arguments, got {len(args)}."
    with pytest.raises(ValueError, match="^" + re.escape(message) + "$"):
        mod.builder.call(f, args)


def test_variable() -> None:
    mod = SimpleModule("test", 0, 0)
    i64 = IntType(mod.context, 64)
    foo = mod.add_external_function("foo", FunctionType(i64, []))
    bar = mod.add_external_function("bar", FunctionType(Type.void(mod.context), [i64]))

    x = mod.builder.call(foo, [])
    assert x is not None
    mod.builder.call(bar, [x])

    ir = mod.ir()
    assert "%0 = call i64 @foo()" in ir
    assert "call void @bar(i64 %0)" in ir


def test_variable_wrong_external_type() -> None:
    mod = SimpleModule("test", 0, 0)
    foo = mod.add_external_function("foo", FunctionType(IntType(mod.context, 64), []))
    bar = mod.add_external_function(
        "bar", FunctionType(Type.void(mod.context), [pyqir.qubit_type(mod.context)])
    )

    x = mod.builder.call(foo, [])
    assert x is not None
    mod.builder.call(bar, [x])

    with pytest.raises(
        OSError, match="^Call parameter type does not match function signature!"
    ):
        mod.ir()


def test_variable_wrong_angle_type() -> None:
    mod = SimpleModule("test", 1, 0)
    qis = BasicQisBuilder(mod.builder)
    foo = mod.add_external_function("foo", FunctionType(IntType(mod.context, 64), []))

    x = mod.builder.call(foo, [])
    assert x is not None
    with pytest.raises(
        BaseException, match=r"^Couldn't convert value to float value\.$"
    ):
        qis.rz(x, mod.qubits[0])


def test_two_variables() -> None:
    mod = SimpleModule("test", 0, 0)
    i64 = IntType(mod.context, 64)
    foo = mod.add_external_function("foo", FunctionType(i64, []))
    bar = mod.add_external_function(
        "bar", FunctionType(Type.void(mod.context), [i64, i64])
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
    qis = BasicQisBuilder(mod.builder)
    foo = mod.add_external_function("foo", FunctionType(Type.double(mod.context), []))

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


def test_record_output() -> None:
    mod = SimpleModule("test", 0, 1)
    result_record_output = mod.add_external_function(
        "__quantum__rt__result_record_output",
        FunctionType(
            Type.void(mod.context),
            [pyqir.result_type(mod.context), PointerType(IntType(mod.context, 8))],
        ),
    )
    tag = mod.add_global_string(b"foo")
    i32 = IntType(mod.context, 32)
    mod.builder.call(
        result_record_output,
        [
            mod.results[0],
            pyqir.const_getelementptr(tag, [pyqir.const(i32, 0), pyqir.const(i32, 0)]),
        ],
    )

    ir = mod.ir()
    assert r'@0 = internal constant [4 x i8] c"foo\00"' in ir
    assert (
        "call void @__quantum__rt__result_record_output(%Result* null, i8* getelementptr inbounds ([4 x i8], [4 x i8]* @0, i32 0, i32 0))"
        in ir
    )

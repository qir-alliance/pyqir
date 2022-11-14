# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    Context,
    FunctionType,
    IntType,
    PointerType,
    Type,
    is_qubit_type,
    is_result_type,
    qubit_type,
    result_type,
)
import pytest


def test_void() -> None:
    void = Type.void(Context())
    assert type(void) is Type
    assert void.is_void


def test_double() -> None:
    double = Type.double(Context())
    assert type(double) is Type
    assert double.is_double


@pytest.mark.parametrize("width", [1, 2, 8, 16, 32, 64, 128])
def test_int(width: int) -> None:
    i = IntType(Context(), width)
    assert i.width == width


def test_function() -> None:
    context = Context()
    function = FunctionType(
        Type.void(context),
        [
            Type.double(context),
            IntType(context, 64),
        ],
    )
    assert function.ret.is_void
    assert len(function.params) == 2
    assert function.params[0].is_double
    assert isinstance(function.params[1], IntType)


def test_pointer() -> None:
    pointer = PointerType(IntType(Context(), 1))
    assert isinstance(pointer.pointee, IntType)


def test_void_pointer() -> None:
    with pytest.raises(ValueError):
        PointerType(Type.void(Context()))


def test_qubit() -> None:
    qubit = qubit_type(Context())
    assert is_qubit_type(qubit)


def test_result() -> None:
    result = result_type(Context())
    assert is_result_type(result)

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest

import pyqir
from pyqir import Context, FunctionType, IntType, PointerType, Type


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
    # We expect this to be an opaque pointer, which always shows up with the void type.
    assert pointer.pointee.is_void


def test_void_pointer() -> None:
    voidp = PointerType(Type.void(Context()))
    assert voidp.pointee.is_void

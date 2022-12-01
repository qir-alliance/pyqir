# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from functools import partial
from typing import Callable, List, Tuple

import pytest

import pyqir
from pyqir import (
    Builder,
    FunctionType,
    IntPredicate,
    IntType,
    SimpleModule,
    Type,
    Value,
)

_OPERATORS: List[Tuple[str, Callable[[Builder], Callable[[Value, Value], Value]]]] = [
    ("and", lambda b: b.and_),
    ("or", lambda b: b.or_),
    ("xor", lambda b: b.xor),
    ("add", lambda b: b.add),
    ("sub", lambda b: b.sub),
    ("mul", lambda b: b.mul),
    ("shl", lambda b: b.shl),
    ("lshr", lambda b: b.lshr),
    ("icmp eq", lambda b: partial(b.icmp, IntPredicate.EQ)),
    ("icmp ne", lambda b: partial(b.icmp, IntPredicate.NE)),
    ("icmp ugt", lambda b: partial(b.icmp, IntPredicate.UGT)),
    ("icmp uge", lambda b: partial(b.icmp, IntPredicate.UGE)),
    ("icmp ult", lambda b: partial(b.icmp, IntPredicate.ULT)),
    ("icmp ule", lambda b: partial(b.icmp, IntPredicate.ULE)),
    ("icmp sgt", lambda b: partial(b.icmp, IntPredicate.SGT)),
    ("icmp sge", lambda b: partial(b.icmp, IntPredicate.SGE)),
    ("icmp slt", lambda b: partial(b.icmp, IntPredicate.SLT)),
    ("icmp sle", lambda b: partial(b.icmp, IntPredicate.SLE)),
]


@pytest.mark.parametrize("name, build", _OPERATORS)
def test_variable_variable(
    name: str, build: Callable[[Builder], Callable[[Value, Value], Value]]
) -> None:
    mod = SimpleModule("test " + name, 0, 0)
    i64 = IntType(mod.context, 64)
    source = mod.add_external_function("source", FunctionType(i64, []))
    ty = IntType(mod.context, 1) if name.startswith("icmp") else i64
    sink = mod.add_external_function("sink", FunctionType(Type.void(mod.context), [ty]))
    x = mod.builder.call(source, [])
    assert x is not None
    y = mod.builder.call(source, [])
    assert y is not None
    z = build(mod.builder)(x, y)
    mod.builder.call(sink, [z])
    assert f"%2 = {name} i64 %0, %1" in mod.ir()


@pytest.mark.parametrize("name, build", _OPERATORS)
def test_constant_variable(
    name: str, build: Callable[[Builder], Callable[[Value, Value], Value]]
) -> None:
    mod = SimpleModule("test " + name, 0, 0)
    i64 = IntType(mod.context, 64)
    source = mod.add_external_function("source", FunctionType(i64, []))
    ty = IntType(mod.context, 1) if name.startswith("icmp") else i64
    sink = mod.add_external_function("sink", FunctionType(Type.void(mod.context), [ty]))
    x = mod.builder.call(source, [])
    assert x is not None
    y = build(mod.builder)(pyqir.const(i64, 1), x)
    mod.builder.call(sink, [y])
    assert f"%1 = {name} i64 1, %0" in mod.ir()


def test_type_mismatch() -> None:
    mod = SimpleModule("test_type_mismatch", 0, 0)
    i16 = IntType(mod.context, 16)
    source = mod.add_external_function("source", FunctionType(i16, []))
    sink = mod.add_external_function(
        "sink", FunctionType(Type.void(mod.context), [i16])
    )
    x = mod.builder.call(source, [])
    assert x is not None
    y = mod.builder.add(x, pyqir.const(IntType(mod.context, 18), 2))
    mod.builder.call(sink, [y])
    with pytest.raises(
        ValueError,
        match="^Both operands to a binary operator are not of the same type!",
    ):
        mod.ir()

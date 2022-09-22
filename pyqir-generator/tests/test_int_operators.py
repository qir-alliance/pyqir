# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from functools import partial
from pyqir.generator import Builder, IntPredicate, SimpleModule, Value, const
from typing import Callable, List, Tuple
import unittest

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


class IntOperatorsTest(unittest.TestCase):
    def test_variable_variable(self) -> None:
        for (name, build) in _OPERATORS:
            with self.subTest(name):
                mod = SimpleModule("test " + name, 0, 0)
                types = mod.types
                source = mod.add_external_function(
                    "source", types.function(types.integer(64), [])
                )
                ty = types.bool if name.startswith("icmp") else types.integer(64)
                sink = mod.add_external_function(
                    "sink", types.function(types.void, [ty])
                )

                x = mod.builder.call(source, [])
                assert x is not None
                y = mod.builder.call(source, [])
                assert y is not None
                z = build(mod.builder)(x, y)
                mod.builder.call(sink, [z])

                self.assertIn(f"%2 = {name} i64 %0, %1", mod.ir())

    def test_constant_variable(self) -> None:
        for (name, build) in _OPERATORS:
            with self.subTest(name):
                mod = SimpleModule("test " + name, 0, 0)
                types = mod.types
                source = mod.add_external_function(
                    "source", types.function(types.integer(64), [])
                )
                ty = types.bool if name.startswith("icmp") else types.integer(64)
                sink = mod.add_external_function(
                    "sink", types.function(types.void, [ty])
                )

                x = mod.builder.call(source, [])
                assert x is not None
                y = build(mod.builder)(const(types.integer(64), 1), x)
                mod.builder.call(sink, [y])

                self.assertIn(f"%1 = {name} i64 1, %0", mod.ir())

    def test_type_mismatch(self) -> None:
        mod = SimpleModule("test_type_mismatch", 0, 0)
        types = mod.types
        source = mod.add_external_function(
            "source", types.function(types.integer(16), [])
        )
        sink = mod.add_external_function(
            "sink",
            types.function(types.void, [types.integer(16)]),
        )

        x = mod.builder.call(source, [])
        assert x is not None
        y = mod.builder.add(x, const(types.integer(18), 2))
        mod.builder.call(sink, [y])

        with self.assertRaises(OSError):
            mod.ir()

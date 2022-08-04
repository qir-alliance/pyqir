# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from functools import partial
from pyqir.generator import Builder, IPredicate, SimpleModule, Value, const, types
from typing import Callable, List, Tuple
import unittest

_BINARY_INTRINSICS: List[Tuple[str, Callable[[Builder], Callable[[Value, Value], Value]]]] = [
    ("and", lambda b: b.and_),
    ("or", lambda b: b.or_),
    ("xor", lambda b: b.xor),
    ("add", lambda b: b.add),
    ("sub", lambda b: b.sub),
    ("mul", lambda b: b.mul),
    ("shl", lambda b: b.shl),
    ("lshr", lambda b: b.lshr),
    ("icmp eq", lambda b: partial(b.icmp, IPredicate.EQ)),
    ("icmp ne", lambda b: partial(b.icmp, IPredicate.NE)),
    ("icmp ugt", lambda b: partial(b.icmp, IPredicate.UGT)),
    ("icmp uge", lambda b: partial(b.icmp, IPredicate.UGE)),
    ("icmp ult", lambda b: partial(b.icmp, IPredicate.ULT)),
    ("icmp ule", lambda b: partial(b.icmp, IPredicate.ULE)),
    ("icmp sgt", lambda b: partial(b.icmp, IPredicate.SGT)),
    ("icmp sge", lambda b: partial(b.icmp, IPredicate.SGE)),
    ("icmp slt", lambda b: partial(b.icmp, IPredicate.SLT)),
    ("icmp sle", lambda b: partial(b.icmp, IPredicate.SLE))
]


class IntIntrinsicsTest(unittest.TestCase):
    def test_variable_variable(self) -> None:
        for (name, build) in _BINARY_INTRINSICS:
            with self.subTest(name):
                mod = SimpleModule("test " + name, 0, 0)
                source = mod.add_external_function(
                    "source", types.Function([], types.Int(64)))
                ty = types.BOOL if name.startswith("icmp") else types.Int(64)
                sink = mod.add_external_function(
                    "sink", types.Function([ty], types.VOID))

                x = mod.builder.call(source, [])
                y = mod.builder.call(source, [])
                z = build(mod.builder)(x, y)
                mod.builder.call(sink, [z])

                self.assertIn(f"%2 = {name} i64 %0, %1", mod.ir())

    def test_constant_variable(self) -> None:
        for (name, build) in _BINARY_INTRINSICS:
            with self.subTest(name):
                mod = SimpleModule("test " + name, 0, 0)
                source = mod.add_external_function(
                    "source", types.Function([], types.Int(64)))
                ty = types.BOOL if name.startswith("icmp") else types.Int(64)
                sink = mod.add_external_function(
                    "sink", types.Function([ty], types.VOID))

                x = mod.builder.call(source, [])
                y = build(mod.builder)(const(types.Int(64), 1), x)
                mod.builder.call(sink, [y])

                self.assertIn(f"%1 = {name} i64 1, %0", mod.ir())

    def test_neg(self) -> None:
        mod = SimpleModule("test_neg", 0, 0)
        source = mod.add_external_function(
            "source", types.Function([], types.Int(64)))
        sink = mod.add_external_function(
            "sink", types.Function([types.Int(64)], types.VOID))

        x = mod.builder.call(source, [])
        y = mod.builder.neg(x)
        mod.builder.call(sink, [y])

        self.assertIn("%1 = sub i64 0, %0", mod.ir())

    def test_type_mismatch(self) -> None:
        mod = SimpleModule("test_type_mismatch", 0, 0)
        with self.assertRaises(TypeError):
            mod.builder.add(const(types.Int(16), 2), const(types.Int(18), 2))

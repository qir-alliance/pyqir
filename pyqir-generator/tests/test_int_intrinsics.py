# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Callable, List, Tuple
from pyqir.generator import Builder, SimpleModule, Value, types
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
    ("icmp eq", lambda b: b.icmp_eq),
    ("icmp ne", lambda b: b.icmp_neq),
    ("icmp ugt", lambda b: b.icmp_ugt),
    ("icmp uge", lambda b: b.icmp_uge),
    ("icmp ule", lambda b: b.icmp_ule),
    ("icmp sgt", lambda b: b.icmp_sgt),
    ("icmp sge", lambda b: b.icmp_sge),
    ("icmp slt", lambda b: b.icmp_slt),
    ("icmp sle", lambda b: b.icmp_sle)
]


class IntIntrinsicsTest(unittest.TestCase):
    def test_variable_variable(self) -> None:
        for (name, build) in _BINARY_INTRINSICS:
            with self.subTest(name):
                mod = SimpleModule("test " + name, 0, 0)
                source = mod.add_external_function(
                    "source", types.Function([], types.INT))
                ty = types.BOOL if name.startswith("icmp") else types.INT
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
                    "source", types.Function([], types.INT))
                ty = types.BOOL if name.startswith("icmp") else types.INT
                sink = mod.add_external_function(
                    "sink", types.Function([ty], types.VOID))

                x = mod.builder.call(source, [])
                y = build(mod.builder)(1, x)
                mod.builder.call(sink, [y])

                self.assertIn(f"%1 = {name} i64 1, %0", mod.ir())

    def test_neg(self) -> None:
        mod = SimpleModule("test_neg", 0, 0)
        source = mod.add_external_function(
            "source", types.Function([], types.INT))
        sink = mod.add_external_function(
            "sink", types.Function([types.INT], types.VOID))

        x = mod.builder.call(source, [])
        y = mod.builder.neg(x)
        mod.builder.call(sink, [y])

        self.assertIn("%1 = sub i64 0, %0", mod.ir())

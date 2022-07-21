# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import SimpleModule, types
import unittest


class IntIntrinsicsTest(unittest.TestCase):
    def test_int_intrinsics(self) -> None:
        intrinsics = [
            ("and", lambda b: b.and_, None),
            ("or", lambda b: b.or_, None),
            ("xor", lambda b: b.xor, None),
            ("add", lambda b: b.add, None),
            ("sub", lambda b: b.sub, None),
            ("mul", lambda b: b.mul, None),
            ("shl", lambda b: b.shl, None),
            ("lshr", lambda b: b.lshr, None),
            ("icmp eq", lambda b: b.icmp_eq, 1),
            ("icmp ne", lambda b: b.icmp_neq, 1),
            ("icmp ugt", lambda b: b.icmp_ugt, 1),
            ("icmp uge", lambda b: b.icmp_uge, 1),
            ("icmp ule", lambda b: b.icmp_ule, 1),
            ("icmp sgt", lambda b: b.icmp_sgt, 1),
            ("icmp sge", lambda b: b.icmp_sge, 1),
            ("icmp slt", lambda b: b.icmp_slt, 1),
            ("icmp sle", lambda b: b.icmp_sle, 1)
        ]

        for (name, build, width) in intrinsics:
            with self.subTest(name):
                mod = SimpleModule("test " + name, 0, 0)
                source = mod.add_external_function(
                    "source", types.Function([], types.INT))
                ty = types.Integer(width) if width else types.INT
                sink = mod.add_external_function(
                    "sink", types.Function([ty], types.VOID))

                x = mod.builder.call(source, [])
                y = mod.builder.call(source, [])
                z = build(mod.builder)(x, y)
                mod.builder.call(sink, [z])

                self.assertIn(f"%2 = {name} i64 %0, %1", mod.ir())

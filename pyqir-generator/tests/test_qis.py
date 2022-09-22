# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, Types, Value, const
from typing import Callable, List, Tuple, Union
import unittest


class QisTest(unittest.TestCase):
    def test_single(self) -> None:
        cases: List[
            Tuple[str, Callable[[BasicQisBuilder], Callable[[Value], None]]]
        ] = [
            ("h", lambda qis: qis.h),
            ("reset", lambda qis: qis.reset),
            ("s", lambda qis: qis.s),
            ("t", lambda qis: qis.t),
            ("x", lambda qis: qis.x),
            ("y", lambda qis: qis.y),
            ("z", lambda qis: qis.z),
        ]

        for name, gate in cases:
            with self.subTest(name):
                mod = SimpleModule("test_single", 1, 0)
                qis = BasicQisBuilder(mod.builder)
                gate(qis)(mod.qubits[0])
                call = f"call void @__quantum__qis__{name}__body(%Qubit* null)"
                self.assertIn(call, mod.ir())

    def test_controlled(self) -> None:
        cases: List[
            Tuple[str, Callable[[BasicQisBuilder], Callable[[Value, Value], None]]]
        ] = [
            ("cnot", lambda qis: qis.cx),
            ("cz", lambda qis: qis.cz),
        ]

        for name, gate in cases:
            with self.subTest(name):
                mod = SimpleModule("test_controlled", 2, 0)
                qis = BasicQisBuilder(mod.builder)
                gate(qis)(mod.qubits[0], mod.qubits[1])
                call = f"call void @__quantum__qis__{name}__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))"
                self.assertIn(call, mod.ir())

    def test_adjoint(self) -> None:
        cases: List[
            Tuple[str, Callable[[BasicQisBuilder], Callable[[Value], None]]]
        ] = [
            ("s", lambda qis: qis.s_adj),
            ("t", lambda qis: qis.t_adj),
        ]

        for name, gate in cases:
            with self.subTest(name):
                mod = SimpleModule("test_adjoint", 1, 0)
                qis = BasicQisBuilder(mod.builder)
                gate(qis)(mod.qubits[0])
                call = f"call void @__quantum__qis__{name}__adj(%Qubit* null)"
                self.assertIn(call, mod.ir())

    def test_rotated(self) -> None:
        cases: List[
            Tuple[
                str,
                Callable[
                    [BasicQisBuilder], Callable[[Union[Value, float], Value], None]
                ],
            ]
        ] = [
            ("rx", lambda qis: qis.rx),
            ("ry", lambda qis: qis.ry),
            ("rz", lambda qis: qis.rz),
        ]

        values: List[Callable[[Types], Union[Value, float]]] = [
            lambda types: const(types.double, 1.0),
            lambda _: 1.0,
        ]

        for name, gate in cases:
            for value in values:
                with self.subTest(f"{name} ({value})"):
                    mod = SimpleModule("test_rotated", 1, 0)
                    qis = BasicQisBuilder(mod.builder)
                    gate(qis)(value(mod.types), mod.qubits[0])
                    call = f"call void @__quantum__qis__{name}__body(double 1.000000e+00, %Qubit* null)"
                    self.assertIn(call, mod.ir())

    def test_mz(self) -> None:
        mod = SimpleModule("test_mz", 1, 1)
        qis = BasicQisBuilder(mod.builder)
        qis.mz(mod.qubits[0], mod.results[0])
        call = f"call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)"
        self.assertIn(call, mod.ir())

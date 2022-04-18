# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule
import unittest


class QisTest(unittest.TestCase):
    def test_single(self) -> None:
        cases = [
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
        cases = [
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
        cases = [
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
        cases = [
            ("rx", lambda qis: qis.rx),
            ("ry", lambda qis: qis.ry),
            ("rz", lambda qis: qis.rz),
        ]

        for name, gate in cases:
            with self.subTest(name):
                mod = SimpleModule("test_rotated", 1, 0)
                qis = BasicQisBuilder(mod.builder)
                gate(qis)(0.0, mod.qubits[0])
                call = f"call void @__quantum__qis__{name}__body(double 0.000000e+00, %Qubit* null)"
                self.assertIn(call, mod.ir())

    def test_m(self) -> None:
        mod = SimpleModule("test_m", 1, 1)
        mod.use_static_result_alloc(False)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])
        call = f"call %Result* @__quantum__qis__m__body(%Qubit* null)"
        self.assertIn(call, mod.ir())

    def test_mz(self) -> None:
        mod = SimpleModule("test_mz", 1, 1)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])
        call = f"call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)"
        self.assertIn(call, mod.ir())

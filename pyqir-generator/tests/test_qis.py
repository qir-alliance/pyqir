# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule
import unittest


class QisTest(unittest.TestCase):
    def test_cx(self) -> None:
        mod = SimpleModule("test", 2, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.cx(mod.qubits[0], mod.qubits[1])
        self.assertIn(_controlled("cnot"), mod.ir())

    def test_cz(self) -> None:
        mod = SimpleModule("test", 2, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.cz(mod.qubits[0], mod.qubits[1])
        self.assertIn(_controlled("cz"), mod.ir())

    def test_h(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.h(mod.qubits[0])
        self.assertIn(_single("h"), mod.ir())

    def test_m(self) -> None:
        mod = SimpleModule("test", 1, 1)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])
        self.assertIn(_measured("m"), mod.ir())

    def test_reset(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.reset(mod.qubits[0])
        self.assertIn(_single("reset"), mod.ir())

    def test_rx(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.rx(0.0, mod.qubits[0])
        self.assertIn(_rotated("rx"), mod.ir())

    def test_ry(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.ry(0.0, mod.qubits[0])
        self.assertIn(_rotated("ry"), mod.ir())

    def test_rz(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.rz(0.0, mod.qubits[0])
        self.assertIn(_rotated("rz"), mod.ir())

    def test_s(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.s(mod.qubits[0])
        self.assertIn(_single("s"), mod.ir())

    def test_s_adj(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.s_adj(mod.qubits[0])
        self.assertIn(_adjoint("s"), mod.ir())

    def test_t(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.t(mod.qubits[0])
        self.assertIn(_single("t"), mod.ir())

    def test_t_adj(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.t_adj(mod.qubits[0])
        self.assertIn(_adjoint("t"), mod.ir())

    def test_x(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.x(mod.qubits[0])
        self.assertIn(_single("x"), mod.ir())

    def test_y(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.y(mod.qubits[0])
        self.assertIn(_single("y"), mod.ir())

    def test_z(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        qis.z(mod.qubits[0])
        self.assertIn(_single("z"), mod.ir())


def _single(gate: str) -> str:
    call = f"call void @__quantum__qis__{gate}__body"
    return f"{call}(%Qubit* null)"


def _controlled(gate: str) -> str:
    call = f"call void @__quantum__qis__{gate}__body"
    return f"{call}(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))"


def _adjoint(gate: str) -> str:
    call = f"call void @__quantum__qis__{gate}__adj"
    return f"{call}(%Qubit* null)"


def _rotated(gate: str) -> str:
    call = f"call void @__quantum__qis__{gate}__body"
    return f"{call}(double 0.000000e+00, %Qubit* null)"


def _measured(gate: str) -> str:
    call = f"call %Result* @__quantum__qis__{gate}__body"
    return f"{call}(%Qubit* null)"

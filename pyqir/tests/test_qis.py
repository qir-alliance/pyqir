# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Callable, Union

import pytest

import pyqir
from pyqir import BasicQisBuilder, Context, SimpleModule, Type, Value


@pytest.mark.parametrize(
    "name, get_gate",
    [
        ("h", lambda qis: qis.h),
        ("reset", lambda qis: qis.reset),
        ("s", lambda qis: qis.s),
        ("t", lambda qis: qis.t),
        ("x", lambda qis: qis.x),
        ("y", lambda qis: qis.y),
        ("z", lambda qis: qis.z),
    ],
)
def test_single(
    name: str, get_gate: Callable[[BasicQisBuilder], Callable[[Value], None]]
) -> None:
    mod = SimpleModule("test_single", 1, 0)
    qis = BasicQisBuilder(mod.builder)
    get_gate(qis)(mod.qubits[0])
    call = f"call void @__quantum__qis__{name}__body(%Qubit* null)"
    assert call in mod.ir()


@pytest.mark.parametrize(
    "name, get_gate",
    [
        ("swap", lambda qis: qis.swap),
    ],
)
def test_two_qubit_gates(
    name: str, get_gate: Callable[[BasicQisBuilder], Callable[[Value, Value], None]]
) -> None:
    mod = SimpleModule("test_two_qubit_gates", 2, 0)
    qis = BasicQisBuilder(mod.builder)
    get_gate(qis)(mod.qubits[0], mod.qubits[1])
    call = f"call void @__quantum__qis__{name}__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))"
    assert call in mod.ir()


@pytest.mark.parametrize(
    "name, get_gate",
    [
        ("cnot", lambda qis: qis.cx),
        ("cz", lambda qis: qis.cz),
    ],
)
def test_controlled(
    name: str, get_gate: Callable[[BasicQisBuilder], Callable[[Value, Value], None]]
) -> None:
    mod = SimpleModule("test_controlled", 2, 0)
    qis = BasicQisBuilder(mod.builder)
    get_gate(qis)(mod.qubits[0], mod.qubits[1])
    call = f"call void @__quantum__qis__{name}__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))"
    assert call in mod.ir()


@pytest.mark.parametrize(
    "name, get_gate",
    [
        ("s", lambda qis: qis.s_adj),
        ("t", lambda qis: qis.t_adj),
    ],
)
def test_adjoint(
    name: str, get_gate: Callable[[BasicQisBuilder], Callable[[Value], None]]
) -> None:
    mod = SimpleModule("test_adjoint", 1, 0)
    qis = BasicQisBuilder(mod.builder)
    get_gate(qis)(mod.qubits[0])
    call = f"call void @__quantum__qis__{name}__adj(%Qubit* null)"
    assert call in mod.ir()


@pytest.mark.parametrize(
    "name, get_gate",
    [
        ("rx", lambda qis: qis.rx),
        ("ry", lambda qis: qis.ry),
        ("rz", lambda qis: qis.rz),
    ],
)
@pytest.mark.parametrize(
    "get_value",
    [
        lambda context: pyqir.const(Type.double(context), 1.0),
        lambda _: 1.0,
    ],
)
def test_rotated(
    name: str,
    get_gate: Callable[[BasicQisBuilder], Callable[[Union[Value, float], Value], None]],
    get_value: Callable[[Context], Union[Value, float]],
) -> None:
    mod = SimpleModule("test_rotated", 1, 0)
    qis = BasicQisBuilder(mod.builder)
    get_gate(qis)(get_value(mod.context), mod.qubits[0])
    call = f"call void @__quantum__qis__{name}__body(double 1.000000e+00, %Qubit* null)"
    assert call in mod.ir()


def test_mz() -> None:
    mod = SimpleModule("test_mz", 1, 1)
    qis = BasicQisBuilder(mod.builder)
    qis.mz(mod.qubits[0], mod.results[0])
    call = f"call void @__quantum__qis__mz__body(%Qubit* null, %Result* null)"
    assert call in mod.ir()

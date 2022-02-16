# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator.module import SimpleModule
from pyqir.generator.qis import BasicQisBuilder


def controlled(gate: str) -> str:
    root = f'call void @__quantum__qis__{gate}__body'
    controlled = f'{root}(%Qubit* %qubit0, %Qubit* %qubit1)'
    return controlled


def single(gate: str) -> str:
    root = f'call void @__quantum__qis__{gate}__body'
    single = f'{root}(%Qubit* %qubit0)'
    return single


def adjoint(gate: str) -> str:
    root = f'call void @__quantum__qis__{gate}__adj'
    single = f'{root}(%Qubit* %qubit0)'
    return single


def rotated(gate: str) -> str:
    root = f'call void @__quantum__qis__{gate}__body'
    single = f'{root}(double 0.000000e+00, %Qubit* %qubit0)'
    return single


def measured(gate: str) -> str:
    root = f'call %Result* @__quantum__qis__{gate}__body'
    single = f'{root}(%Qubit* %qubit0)'
    return single


def create_context(num_qubits=0, num_results=0):
    module = SimpleModule("test", num_qubits, num_results)
    qis = BasicQisBuilder(module.builder)
    return (qis, module, module.qubits, module.results)


def test_cx() -> None:
    (qis, module, qubits, _) = create_context(2)
    qis.cx(qubits[0], qubits[1])
    ir = module.ir()
    assert controlled('cnot') in ir


def test_cz() -> None:
    (qis, module, qubits, _) = create_context(2)
    qis.cz(qubits[0], qubits[1])
    ir = module.ir()
    assert controlled('cz') in ir


def test_h() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.h(qubits[0])
    ir = module.ir()
    assert single('h') in ir


def test_m() -> None:
    (qis, module, qubits, results) = create_context(1, 1)
    qis.m(qubits[0], results[0])
    ir = module.ir()
    assert measured('m') in ir


def test_reset() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.reset(qubits[0])
    ir = module.ir()
    assert single('reset') in ir


def test_rx() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.rx(0.0, qubits[0])
    ir = module.ir()
    assert rotated('rx') in ir


def test_ry() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.ry(0.0, qubits[0])
    ir = module.ir()
    assert rotated('ry') in ir


def test_rz() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.rz(0.0, qubits[0])
    ir = module.ir()
    assert rotated('rz') in ir


def test_s() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.s(qubits[0])
    ir = module.ir()
    assert single('s') in ir


def test_s_adj() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.s_adj(qubits[0])
    ir = module.ir()
    assert adjoint('s') in ir


def test_t() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.t(qubits[0])
    ir = module.ir()
    assert single('t') in ir


def test_t_adj() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.t_adj(qubits[0])
    ir = module.ir()
    assert adjoint('t') in ir


def test_x() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.x(qubits[0])
    ir = module.ir()
    assert single('x') in ir


def test_y() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.y(qubits[0])
    ir = module.ir()
    assert single('y') in ir


def test_z() -> None:
    (qis, module, qubits, _) = create_context(1)
    qis.z(qubits[0])
    ir = module.ir()
    assert single('z') in ir

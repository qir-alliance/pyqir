# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator.module import SimpleModule
from pyqir.generator.qis import BasicQisBuilder


def test_bell() -> None:
    module = SimpleModule("Bell circuit", num_qubits=2, num_results=2)
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])
    qis.m(module.qubits[0], module.results[0])
    qis.m(module.qubits[1], module.results[1])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'Bell circuit'")


def test_bell_no_measure() -> None:
    module = SimpleModule("Bell circuit", num_qubits=2, num_results=0)
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'Bell circuit'")


def test_bernstein_vazirani() -> None:
    module = SimpleModule("Bernstein-Vazirani", num_qubits=6, num_results=5)
    qis = BasicQisBuilder(module.builder)
    inputs = module.qubits[:5]
    target = module.qubits[5]
    outputs = module.results

    qis.x(target)

    qis.h(inputs[0])
    qis.h(inputs[1])
    qis.h(inputs[2])
    qis.h(inputs[3])
    qis.h(inputs[4])

    qis.h(target)

    qis.cx(inputs[1], target)
    qis.cx(inputs[3], target)
    qis.cx(inputs[4], target)

    qis.h(inputs[0])
    qis.h(inputs[1])
    qis.h(inputs[2])
    qis.h(inputs[3])
    qis.h(inputs[4])

    qis.m(inputs[0], outputs[0])
    qis.m(inputs[1], outputs[1])
    qis.m(inputs[2], outputs[2])
    qis.m(inputs[3], outputs[3])
    qis.m(inputs[4], outputs[4])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'Bernstein-Vazirani'")


def test_all_gates() -> None:
    module = SimpleModule("All Gates", num_qubits=5, num_results=5)
    qis = BasicQisBuilder(module.builder)
    q = module.qubits[:4]
    control = module.qubits[4]
    c = module.results

    qis.cx(q[0], control)
    qis.cz(q[1], control)
    qis.h(q[0])
    qis.reset(q[0])
    qis.rx(15.0, q[1])
    qis.ry(16.0, q[2])
    qis.rz(17.0, q[3])
    qis.s(q[0])
    qis.s_adj(q[1])
    qis.t(q[2])
    qis.t_adj(q[3])
    qis.x(q[0])
    qis.y(q[1])
    qis.z(q[2])

    qis.m(q[0], c[0])
    qis.m(q[1], c[1])
    qis.m(q[2], c[2])
    qis.m(q[3], c[3])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'All Gates'")

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import BasicQisBuilder, SimpleModule, Module
import pytest

def bell() -> SimpleModule:
    module = SimpleModule("Bell circuit", 2, 2, "bell")
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])
    qis.mz(module.qubits[0], module.results[0])
    qis.mz(module.qubits[1], module.results[1])

    return module


def bell_no_measure() -> SimpleModule:
    module = SimpleModule("Bell circuit", 2, 0, "bell_no_measure")
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])

    return module


def bernstein_vazirani() -> SimpleModule:
    module = SimpleModule("Bernstein-Vazirani", 6, 5, "Bernstein_Vazirani")
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

    qis.mz(inputs[0], outputs[0])
    qis.mz(inputs[1], outputs[1])
    qis.mz(inputs[2], outputs[2])
    qis.mz(inputs[3], outputs[3])
    qis.mz(inputs[4], outputs[4])

    return module

def test_foo() -> None:
    a, b, c = bell(), bell_no_measure(), bernstein_vazirani()

    names, module = Module.link([Module.from_bitcode(a.bitcode()), Module.from_bitcode(b.bitcode()), Module.from_bitcode(c.bitcode())], "combined")
    ir = str(module)
    print(names)
    print(ir)
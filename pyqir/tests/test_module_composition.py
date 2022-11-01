# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from importlib.metadata import entry_points
from pyqir import BasicQisBuilder, SimpleModule, Context, Module, is_entry_point
import pytest

def bell(m) -> SimpleModule:
    module = SimpleModule(m, 2, 2, "bell")
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])
    qis.mz(module.qubits[0], module.results[0])
    qis.mz(module.qubits[1], module.results[1])

    return module


def bell_no_measure(m) -> SimpleModule:
    module = SimpleModule(m, 2, 0, "bell")
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])

    return module


def bernstein_vazirani(m) -> SimpleModule:
    module = SimpleModule(m, 6, 5, "Bernstein_Vazirani")
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

def test_link() -> None:
    parent = Module(Context(), "batch")
    simple_modules = [bell(parent), bell_no_measure(parent), bernstein_vazirani(parent)]
    entry_points = [x.entry_point for x in simple_modules]
    
    for f in parent.functions:
        print(f.name)
    print("entry points:")
    print(entry_points)
    
    ir = str(parent)
    #print(entry_point_names)
    #print(ir)
#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from pathlib import Path
from typing import List

from pyqir import (
    BasicQisBuilder,
    Call,
    Context,
    Module,
    is_entry_point,
    is_qubit_type,
    is_result_type,
    qubit_id,
    required_num_qubits,
    required_num_results,
    result_id,
    SimpleModule,
)

# Create a simple entry point defining the sequence
def create_bernstein_vazirani() -> SimpleModule:
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

    qis.mz(inputs[0], outputs[0])
    qis.mz(inputs[1], outputs[1])
    qis.mz(inputs[2], outputs[2])
    qis.mz(inputs[3], outputs[3])
    qis.mz(inputs[4], outputs[4])

    return module

# Convert a QIS operation to a simple string representation
def gate_inst_to_str(inst) -> str:
    raw = inst.callee.name.strip("__quantum__qis__").strip("__body")
    args = []
    for arg in inst.args:
        if is_qubit_type(arg.type):
            args.append(str(qubit_id(arg)))
        elif is_result_type(arg.type):
            args.append(str(result_id(arg)))
        else:
            args.append(str(arg))
    arg_str = ", ".join(args)
    call = f"{raw}({arg_str})"
    return call

# We can get the bitcode from the simple module
bitcode = create_bernstein_vazirani().bitcode()

# We can also load a new module from that bitcode
mod = Module.from_bitcode(Context(), bitcode)

# Find the entry point and make sure it has the proper attributes
entry_point = next(filter(is_entry_point, mod.functions))
assert required_num_qubits(entry_point) == 6
assert required_num_results(entry_point) == 5

# Get the list of QIS calls
calls: List[Call] = []

for block in entry_point.basic_blocks:
    for inst in block.instructions:
        if isinstance(inst, Call):
            calls.append(inst)

# Convert the calls into simple representations
call_strs = list(map(gate_inst_to_str, calls))
assert call_strs == ['x(5)', 'h(0)', 'h(1)', 'h(2)', 'h(3)', 'h(4)', 'h(5)', 'cnot(1, 5)', 'cnot(3, 5)', 'cnot(4, 5)', 'h(0)', 'h(1)', 'h(2)', 'h(3)', 'h(4)', 'z(0, 0)', 'z(1, 1)', 'z(2, 2)', 'z(3, 3)', 'z(4, 4)']

if __name__ == "__main__":
    print(call_strs)
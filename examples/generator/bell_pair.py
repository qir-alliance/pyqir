#!/usr/bin/env python3

# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_generator.instruction import *
from pyqir_generator.module import Module, Register

# This operation creates a Bell pair and returns the result of measuring each
# qubit.
instructions = [
    H("qubit0"),
    Cx("qubit0", "qubit1"),

    M("qubit0", "output0"),
    M("qubit1", "output1"),
]

bell_pair = Module(
    name="Bell",
    bits=[Register("output", 2)],
    qubits=[Register("qubit", 2)],
    instructions=instructions
)

if __name__ == "__main__":
    print(bell_pair.ir())

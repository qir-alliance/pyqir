#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_generator.module import SimpleModule
from pyqir_generator.qis import BasicQisBuilder

# This module creates a Bell pair and returns the result of measuring each
# qubit.
bell = SimpleModule("bell", num_qubits=2, num_results=2)
qis = BasicQisBuilder(bell.builder)

qis.h(bell.qubits[0])
qis.cx(bell.qubits[0], bell.qubits[1])
qis.m(bell.qubits[0], bell.results[0])
qis.m(bell.qubits[1], bell.results[1])

if __name__ == "__main__":
    print(bell.ir())

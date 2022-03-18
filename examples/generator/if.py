# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule

mod = SimpleModule("if", num_qubits=2, num_results=2)
qis = BasicQisBuilder(mod.builder)

# Manually reset a qubit by measuring it and applying the X gate if the result
# is one.
qis.h(mod.qubits[0])
qis.m(mod.qubits[0], mod.results[0])
qis.if_result(mod.results[0], lambda: qis.x(mod.qubits[0]))

# Branches can be nested, for example, to execute an instruction only if both
# results are one.
for i in range(2):
    qis.h(mod.qubits[i])
    qis.m(mod.qubits[i], mod.results[i])


def x_both():
    qis.x(mod.qubits[0])
    qis.x(mod.qubits[1])


qis.if_result(mod.results[0], lambda: qis.if_result(mod.results[1], x_both))

if __name__ == "__main__":
    print(mod.ir())

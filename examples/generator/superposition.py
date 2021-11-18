#!/usr/bin/env python3

# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_generator import QirBuilder


class Superposition:
    """
    This operation sets 2 qubits in superposition and returns the result
    of measuring them.
    """

    def __init__(self, entangle: bool):
        """
        :param entangle: Whether the two qubits should be entangled.
            If "true", the two qubits are entangled such that the state of both
            is one of the Bell states: | x 〉| y 〉 = 1/sqrt(2) [|0〉|0〉 + |1〉|1〉]
            If "false", the two qubits are entangled such that the state is:
            | x 〉| y 〉 = 1/4 [|0〉|0〉 + |0〉|1〉 + |1〉|0〉 + |1〉|1〉]
        :type entangle: bool
        """
        self.builder = QirBuilder("Bell")
        self.entangle = entangle
        self.apply()

    def apply(self):
        self.builder.add_quantum_register("qubit", 2)
        self.builder.add_classical_register("output", 2)
        self.builder.h("qubit0")
        
        if(self.entangle):
            self.builder.cx("qubit0", "qubit1")
        else:
            self.builder.h("qubit1")

        self.builder.m("qubit0", "output0")
        self.builder.m("qubit1", "output1")

    def write_ir_file(self, file_path: str):
        self.builder.write(file_path)

    def get_ir_string(self) -> str:
        return self.builder.get_ir_string()

    def generate_ir_file(file_path: str, entangle):
        instance = Superposition(entangle)
        instance.write(file_path)


if __name__ == "__main__":
    print(Superposition(True).get_ir_string())

#!/usr/bin/env python3

# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_generator import QirBuilder


class BellPair:
    """
    This operation creates a Bell pair and returns the result
    of measuring each qubit.
    """

    def __init__(self):
        self.builder = QirBuilder("Bell")
        self.apply()

    def apply(self):
        self.builder.add_quantum_register("qubit", 2)
        self.builder.add_classical_register("output", 2)

        self.builder.h("qubit0")
        self.builder.cx("qubit0", "qubit1")

        self.builder.m("qubit0", "output0")
        self.builder.m("qubit1", "output1")

    def write_ir_file(self, file_path: str):
        self.builder.write(file_path)

    def get_ir_string(self) -> str:
        return self.builder.get_ir_string()

    def generate_ir_file(file_path: str):
        instance = BellPair()
        instance.write(file_path)


if __name__ == "__main__":
    print(BellPair().get_ir_string())

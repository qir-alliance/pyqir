from pyqir_jit import *
import pytest

class GateLogger(GateSet):
    def __init__(self):
        # call parent class constructor
        super().__init__()
        self.number_of_registers = 0
        self.instructions = []

    def cx(self, control: str, target: str):
        self.instructions.append(f"cx control[{control}], target[{target}]")

    def cz(self, control: str, target: str):
        self.instructions.append(f"cz control[{control}], target[{target}]")

    def h(self, target: str):
        self.instructions.append(f"h qubit[{target}]")

    def m(self, qubit: str, target: str):
        self.instructions.append(f"m qubit[{qubit}] => out[{target}]")

    def reset(self, target: str):
        self.instructions.append(f"reset {target}")

    def rx(self, theta: float, qubit: str):
        self.instructions.append(f"rx theta[{theta}] qubit[{qubit}]")

    def ry(self, theta: float, qubit: str):
        self.instructions.append(f"ry theta[{theta}] qubit[{qubit}]")

    def rz(self, theta: float, qubit: str):
        self.instructions.append(f"rz theta[{theta}] qubit[{qubit}]")

    def s(self, qubit: str):
        self.instructions.append(f"s qubit[{qubit}]")

    def s_adj(self, qubit: str):
        self.instructions.append(f"s_adj qubit[{qubit}]")

    def t(self, qubit: str):
        self.instructions.append(f"t qubit[{qubit}]")

    def t_adj(self, qubit: str):
        self.instructions.append(f"t_adj qubit[{qubit}]")

    def x(self, qubit: str):
        self.instructions.append(f"x qubit[{qubit}]")

    def y(self, qubit: str):
        self.instructions.append(f"y qubit[{qubit}]")

    def z(self, qubit: str):
        self.instructions.append(f"z qubit[{qubit}]")

    def dump_machine(self):
        self.instructions.append(f"dumpmachine")

    def finish(self, metadata: dict):
        print("finished")
        super().finish(metadata)
        self.number_of_registers = self.number_of_qubits

def test_bell_qir():
    file = "tests/bell_qir_measure.ll"
    qirjit = QirJit()
    generator = GateLogger()
    qirjit.eval(file, generator)

    assert len(generator.instructions) == 2
    assert str(generator.instructions[0]).startswith("h")
    assert str(generator.instructions[1]).startswith("cx")


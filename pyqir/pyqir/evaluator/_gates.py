# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Any, Dict, List


class GateSet:
    """
    Defines the quantum circuit operations which may be registered for
    callbacks during evaluation of QIR
    """

    def cx(self, control: str, target: str) -> None:
        pass

    def cz(self, control: str, target: str) -> None:
        pass

    def h(self, target: str) -> None:
        pass

    def m(self, qubit: str, target: str) -> None:
        pass

    def mz(self, qubit: str, target: str) -> None:
        pass

    def reset(self, target: str) -> None:
        pass

    def rx(self, theta: float, qubit: str) -> None:
        pass

    def ry(self, theta: float, qubit: str) -> None:
        pass

    def rz(self, theta: float, qubit: str) -> None:
        pass

    def s(self, qubit: str) -> None:
        pass

    def s_adj(self, qubit: str) -> None:
        pass

    def t(self, qubit: str) -> None:
        pass

    def t_adj(self, qubit: str) -> None:
        pass

    def x(self, qubit: str) -> None:
        pass

    def y(self, qubit: str) -> None:
        pass

    def z(self, qubit: str) -> None:
        pass

    def finish(self, metadata: Dict[str, Any]) -> None:
        """
        Called at the end of QIR evaluation supplying run metadata.
        """
        pass


class GateLogger(GateSet):
    """
    Records the quantum circuit operations executed during QIR evaluation.
    """

    number_of_qubits: int
    number_of_registers: int
    instructions: List[str]

    def __init__(self) -> None:
        self.number_of_qubits = 0
        self.number_of_registers = 0
        self.instructions = []

    def cx(self, control: str, target: str) -> None:
        self.instructions.append(f"cx qubit[{control}], qubit[{target}]")

    def cz(self, control: str, target: str) -> None:
        self.instructions.append(f"cz qubit[{control}], qubit[{target}]")

    def h(self, target: str) -> None:
        self.instructions.append(f"h qubit[{target}]")

    def m(self, qubit: str, target: str) -> None:
        self.instructions.append(f"m qubit[{qubit}] => out[{target}]")

    def mz(self, qubit: str, target: str) -> None:
        self.instructions.append(f"m qubit[{qubit}] => out[{target}]")

    def reset(self, target: str) -> None:
        self.instructions.append(f"reset {target}")

    def rx(self, theta: float, qubit: str) -> None:
        self.instructions.append(f"rx theta[{theta}] qubit[{qubit}]")

    def ry(self, theta: float, qubit: str) -> None:
        self.instructions.append(f"ry theta[{theta}] qubit[{qubit}]")

    def rz(self, theta: float, qubit: str) -> None:
        self.instructions.append(f"rz theta[{theta}] qubit[{qubit}]")

    def s(self, qubit: str) -> None:
        self.instructions.append(f"s qubit[{qubit}]")

    def s_adj(self, qubit: str) -> None:
        self.instructions.append(f"s_adj qubit[{qubit}]")

    def t(self, qubit: str) -> None:
        self.instructions.append(f"t qubit[{qubit}]")

    def t_adj(self, qubit: str) -> None:
        self.instructions.append(f"t_adj qubit[{qubit}]")

    def x(self, qubit: str) -> None:
        self.instructions.append(f"x qubit[{qubit}]")

    def y(self, qubit: str) -> None:
        self.instructions.append(f"y qubit[{qubit}]")

    def z(self, qubit: str) -> None:
        self.instructions.append(f"z qubit[{qubit}]")

    def finish(self, metadata: Dict[str, Any]) -> None:
        super().finish(metadata)
        self.number_of_qubits = metadata["number_of_qubits"]
        self.number_of_registers = self.number_of_qubits

    def print(self) -> None:
        print(f"qubits[{self.number_of_qubits}]")
        print(f"out[{self.number_of_registers}]")

        for instruction in self.instructions:
            print(instruction)

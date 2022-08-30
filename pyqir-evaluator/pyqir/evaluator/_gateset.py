# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Any, Dict


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

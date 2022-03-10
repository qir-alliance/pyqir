# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

class GateSet:
    """
    Defines the quantum circuit operations which may be registered for
    callbacks during evaluation of QIR
    """

    def cx(self, control: str, target: str):
        pass

    def cz(self, control: str, target: str):
        pass

    def h(self, target: str):
        pass

    def m(self, qubit: str, target: str):
        pass

    def reset(self, target: str):
        pass

    def rx(self, theta: float, qubit: str):
        pass

    def ry(self, theta: float, qubit: str):
        pass

    def rz(self, theta: float, qubit: str):
        pass

    def s(self, qubit: str):
        pass

    def s_adj(self, qubit: str):
        pass

    def t(self, qubit: str):
        pass

    def t_adj(self, qubit: str):
        pass

    def x(self, qubit: str):
        pass

    def y(self, qubit: str):
        pass

    def z(self, qubit: str):
        pass

    def finish(self, metadata: dict):
        """
        Called at the end of QIR evaluation supplying run metadata.
        """
        pass

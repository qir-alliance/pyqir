# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import Builder, Value
from typing import Callable, Optional, Union
import pyqir.qis as qis


class BasicQisBuilder:
    """An instruction builder that generates instructions from the basic quantum instruction set."""

    def __init__(self, builder: Builder) -> None:
        """
        Initializes a basic QIS builder.

        :param Builder builder: The IR Builder used to create the instructions
        """
        self._builder = builder

    def cx(self, control: Value, target: Value) -> None:
        """
        Inserts a controlled Pauli :math:`X` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        qis.cx(self._builder, control, target)

    def cz(self, control: Value, target: Value) -> None:
        """
        Inserts a controlled Pauli :math:`Z` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        qis.cz(self._builder, control, target)

    def h(self, qubit: Value) -> None:
        """
        Inserts a Hadamard gate.

        :param qubit: The target qubit.
        """
        qis.h(self._builder, qubit)

    def mz(self, qubit: Value, result: Value) -> None:
        """
        Inserts a Z-basis measurement operation.

        :param qubit: The qubit to measure.
        :param result: A result where the measurement result will be written to.
        """
        qis.mz(self._builder, qubit, result)

    def reset(self, qubit: Value) -> None:
        """
        Inserts a reset operation.

        :param qubit: The qubit to reset.
        """
        qis.reset(self._builder, qubit)

    def rx(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`x` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        qis.rx(self._builder, theta, qubit)

    def ry(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`y` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        qis.ry(self._builder, theta, qubit)

    def rz(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`z` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        qis.rz(self._builder, theta, qubit)

    def s(self, qubit: Value) -> None:
        """
        Inserts an :math:`S` gate.

        :param qubit: The target qubit.
        """
        qis.s(self._builder, qubit)

    def s_adj(self, qubit: Value) -> None:
        """
        Inserts an adjoint :math:`S` gate.

        :param qubit: The target qubit.
        """
        qis.s_adj(self._builder, qubit)

    def t(self, qubit: Value) -> None:
        """
        Inserts a :math:`T` gate.

        :param qubit: The target qubit.
        """
        qis.t(self._builder, qubit)

    def t_adj(self, qubit: Value) -> None:
        """
        Inserts an adjoint :math:`T` gate.

        :param qubit: The target qubit.
        """
        qis.t_adj(self._builder, qubit)

    def x(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`X` gate.

        :param qubit: The target qubit.
        """
        qis.x(self._builder, qubit)

    def y(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`Y` gate.

        :param qubit: The target qubit.
        """
        qis.y(self._builder, qubit)

    def z(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`Z` gate.

        :param qubit: The target qubit.
        """
        qis.z(self._builder, qubit)

    def if_result(
        self,
        cond: Value,
        one: Callable[[], None] = lambda: None,
        zero: Callable[[], None] = lambda: None,
    ) -> None:
        """
        Inserts a branch conditioned on a measurement result.

        Instructions inserted when ``one`` is called will be inserted into the one branch.
        Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
        and zero callables should use this module's builder to build instructions.

        :param cond: The result condition to branch on.
        :param one: A callable that inserts instructions for the branch where the result is one.
        :param zero: A callable that inserts instructions for the branch where the result is zero.
        """
        qis.if_result(self._builder, cond, one, zero)

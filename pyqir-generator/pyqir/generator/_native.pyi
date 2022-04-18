# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import types
from pyqir.generator._values import Value
from typing import Callable, Sequence, Tuple


class Qubit:
    """A qubit identifier."""
    ...


class ResultRef:
    """A mutable reference cell that holds a measurement result."""
    ...


class Function:
    """A callable value for a module function."""
    ...


class Builder:
    """An instruction builder."""

    def call(self, function: Function, args: Sequence[Value]) -> None:
        """
        Builds a call instruction.

        :param function: The function to call.
        :param args: The arguments to the function.
        """
        ...


class SimpleModule:
    """
    A simple module represents an executable QIR program with these
    restrictions:

    - There is one global quantum register and one global classical register.
      Both are allocated automatically with a fixed size before the program
      starts.
    - There is only a single function that runs as the entry point.
    """

    def __init__(
        self,
        name: str,
        num_qubits: int,
        num_results: int,
    ) -> None:
        """
        Initializes the module with a name and the number of qubits and results
        in the quantum and classical registers.

        :param name: The name of the module.
        :param num_qubits: The size of the global quantum register.
        :param num_results: The size of the global classical register.
        """
        ...

    @property
    def qubits(self) -> Tuple[Qubit, ...]:
        """A sequence of qubits representing the global quantum register."""
        ...

    @property
    def results(self) -> Tuple[ResultRef, ...]:
        """
        A sequence of result references representing the global
        classical register.
        """
        ...

    @property
    def builder(self) -> Builder:
        """The instruction builder."""
        ...

    def ir(self) -> str:
        """Emits the LLVM IR for the module as plain text."""
        ...

    def bitcode(self) -> bytes:
        """Emits the LLVM bitcode for the module as a sequence of bytes."""
        ...

    def add_external_function(self, name: str, ty: types.Function) -> Function:
        """
        Adds a declaration for an externally linked function to the module.

        :param name: The name of the function.
        :param ty: The type of the function.
        :return: The function value.
        """
        ...

    def use_static_qubit_alloc(self, value: bool):
        """
        Configures code generation to use static or dynamic qubit allocation
        based on the provided value. Default is `True`.

        :param name: The value indicating to use static qubit
                     allocation (`True`) or dynamic allocation (`False`)
        """
        ...

    def use_static_result_alloc(self, value: bool):
        """
        Configures code generation to use static or dynamic result allocation
        based on the provided value. Default is `True`.

        :param name: The value indicating to use static result
                     allocation (`True`) or dynamic allocation (`False`)
        """
        ...


class BasicQisBuilder:
    """
    An instruction builder that generates instructions from the basic quantum
    instruction set.
    """

    def __init__(self, builder: Builder) -> None:
        """
        Initializes a new basic QIS instruction builder that wraps the given
        builder.
        """
        ...

    def cx(self, control: Qubit, target: Qubit) -> None:
        """
        Builds a controlled Pauli :math:`X` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...

    def cz(self, control: Qubit, target: Qubit) -> None:
        """
        Builds a controlled Pauli :math:`Z` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...

    def h(self, qubit: Qubit) -> None:
        """
        Builds a Hadamard gate.

        :param qubit: The target qubit.
        """
        ...

    def m(self, qubit: Qubit, result: ResultRef) -> None:
        """
        Builds a measurement operation.

        :param qubit: The qubit to measure.
        :param result: A result reference where the measurement result will be
                       written to.
        """
        ...

    def reset(self, qubit: Qubit) -> None:
        """
        Builds a reset operation.

        :param qubit: The qubit to reset.
        """
        ...

    def rx(self, theta: float, qubit: Qubit) -> None:
        """
        Builds a rotation gate about the :math:`x` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def ry(self, theta: float, qubit: Qubit) -> None:
        """
        Builds a rotation gate about the :math:`y` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def rz(self, theta: float, qubit: Qubit) -> None:
        """
        Builds a rotation gate about the :math:`z` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def s(self, qubit: Qubit) -> None:
        """
        Builds an :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...

    def s_adj(self, qubit: Qubit) -> None:
        """
        Builds an adjoint :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...

    def t(self, qubit: Qubit) -> None:
        """
        Builds a :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...

    def t_adj(self, qubit: Qubit) -> None:
        """
        Builds an adjoint :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...

    def x(self, qubit: Qubit) -> None:
        """
        Builds a Pauli :math:`X` gate.

        :param qubit: The target qubit.
        """
        ...

    def y(self, qubit: Qubit) -> None:
        """
        Builds a Pauli :math:`Y` gate.

        :param qubit: The target qubit.
        """
        ...

    def z(self, qubit: Qubit) -> None:
        """
        Builds a Pauli :math:`Z` gate.

        :param qubit: The target qubit.
        """
        ...

    def if_result(
        self,
        result: ResultRef,
        one: Callable[[], None] = ...,
        zero: Callable[[], None] = ...,
    ) -> None:
        """
        Builds a conditional branch on the result of a measurement.

        Dereferences the result reference, then evaluates the instructions
        built by ``one`` if the result is one, or the instructions built by
        ``zero`` if the result is zero. The one and zero callables should
        use this builder to build instructions.

        :param result: The result to branch on.
        :param one: A callable that builds instructions for the branch where
                    the result is one.
        :param zero: A callable that builds instructions for the branch where
                     the result is zero.
        """
        ...

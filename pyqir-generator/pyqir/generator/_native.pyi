# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator._builder import IPredicate
from pyqir.generator.types import Type
from typing import Callable, Optional, Sequence, Tuple, Union


def ir_to_bitcode(ir: str, module_name: Optional[str], source_file_name: Optional[str]) -> bytes:
    """
    Converts the supplied QIR string to its bitcode equivalent.

    :param ir: The QIR string to convert
    :param module_name: The name of the QIR module, default is "" if None
    :param source_file_name: The source file name of the QIR module. Unchanged if None
    :return: The equivalent bitcode as bytes.
    """
    ...


def bitcode_to_ir(bitcode: bytes, module_name: Optional[str], source_file_name: Optional[str]) -> str:
    """
    Converts the supplied bitcode to its QIR string equivalent.

    :param ir: The bitcode bytes to convert
    :param module_name: The name of the QIR module, default is "" if None
    :param source_file_name: The source file name of the QIR module. Unchanged if None
    :return: The equivalent QIR string.
    """
    ...


class ResultRef:
    """A mutable reference cell that holds a measurement result."""
    ...


class Function:
    """A QIR function."""
    ...


class Value:
    """A QIR value."""
    ...


def const(ty: Type, value: Union[int, float]) -> Value:
    """
    Creates a constant QIR value.

    :param ty: The type of the value.
    :param value: A Python value that will be converted into a QIR value.
    :returns: The constant QIR value.
    """
    ...


class Builder:
    """An instruction builder."""

    def neg(self, value: Value) -> Value: ...

    def and_(self, lhs: Value, rhs: Value) -> Value: ...

    def or_(self, lhs: Value, rhs: Value) -> Value: ...

    def xor(self, lhs: Value, rhs: Value) -> Value: ...

    def add(self, lhs: Value, rhs: Value) -> Value: ...

    def sub(self, lhs: Value, rhs: Value) -> Value: ...

    def mul(self, lhs: Value, rhs: Value) -> Value: ...

    def shl(self, lhs: Value, rhs: Value) -> Value: ...

    def lshr(self, lhs: Value, rhs: Value) -> Value: ...

    def icmp(self, predicate: IPredicate, lhs: Value, rhs: Value) -> Value: ...

    def call(
        self,
        function: Function,
        args: Sequence[Union[Value, ResultRef, bool, int, float]]
    ) -> Optional[Value]:
        """
        Adds a call instruction.

        :param function: The function to call.
        :param args: The arguments to the function.
        :returns: The return value, or None if the function has a void return type.
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
    def qubits(self) -> Tuple[Value, ...]:
        """A sequence of qubits representing the global quantum register."""
        ...

    @property
    def results(self) -> Tuple[ResultRef, ...]:
        """
        A sequence of result references representing the global classical
        register.
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

    def add_external_function(self, name: str, ty: Type) -> Function:
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

    def cx(self, control: Value, target: Value) -> None:
        """
        Builds a controlled Pauli :math:`X` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...

    def cz(self, control: Value, target: Value) -> None:
        """
        Builds a controlled Pauli :math:`Z` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...

    def h(self, qubit: Value) -> None:
        """
        Builds a Hadamard gate.

        :param qubit: The target qubit.
        """
        ...

    def m(self, qubit: Value, result: ResultRef) -> None:
        """
        Builds a measurement operation.

        :param qubit: The qubit to measure.
        :param result: A result reference where the measurement result will be
                       written to.
        """
        ...

    def reset(self, qubit: Value) -> None:
        """
        Builds a reset operation.

        :param qubit: The qubit to reset.
        """
        ...

    def rx(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Builds a rotation gate about the :math:`x` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def ry(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Builds a rotation gate about the :math:`y` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def rz(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Builds a rotation gate about the :math:`z` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def s(self, qubit: Value) -> None:
        """
        Builds an :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...

    def s_adj(self, qubit: Value) -> None:
        """
        Builds an adjoint :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...

    def t(self, qubit: Value) -> None:
        """
        Builds a :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...

    def t_adj(self, qubit: Value) -> None:
        """
        Builds an adjoint :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...

    def x(self, qubit: Value) -> None:
        """
        Builds a Pauli :math:`X` gate.

        :param qubit: The target qubit.
        """
        ...

    def y(self, qubit: Value) -> None:
        """
        Builds a Pauli :math:`Y` gate.

        :param qubit: The target qubit.
        """
        ...

    def z(self, qubit: Value) -> None:
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

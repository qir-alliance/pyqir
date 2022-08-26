# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator._builder import IntPredicate
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

    def and_(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a bitwise logical and instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The result.
        """
        ...

    def or_(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a bitwise logical or instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The result.
        """
        ...

    def xor(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a bitwise logical exclusive or instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The result.
        """
        ...

    def add(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts an addition instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The sum.
        """
        ...

    def sub(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a subtraction instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The difference.
        """
        ...

    def mul(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a multiplication instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The product.
        """
        ...

    def shl(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a shift left instruction.

        :param lhs: The value to shift.
        :param rhs: The number of bits to shift by.
        :returns: The result.
        """
        ...

    def lshr(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a logical (zero fill) shift right instruction.

        :param lhs: The value to shift.
        :param rhs: The number of bits to shift by.
        :returns: The result.
        """
        ...

    def icmp(self, pred: IntPredicate, lhs: Value, rhs: Value) -> Value:
        """
        Inserts an integer comparison instruction.

        :param pred: The predicate to compare by.
        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :return: The boolean result.
        """
        ...

    def call(
        self,
        function: Function,
        args: Sequence[Union[Value, bool, int, float]]
    ) -> Optional[Value]:
        """
        Inserts a call instruction.

        :param function: The function to call.
        :param args: The arguments to the function.
        :returns: The return value, or None if the function has a void return type.
        """
        ...

    def if_(self, cond: Value, true: Callable[[], None] = ..., false: Callable[[], None] = ...) -> None:
        """
        Inserts a branch conditioned on a boolean.

        Instructions inserted when ``true`` is called will be inserted into the
        true branch. Instructions inserted when ``false`` is called will be
        inserted into the false branch. The true and false callables should use
        this module's builder to build instructions.

        :param cond: The boolean condition to branch on.
        :param true: A callable that inserts instructions for the branch where
                    the condition is true.
        :param false: A callable that inserts instructions for the branch where
                     the condition is false.
        """
        ...


class SimpleModule:
    """
    A simple module represents an executable QIR program with these
    restrictions:

    - There is one global qubit register and one global result register. Both
      are statically allocated with a fixed size.
    - There is only a single function that runs as the entry point.
    """

    def __init__(
        self,
        name: str,
        num_qubits: int,
        num_results: int,
    ) -> None:
        """
        Initializes the module.

        :param name: The name of the module.
        :param num_qubits: The number of statically allocated qubits.
        :param num_results: The number of statically allocated results.
        """
        ...

    @property
    def qubits(self) -> Tuple[Value, ...]:
        """The global qubit register."""
        ...

    @property
    def results(self) -> Tuple[Value, ...]:
        """The global result register."""
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
        Inserts a controlled Pauli :math:`X` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...

    def cz(self, control: Value, target: Value) -> None:
        """
        Inserts a controlled Pauli :math:`Z` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...

    def h(self, qubit: Value) -> None:
        """
        Inserts a Hadamard gate.

        :param qubit: The target qubit.
        """
        ...

    def mz(self, qubit: Value, result: Value) -> None:
        """
        Inserts a Z-basis measurement operation.

        :param qubit: The qubit to measure.
        :param result: A result where the measurement result will be written to.
        """
        ...

    def reset(self, qubit: Value) -> None:
        """
        Inserts a reset operation.

        :param qubit: The qubit to reset.
        """
        ...

    def rx(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`x` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def ry(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`y` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def rz(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`z` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...

    def s(self, qubit: Value) -> None:
        """
        Inserts an :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...

    def s_adj(self, qubit: Value) -> None:
        """
        Inserts an adjoint :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...

    def t(self, qubit: Value) -> None:
        """
        Inserts a :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...

    def t_adj(self, qubit: Value) -> None:
        """
        Inserts an adjoint :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...

    def x(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`X` gate.

        :param qubit: The target qubit.
        """
        ...

    def y(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`Y` gate.

        :param qubit: The target qubit.
        """
        ...

    def z(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`Z` gate.

        :param qubit: The target qubit.
        """
        ...

    def if_result(self, result: Value, one: Callable[[], None] = ..., zero: Callable[[], None] = ...) -> None:
        """
        Inserts a branch conditioned on a measurement result.

        Instructions inserted when ``one`` is called will be inserted into the
        one branch. Instructions inserted when ``zero`` is called will be
        inserted into the zero branch. The one and zero callables should use
        this module's builder to build instructions.

        :param cond: The result condition to branch on.
        :param one: A callable that inserts instructions for the branch where
                    the result is one.
        :param zero: A callable that inserts instructions for the branch where
                     the result is zero.
        """
        ...

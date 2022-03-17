# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""
Types from the LLVM type system, as well as additional type aliases and opaque
types defined by QIR.
"""

from dataclasses import dataclass
from enum import Enum, auto
from typing import Sequence, Union


class Void(Enum):
    """The void type."""
    _VOID = auto()


@dataclass
class Integer:
    """The integer type."""

    width: int
    """The number of bits used to represent the integer."""


class Double(Enum):
    """The 64-bit floating-point type."""
    _DOUBLE = auto()


class Qubit(Enum):
    """The QIR qubit type, represented by the LLVM type `%Qubit*`."""
    _QUBIT = auto()


class Result(Enum):
    """The QIR result type, represented by the LLVM type `%Result*`."""
    _RESULT = auto()


Value = Union[Integer, Double, Qubit, Result]
"""The set of types that can represent a value."""

Return = Union[Void, Value]
"""The set of types that can be used as the return type of a function."""


@dataclass
class Function:
    """The type of a function."""

    param_types: Sequence[Value]
    """The type of each parameter to the function."""

    return_type: Return
    """The return type of the function."""


VOID: Void = Void._VOID
"""The void type."""

BOOL: Integer = Integer(1)
"""The QIR `Bool` type, represented by the LLVM type `i1`."""

INT: Integer = Integer(64)
"""The QIR `Int` type, represented by the LLVM type `i64`."""

DOUBLE: Double = Double._DOUBLE
"""The 64-bit floating-point type."""

QUBIT: Qubit = Qubit._QUBIT
"""The QIR qubit type, represented by the LLVM type `%Qubit*`."""

RESULT: Result = Result._RESULT
"""The QIR result type, represented by the LLVM type `%Result*`."""

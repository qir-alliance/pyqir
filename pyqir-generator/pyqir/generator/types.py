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
class Int:
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


@dataclass
class Function:
    """The type of a function."""

    params: Sequence["Type"]
    """The type of each parameter to the function."""

    result: "Type"
    """The return type of the function."""


Type = Union[Void, Int, Double, Qubit, Result, Function]
"""A QIR type."""

VOID: Void = Void._VOID
"""The void type."""

BOOL: Int = Int(1)
"""The boolean type, represented by a 1-bit integer."""

DOUBLE: Double = Double._DOUBLE
"""The 64-bit floating-point type."""

QUBIT: Qubit = Qubit._QUBIT
"""The QIR qubit type, represented by the LLVM type `%Qubit*`."""

RESULT: Result = Result._RESULT
"""The QIR result type, represented by the LLVM type `%Result*`."""

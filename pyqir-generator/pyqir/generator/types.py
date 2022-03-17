from dataclasses import dataclass
from enum import Enum, auto
from typing import Sequence, Union


class Void(Enum):
    _VOID = auto()


@dataclass
class Integer:
    width: int


class Double(Enum):
    _DOUBLE = auto()


class Qubit(Enum):
    _QUBIT = auto()


class Result(Enum):
    _RESULT = auto()


Value = Union[Integer, Double, Qubit, Result]
Return = Union[Void, Value]


@dataclass
class Function:
    param_types: Sequence[Value]
    return_type: Return


VOID: Void = Void._VOID
BOOL: Integer = Integer(1)
INT: Integer = Integer(64)
DOUBLE: Double = Double._DOUBLE
QUBIT: Qubit = Qubit._QUBIT
RESULT: Result = Result._RESULT

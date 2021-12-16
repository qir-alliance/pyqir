# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from dataclasses import dataclass, field
from typing import List, Union

Instruction = Union[
    "Cx",
    "Cz",
    "H",
    "M",
    "Reset",
    "Rx",
    "Ry",
    "Rz",
    "S",
    "SAdj",
    "T",
    "TAdj",
    "X",
    "Y",
    "Z",
    "DumpMachine",
    "If",
]


@dataclass
class Cx:
    control: str
    target: str


@dataclass
class Cz:
    control: str
    target: str


@dataclass
class H:
    qubit: str


@dataclass
class M:
    qubit: str
    target: str


@dataclass
class Reset:
    qubit: str


@dataclass
class Rx:
    theta: float
    qubit: str


@dataclass
class Ry:
    theta: float
    qubit: str


@dataclass
class Rz:
    theta: float
    qubit: str


@dataclass
class S:
    qubit: str


@dataclass
class SAdj:
    qubit: str


@dataclass
class T:
    qubit: str


@dataclass
class TAdj:
    qubit: str


@dataclass
class X:
    qubit: str


@dataclass
class Y:
    qubit: str


@dataclass
class Z:
    qubit: str


@dataclass
class DumpMachine:
    pass


@dataclass
class If:
    condition: str
    true: List[Instruction] = field(default_factory=list)
    false: List[Instruction] = field(default_factory=list)

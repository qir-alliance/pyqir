# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum, auto


class IPredicate(Enum):
    EQ = auto()
    NE = auto()
    UGT = auto()
    UGE = auto()
    ULT = auto()
    ULE = auto()
    SGT = auto()
    SGE = auto()
    SLT = auto()
    SLE = auto()

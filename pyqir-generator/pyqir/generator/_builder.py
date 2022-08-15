# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum, auto


class IntPredicate(Enum):
    """An integer comparison predicate."""

    EQ = auto()
    """Equal."""

    NE = auto()
    """Not equal."""

    UGT = auto()
    """Unsigned greater than."""

    UGE = auto()
    """Unsigned greater or equal."""

    ULT = auto()
    """Unsigned less than."""

    ULE = auto()
    """Unsigned less or equal."""

    SGT = auto()
    """Signed greater than."""

    SGE = auto()
    """Signed greater or equal."""

    SLT = auto()
    """Signed less than."""

    SLE = auto()
    """Signed less or equal."""

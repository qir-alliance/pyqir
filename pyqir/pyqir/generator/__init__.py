# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""
Provides the ability to generate QIR using a Python API.

This package is intended to be used by code automating translation processes
enabling the conversion in some format to QIR via Python; i.e., this is a low
level API intended to be used as a bridge to existing Python frameworks enabling
the generation of QIR rather than directly consumed by an end-user. It is not
intended to be used as a framework for algorithm and application development.
"""

from enum import Enum, auto
from pyqir._native import (
    BasicQisBuilder as BasicQisBuilder,
    Builder as Builder,
    SimpleModule as SimpleModule,
    Type as Type,
    TypeFactory as TypeFactory,
    Value as Value,
    bitcode_to_ir as bitcode_to_ir,
    const as const,
    ir_to_bitcode as ir_to_bitcode,
)


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

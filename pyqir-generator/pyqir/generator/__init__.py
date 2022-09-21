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

from pyqir.generator._builder import IntPredicate as IntPredicate
from pyqir.generator._native import (
    BasicQisBuilder as BasicQisBuilder,
    Builder as Builder,
    Context as Context,
    SimpleModule as SimpleModule,
    Type as Type,
    Value as Value,
    bitcode_to_ir as bitcode_to_ir,
    const as const,
    ir_to_bitcode as ir_to_bitcode,
)

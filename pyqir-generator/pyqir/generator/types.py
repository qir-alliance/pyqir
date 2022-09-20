# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator._native import (
    ArrayType as Array,
    DoubleType as Double,
    FunctionType as Function,
    IntegerType as Integer,
    PointerType as Pointer,
    StructType as Struct,
    VoidType as Void,
)

__all__ = ["Array", "Double", "Function", "Integer", "Pointer", "Struct", "Void"]

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator._native import Qubit, ResultRef, Value
from typing import Union

Valuable = Union[Value, bool, int, float, Qubit, ResultRef]
"""
A QIR or LLVM value, or a Python value that can be automatically converted into
one.
"""

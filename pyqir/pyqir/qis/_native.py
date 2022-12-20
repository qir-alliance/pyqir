# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import Builder, Value

def barrier(builder: Builder) -> None:
    """
    Inserts a barrier instruction

    :param builder: The underlying builder used to build QIS instructions.
    :rtype: None
    """
    ...

def swap(builder: Builder, qubit1: Value, qubit2: Value) -> None:
    """
    Inserts a swap gate

    :param builder: The underlying builder used to build QIS instructions.
    :param Value qubit1: The first qubit to apply the gate to.
    :param Value qubit2: The second qubit to apply the gate to.
    :rtype: None
    """
    ...

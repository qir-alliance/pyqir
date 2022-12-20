# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import Builder, Value


def array_record_output(builder: Builder, num_elements: Value, label: Value) -> None:
    """
    Inserts a marker in the generated output that indicates the start
    of an array and how many array elements it has.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value num_elements: How many array elements the array has
    :param str label: A string label for the array. Depending on the output schema, the label is included in the output or omitted.
    """
    ...


def initialize(builder: Builder, reserved: Value) -> None:
    """
    Initializes the execution environment. Sets all qubits to a zero-state
    if they are not dynamically managed.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value reserved: Reserved. For base profile QIR, a const null i8* Value should be passed.
    """
    ...


def result_record_output(builder: Builder, result: Value, label: Value) -> None:
    """
    Adds a measurement result to the generated output.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value result: A result measurement to record
    :param str label: A string label for the result value. Depending on the output schema, the label is included in the output or omitted.
    """
    ...


def tuple_record_output(builder: Builder, num_elements: Value, label: Value) -> None:
    """
    Inserts a marker in the generated output that indicates the start
    of a tuple and how many tuple elements it has.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value num_elements: How many tuple elements the tuple has
    :param str label: A string label for the tuple. Depending on the output schema, the label is included in the output or omitted.
    """
    ...

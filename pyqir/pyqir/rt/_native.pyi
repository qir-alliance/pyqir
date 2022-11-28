# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import Builder, Value

def array_record_output(
    builder: Builder, num_elements: Value, label: Value
) -> None: ...
def initialize(reserved: Value) -> None: ...
def result_record_output(builder: Builder, result: Value, label: Value) -> None: ...
def tuple_record_output(
    builder: Builder, num_elements: Value, label: Value
) -> None: ...
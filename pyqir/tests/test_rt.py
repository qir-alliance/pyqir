# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import BasicQisBuilder, SimpleModule, TypeFactory, Value, const
import pytest
from typing import Callable, Union



def test_array_record_output_untagged() -> None:
    mod = SimpleModule("test_single", 1, 0)
    qis = BasicQisBuilder(mod.builder)
    qis.array_record_output(const(42), TODO)
    name = "array"
    call = f"call void @__quantum__rt__{name}(i64 42, i8* null)"
    assert call in mod.ir()


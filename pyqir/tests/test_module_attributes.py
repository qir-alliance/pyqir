# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from functools import partial
from typing import Callable, List, Tuple

import pytest

import pyqir
from pyqir import (
    IntType,
    ModuleFlagBehavior,
)


def test_getting_non_existing_metadata_returns_none() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    assert pyqir.get_flag(mod, "not found") is None


def test_value_metadata_can_added() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i32 = IntType(mod.context, 32)
    value = pyqir.const(i32, 42)
    pyqir.add_value_flag(mod, "expected", ModuleFlagBehavior.ERROR, value)
    ir = str(mod)
    print(ir)
    assert "!llvm.module.flags = !{!0}" in ir
    assert '!0 = !{i32 1, !"expected", i32 42}' in ir


def test_value_metadata_can_retrieved() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i32 = IntType(mod.context, 32)
    value = pyqir.const(i32, 42)
    pyqir.add_value_flag(mod, "expected", ModuleFlagBehavior.ERROR, value)
    flag = pyqir.get_flag(mod, "expected")
    assert flag is not None

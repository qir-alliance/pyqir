# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir
from pyqir import (
    IntType,
    ModuleFlagBehavior,
)


def test_getting_non_existing_metadata_returns_none() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    assert mod.get_flag("not found") is None


def test_value_metadata_can_added() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i32 = IntType(mod.context, 32)
    value = pyqir.const(i32, 42)
    mod.add_value_flag("expected", ModuleFlagBehavior.ERROR, value)
    ir = str(mod)
    print(ir)
    assert "!llvm.module.flags = !{!0}" in ir
    assert '!0 = !{i32 1, !"expected", i32 42}' in ir


def test_value_metadata_can_retrieved() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i32 = IntType(mod.context, 32)
    value = pyqir.const(i32, 42)
    mod.add_value_flag("expected", ModuleFlagBehavior.ERROR, value)
    flag = mod.get_flag("expected")
    assert flag is not None

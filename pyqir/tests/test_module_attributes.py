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
    mod.add_value_flag(ModuleFlagBehavior.ERROR, "expected", value)
    ir = str(mod)
    print(ir)
    assert "!llvm.module.flags = !{!0}" in ir
    assert '!0 = !{i32 1, !"expected", i32 42}' in ir


def test_i32_value_metadata_can_retrieved() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i32 = IntType(mod.context, 32)
    value = pyqir.const(i32, 42)
    mod.add_value_flag(ModuleFlagBehavior.ERROR, "expected", value)
    flag = mod.get_flag("expected")
    assert flag is not None
    assert str(flag) == "i32 42"


def test_bool_value_metadata_can_retrieved() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i1 = IntType(mod.context, 1)
    false_value = pyqir.const(i1, False)
    mod.add_value_flag(ModuleFlagBehavior.ERROR, "id_f", false_value)

    true_value = pyqir.const(i1, True)
    mod.add_value_flag(ModuleFlagBehavior.ERROR, "id_t", true_value)
    false_flag = mod.get_flag("id_f")
    assert false_flag is not None
    assert str(false_flag) == "i1 false"
    true_flag = mod.get_flag("id_t")
    assert true_flag is not None
    assert str(true_flag) == "i1 true"


def test_metadata_string_value_metadata_can_retrieved() -> None:
    context = pyqir.Context()
    source = "md string"
    id = "md_id"
    expected = f'!"{source}"'
    md = context.create_metadata_string(source)
    mod = pyqir.Module(context, "test")

    mod.add_metadata_flag(ModuleFlagBehavior.ERROR, id, md)
    flag = mod.get_flag(id)
    assert flag is not None
    assert expected == str(flag)

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir
from pyqir import (
    IntType,
    ModuleFlagBehavior,
)
import pytest


def test_getting_non_existing_metadata_returns_none() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    assert mod.get_flag("not found") is None


def test_value_metadata_can_added() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i32 = IntType(mod.context, 32)
    value = pyqir.const(i32, 42)
    mod.add_flag(ModuleFlagBehavior.ERROR, "expected", value)
    ir = str(mod)
    print(ir)
    assert "!llvm.module.flags = !{!0}" in ir
    assert '!0 = !{i32 1, !"expected", i32 42}' in ir


def test_i32_value_metadata_can_retrieved() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i32 = IntType(mod.context, 32)
    value = pyqir.const(i32, 42)
    mod.add_flag(ModuleFlagBehavior.ERROR, "expected", value)
    flag = mod.get_flag("expected")
    assert flag is not None
    assert isinstance(flag, pyqir.ConstantAsMetadata)
    assert isinstance(flag.value, pyqir.IntConstant)
    assert flag.value.value == 42
    assert str(flag) == "i32 42"


def test_bool_value_metadata_can_retrieved() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    i1 = IntType(mod.context, 1)
    false_value = pyqir.const(i1, False)
    mod.add_flag(ModuleFlagBehavior.ERROR, "id_f", false_value)

    true_value = pyqir.const(i1, True)
    mod.add_flag(ModuleFlagBehavior.ERROR, "id_t", true_value)
    false_flag = mod.get_flag("id_f")
    assert false_flag is not None
    assert isinstance(false_flag, pyqir.ConstantAsMetadata)
    assert str(false_flag) == "i1 false"
    assert isinstance(false_flag.value, pyqir.IntConstant)
    assert false_flag.value.value == False
    true_flag = mod.get_flag("id_t")
    assert true_flag is not None
    assert isinstance(true_flag, pyqir.ConstantAsMetadata)
    assert str(true_flag) == "i1 true"
    assert isinstance(true_flag.value, pyqir.IntConstant)
    assert true_flag.value.value == True


def test_metadata_string_value_metadata_can_retrieved() -> None:
    context = pyqir.Context()
    source = "md string"
    id = "md_id"
    expected = f'!"{source}"'
    mds = pyqir.MetadataString(context, source)
    mod = pyqir.Module(context, "test")

    mod.add_flag(ModuleFlagBehavior.ERROR, id, mds)
    flag = mod.get_flag(id)
    assert flag is not None
    assert isinstance(flag, pyqir.MetadataString)
    assert expected == str(flag)
    assert flag.value == source


def test_add_metadata_flag_raises_with_wrong_ownership() -> None:
    mod = pyqir.Module(pyqir.Context(), "")
    md = pyqir.MetadataString(pyqir.Context(), "value")
    with pytest.raises(ValueError):
        mod.add_flag(ModuleFlagBehavior.ERROR, "", md)


def test_add_value_flag_raises_with_wrong_ownership() -> None:
    i32 = IntType(pyqir.Context(), 32)
    value = pyqir.const(i32, 42)
    mod = pyqir.Module(pyqir.Context(), "")
    with pytest.raises(ValueError):
        mod.add_flag(ModuleFlagBehavior.ERROR, "", value)


def test_module_qir_major_version() -> None:
    mod = pyqir.Module(pyqir.Context(), "")
    assert mod.qir_major_version is None
    mod.qir_major_version = 42
    assert mod.qir_major_version == 42
    # value can only be set once
    with pytest.raises(ValueError):
        mod.qir_major_version = 7


def test_module_qir_minor_version() -> None:
    mod = pyqir.Module(pyqir.Context(), "")
    assert mod.qir_minor_version is None
    mod.qir_minor_version = 7
    assert mod.qir_minor_version == 7
    # value can only be set once
    with pytest.raises(ValueError):
        mod.qir_minor_version = 42


def test_module_dynamic_qubit_management() -> None:
    mod = pyqir.Module(pyqir.Context(), "")
    assert mod.dynamic_qubit_management is None
    mod.dynamic_qubit_management = True
    assert mod.dynamic_qubit_management == True
    # value can only be set once
    with pytest.raises(ValueError):
        mod.dynamic_qubit_management = False


def test_module_dynamic_result_management() -> None:
    mod = pyqir.Module(pyqir.Context(), "")
    assert mod.dynamic_result_management is None
    mod.dynamic_result_management = True
    assert mod.dynamic_result_management == True
    # value can only be set once
    with pytest.raises(ValueError):
        mod.dynamic_result_management = False

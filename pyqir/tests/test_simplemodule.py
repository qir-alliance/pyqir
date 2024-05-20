# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir
from pyqir import (
    required_num_qubits,
    required_num_results,
    is_entry_point,
)


def test_default_attributes_are_set() -> None:
    simple = pyqir.SimpleModule("test", 2, 5)
    mod = pyqir.Module.from_bitcode(pyqir.Context(), simple.bitcode())

    entry = next(filter(is_entry_point, mod.functions))
    assert required_num_qubits(entry) == 2
    assert required_num_results(entry) == 5

    assert mod.get_flag("qir_major_version") is not None
    assert str(mod.get_flag("qir_major_version")) == "i32 1"

    assert mod.get_flag("qir_minor_version") is not None
    assert str(mod.get_flag("qir_minor_version")) == "i32 0"

    assert mod.get_flag("dynamic_qubit_management") is not None
    assert str(mod.get_flag("dynamic_qubit_management")) == "i1 false"

    assert mod.get_flag("dynamic_result_management") is not None
    assert str(mod.get_flag("dynamic_result_management")) == "i1 false"


def test_entry_point_override_is_applied() -> None:
    simple = pyqir.SimpleModule("test", 2, 5, "new_entry")
    mod = pyqir.Module.from_bitcode(pyqir.Context(), simple.bitcode())

    entry = next(filter(is_entry_point, mod.functions))
    assert entry.name == "new_entry"

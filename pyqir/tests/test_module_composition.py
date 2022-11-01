# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import BasicQisBuilder, SimpleModule, Context, Module
import pytest

def bell(parent: Module) -> SimpleModule:
    module = SimpleModule(parent, 2, 2, "bell")
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])
    qis.mz(module.qubits[0], module.results[0])
    qis.mz(module.qubits[1], module.results[1])

    return module


def bell_no_measure(parent: Module) -> SimpleModule:
    module = SimpleModule(parent, 2, 0, "bell_no_measure")
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])

    return module


def test_module_creation_with_non_str_name_should_fail() -> None:
    with pytest.raises(TypeError):
        Module(Context(), 5)


def test_module_creation_with_name() -> None:
    expected = "module's name"
    module = Module(Context(), expected)
    assert expected == module.name


def test_module_creation_with_empty_name() -> None:
    expected = ""
    module = Module(Context(), expected)
    assert expected == module.name


def test_module_creation_with_none_name_should_fail() -> None:
    with pytest.raises(TypeError):
        Module(Context(), None)


def test_module_creation_with_name_can_change() -> None:
    initial = "source module's name"
    module = Module(Context(), initial)
    assert initial == module.name
    # make sure setter/getter work
    expected = "new name"
    module.name = expected
    assert expected == module.name
    # make sure set value flows through into IR
    ir = str(module)
    assert ir.startswith(f"; ModuleID = '{expected}'")


def test_simple_module_creation_with_name() -> None:
    expected = "module's name"
    module = SimpleModule(expected, 2, 2)
    ir = module.ir()
    assert ir.startswith(f"; ModuleID = '{expected}'")


def test_simple_module_creation_without_entry_point_name() -> None:
    module_name = "module's name"
    module = SimpleModule(module_name, 2, 2)
    expected = "main"
    assert expected == module.entry_point


def test_simple_module_creation_with_entry_point_name() -> None:
    module_name = "module's name"
    expected = "entry point"
    module = SimpleModule(module_name, 2, 2, expected)
    assert expected == module.entry_point


def test_simple_module_creation_with_parent_module() -> None:
    expected = "module's name"
    parent = Module(Context(), expected)
    module = SimpleModule(parent, 2, 2)
    ir = module.ir()
    assert ir.startswith(f"; ModuleID = '{expected}'")


def test_simple_module_creation_with_invalid_parent_or_name_should_fail() -> None:
    with pytest.raises(TypeError):
        SimpleModule(5, 2, 2)


def test_module_composition_with_conflicting_entry_points_uniques_them() -> None:
    expected = "module's name"
    parent = Module(Context(), expected)
    module0 = SimpleModule(parent, 2, 2, "entry")
    assert module0.entry_point == "entry"

    module1 = SimpleModule(parent, 2, 2, "entry")
    assert module1.entry_point != "entry"
    
    assert module1.entry_point.startswith("entry")

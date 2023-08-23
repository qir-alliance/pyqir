# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir
from pyqir import (
    IntType,
    ModuleFlagBehavior,
    Module,
    Context,
    add_string_attribute,
    Function,
    Linkage,
    FunctionType,
)
import pytest


def test_basic_key_value_attribute() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    add_string_attribute(function, "foo", "bar")
    assert function.attributes.func["foo"].string_value == "bar"
    ir = str(mod)
    assert 'attributes #0 = { "foo"="bar" }' in ir


def test_basic_key_only_attribute() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    add_string_attribute(function, "foo")
    assert function.attributes.func["foo"].string_value == ""
    ir = str(mod)
    assert 'attributes #0 = { "foo" }' in ir


def test_round_trip() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    add_string_attribute(function, "foo", "bar")
    # also test for non-value attributes
    add_string_attribute(function, "entry_point", "")
    # test behavior of empty attribute
    add_string_attribute(function, "", "")
    ir = str(mod)
    parsed_mod = Module.from_ir(Context(), ir, "test")
    assert str(parsed_mod) == str(mod)


def test_duplicate_attr_key_replaces_previous() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    add_string_attribute(function, "foo", "bar")
    add_string_attribute(function, "foo", "")
    ir = str(mod)
    # Tests that subsequently added attributes with the same key
    # replace previously added ones
    assert 'attributes #0 = { "foo" }' in ir


def test_attribute_alphabetical_sorting() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    add_string_attribute(function, "b", "A")
    add_string_attribute(function, "c", "")
    add_string_attribute(function, "a", "a")
    add_string_attribute(function, "1", "")
    add_string_attribute(function, "A", "123")
    ir = str(mod)
    parsed_mod = Module.from_ir(Context(), ir, "test")
    print(str(mod))
    assert str(parsed_mod) == str(mod)
    # Tests that attributes are sorted alphabetically by key,
    # irrespective of their value
    assert 'attributes #0 = { "1" "A"="123" "a"="a" "b"="A" "c" }' in ir

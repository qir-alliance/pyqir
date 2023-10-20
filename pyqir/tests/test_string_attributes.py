# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List
import pyqir
from pyqir import (
    Attribute,
    ATTR_FUNCTION_INDEX,
    ATTR_RETURN_INDEX,
    Builder,
    IntType,
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


def test_round_trip_serialize_parse() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    add_string_attribute(function, "foo", "bar")
    # also test for non-value attributes
    add_string_attribute(function, "entry_point")
    # test behavior of empty attribute
    add_string_attribute(function, "")
    ir = str(mod)
    parsed_mod = Module.from_ir(Context(), ir, "test")
    assert str(parsed_mod) == str(mod)


def test_duplicate_attr_key_replaces_previous() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    add_string_attribute(function, "foo", "bar")
    add_string_attribute(function, "foo")
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


def test_function_attributes_can_be_iterated_in_alphabetical_order() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    function = Function(FunctionType(void, []), Linkage.EXTERNAL, "test_function", mod)
    # add them out of order, they will be sorted automatically
    add_string_attribute(function, "required_num_results", "1")
    add_string_attribute(function, "entry_point")
    add_string_attribute(function, "required_num_qubits", "2")
    attrs: List[Attribute] = list(function.attributes.func)
    assert len(attrs) == 3
    # Tests that attributes are sorted alphabetically by indexing into the list
    assert attrs[0].string_kind == "entry_point"
    assert attrs[0].string_value == ""
    assert attrs[1].string_kind == "required_num_qubits"
    assert attrs[1].string_value == "2"
    assert attrs[2].string_kind == "required_num_results"
    assert attrs[2].string_value == "1"


def test_parameter_attrs() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    i8 = IntType(mod.context, 8)
    function = Function(
        FunctionType(void, [i8]), Linkage.EXTERNAL, "test_function", mod
    )
    # add them out of order, they will be sorted automatically
    add_string_attribute(function, "zeroext", "", 1)
    add_string_attribute(function, "mycustom", "myvalue", 1)

    # params have their own AttributeSet
    attrs = list(function.attributes.param(0))

    attr = attrs[0]
    assert attr.string_kind == "mycustom"
    assert attr.string_value == "myvalue"

    attr = attrs[1]
    assert attr.string_kind == "zeroext"
    assert attr.string_value == ""


def test_return_attrs_can_be_added_and_read() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    i8 = IntType(mod.context, 8)
    function = Function(
        FunctionType(void, [i8]), Linkage.EXTERNAL, "test_function", mod
    )
    builder = Builder(mod.context)
    builder.ret(None)

    add_string_attribute(function, "mycustom", "myvalue", ATTR_RETURN_INDEX)

    # params have their own AttributeSet
    attrs = list(function.attributes.ret)

    attr = attrs[0]
    assert attr.string_kind == "mycustom"
    assert attr.string_value == "myvalue"


def test_explicit_function_index_attrs_can_be_added_and_read() -> None:
    mod = pyqir.Module(pyqir.Context(), "test")
    void = pyqir.Type.void(mod.context)
    i8 = IntType(mod.context, 8)
    function = Function(
        FunctionType(void, [i8]), Linkage.EXTERNAL, "test_function", mod
    )
    builder = Builder(mod.context)
    builder.ret(None)

    add_string_attribute(function, "mycustom", "myvalue", ATTR_FUNCTION_INDEX)

    attrs = list(function.attributes.func)

    attr = attrs[0]
    assert attr.string_kind == "mycustom"
    assert attr.string_value == "myvalue"

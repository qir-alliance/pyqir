# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    BasicQisBuilder,
    Constant,
    IntType,
    PointerType,
    SimpleModule,
    Value,
    const,
    const_getelementptr,
)
import pytest
from typing import Callable, Union


def test_array_record_output_untagged() -> None:
    mod = SimpleModule("array_record_output", 0, 0)
    qis = BasicQisBuilder(mod.builder)
    i8p = PointerType(IntType(mod.context, 8))
    qis.array_record_output(const(IntType(mod.context, 64), 42), Constant.null(i8p))
    name = "array_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, i8* null)"
    assert call in mod.ir()


def test_array_record_output_tagged() -> None:
    mod = SimpleModule("array_record_output", 0, 0)
    qis = BasicQisBuilder(mod.builder)
    tag = mod.add_global_string(b"some tag")
    i32 = IntType(mod.context, 32)
    gep = const_getelementptr(tag, [const(i32, 0), const(i32, 0)])
    qis.array_record_output(const(IntType(mod.context, 64), 42), gep)
    name = "array_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, i8* getelementptr inbounds ([9 x i8], [9 x i8]* @0, i32 0, i32 0))"
    assert call in mod.ir()


def test_tuple_record_output_untagged() -> None:
    mod = SimpleModule("tuple_record_output", 0, 0)
    qis = BasicQisBuilder(mod.builder)
    i8p = PointerType(IntType(mod.context, 8))
    qis.tuple_record_output(const(IntType(mod.context, 64), 42), Constant.null(i8p))
    name = "tuple_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, i8* null)"
    assert call in mod.ir()


def test_tuple_record_output_tagged() -> None:
    mod = SimpleModule("tuple_record_output", 0, 0)
    qis = BasicQisBuilder(mod.builder)
    tag = mod.add_global_string(b"some tag")
    i32 = IntType(mod.context, 32)
    gep = const_getelementptr(tag, [const(i32, 0), const(i32, 0)])
    qis.tuple_record_output(const(IntType(mod.context, 64), 42), gep)
    name = "tuple_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, i8* getelementptr inbounds ([9 x i8], [9 x i8]* @0, i32 0, i32 0))"
    assert call in mod.ir()


def test_result_record_output_tagged() -> None:
    mod = SimpleModule("result_record_output", 0, 1)
    qis = BasicQisBuilder(mod.builder)
    i8p = PointerType(IntType(mod.context, 8))
    qis.result_record_output(mod.results[0], Constant.null(i8p))
    name = "result_record_output"
    call = f"call void @__quantum__rt__{name}(%Result* null, i8* null)"
    assert call in mod.ir()


def test_result_record_output_tagged() -> None:
    mod = SimpleModule("result_record_output", 0, 1)
    qis = BasicQisBuilder(mod.builder)
    tag = mod.add_global_string(b"some tag")
    i32 = IntType(mod.context, 32)
    gep = const_getelementptr(tag, [const(i32, 0), const(i32, 0)])
    qis.result_record_output(mod.results[0], gep)
    name = "result_record_output"
    call = f"call void @__quantum__rt__{name}(%Result* null, i8* getelementptr inbounds ([9 x i8], [9 x i8]* @0, i32 0, i32 0))"
    assert call in mod.ir()

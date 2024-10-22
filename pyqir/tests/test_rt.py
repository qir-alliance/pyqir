# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    Constant,
    IntType,
    PointerType,
    SimpleModule,
    const,
)
import pyqir.rt as rt


def test_array_record_output_untagged() -> None:
    mod = SimpleModule("array_record_output", 0, 0)
    i8p = PointerType(IntType(mod.context, 8))
    rt.array_record_output(
        mod.builder, const(IntType(mod.context, 64), 42), Constant.null(i8p)
    )
    name = "array_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, ptr null)"
    assert call in mod.ir()


def test_array_record_output_tagged() -> None:
    mod = SimpleModule("array_record_output", 0, 0)
    label = mod.add_byte_string(b"some tag")
    rt.array_record_output(mod.builder, const(IntType(mod.context, 64), 42), label)
    name = "array_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, ptr getelementptr inbounds ([9 x i8], [9 x i8]* @0, i32 0, i32 0))"
    assert call in mod.ir()


def test_initialize() -> None:
    mod = SimpleModule("initialize", 0, 0)
    i8p = PointerType(IntType(mod.context, 8))
    rt.initialize(mod.builder, Constant.null(i8p))
    name = "initialize"
    call = f"call void @__quantum__rt__{name}(ptr null)"
    assert call in mod.ir()


def test_tuple_record_output_untagged() -> None:
    mod = SimpleModule("tuple_record_output", 0, 0)
    i8p = PointerType(IntType(mod.context, 8))
    rt.tuple_record_output(
        mod.builder, const(IntType(mod.context, 64), 42), Constant.null(i8p)
    )
    name = "tuple_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, ptr null)"
    assert call in mod.ir()


def test_tuple_record_output_tagged() -> None:
    mod = SimpleModule("tuple_record_output", 0, 0)
    label = mod.add_byte_string(b"some tag")
    rt.tuple_record_output(mod.builder, const(IntType(mod.context, 64), 42), label)
    name = "tuple_record_output"
    call = f"call void @__quantum__rt__{name}(i64 42, ptr getelementptr inbounds ([9 x i8], [9 x i8]* @0, i32 0, i32 0))"
    assert call in mod.ir()


def test_result_record_output_untagged() -> None:
    mod = SimpleModule("result_record_output", 0, 1)
    i8p = PointerType(IntType(mod.context, 8))
    rt.result_record_output(mod.builder, mod.results[0], Constant.null(i8p))
    name = "result_record_output"
    call = f"call void @__quantum__rt__{name}(ptr null, ptr null)"
    assert call in mod.ir()


def test_result_record_output_tagged() -> None:
    mod = SimpleModule("result_record_output", 0, 1)
    label = mod.add_byte_string(b"some tag")
    rt.result_record_output(mod.builder, mod.results[0], label)
    name = "result_record_output"
    call = f"call void @__quantum__rt__{name}(ptr null, ptr getelementptr inbounds ([9 x i8], [9 x i8]* @0, i32 0, i32 0))"
    assert call in mod.ir()

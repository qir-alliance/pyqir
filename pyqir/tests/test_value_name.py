# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pathlib import Path

import pyqir


current_file_path = Path(__file__)
# Get the directory of the current file
current_dir = current_file_path.parent

from pyqir import (
    Context,
    is_entry_point,
)


def read_file(file_name: str) -> str:
    return Path(current_dir / file_name).read_text(encoding="utf-8")


def test_setting_function_name_changes_name() -> None:
    context = Context()
    ir = read_file("random_bit.ll")
    mod = pyqir.Module.from_ir(context, ir)
    entry_point = next(filter(is_entry_point, mod.functions))
    assert entry_point.name == "random_bit"
    expected = "new_name"
    entry_point.name = expected
    entry_point = next(filter(is_entry_point, mod.functions))
    assert entry_point.name == expected


def test_setting_constant_name_does_not_do_anything() -> None:
    context = pyqir.Context()
    const0 = pyqir.const(pyqir.IntType(context, 64), 42)
    const0.name = "my_value"
    assert const0.name == ""


def test_setting_block_name_changes_name() -> None:
    mod = pyqir.SimpleModule("test_type_mismatch", 0, 0)
    mod.entry_block.name = "my_block"
    ir = mod.ir()
    assert "my_block:" in ir


def test_int_variable() -> None:
    mod = pyqir.SimpleModule("test", 0, 0)
    i64 = pyqir.IntType(mod.context, 64)

    source = mod.add_external_function("source", pyqir.FunctionType(i64, []))
    sink = mod.add_external_function("sink", pyqir.FunctionType(i64, [i64]))

    source_res = mod.builder.call(source, [])
    source_res.name = "my_var"

    sink_res = mod.builder.call(sink, [source_res])
    sink_res.name = "my_res"
    ir = mod.ir()
    assert "%my_var = call i64 @source()" in ir
    assert "%my_res = call i64 @sink(i64 %my_var)" in ir


def test_function_name_can_contain_spaces_and_chars() -> None:
    simple_mod = pyqir.SimpleModule("test", 0, 0)
    expected = "Some - ; name fin"
    simple_mod.entry_point.name = expected

    # verify the name is use and wrapped in quotes
    ir = simple_mod.ir()
    assert f'@"{expected}"() #0' in ir

    # verify we can find it by name without having to use quotes
    func = next(filter(lambda f: f.name == expected, simple_mod._module.functions))
    assert func == simple_mod.entry_point

    # Double check that the module is valid with this kind of name
    mod = pyqir.Module.from_ir(Context(), ir)
    assert mod.verify() is None

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pathlib import Path

import pytest

current_file_path = Path(__file__)
# Get the directory of the current file
current_dir = current_file_path.parent

from pyqir import (
    Context,
    Module,
)


def read_file(file_name: str) -> str:
    return Path(current_dir / file_name).read_text(encoding="utf-8")


def test_link_modules_with_same_context() -> None:
    context = Context()
    ir = read_file("random_bit.ll")
    dest = Module.from_ir(context, ir)
    ir = read_file("5_bit_random_number.ll")
    src = Module.from_ir(context, ir)
    dest.link(src)
    assert dest.verify() is None
    actual_ir = str(dest)
    expected_ir = str(read_file("combined_module.ll"))
    assert actual_ir == expected_ir


def test_link_modules_with_different_contexts() -> None:
    ir = read_file("random_bit.ll")
    dest = Module.from_ir(Context(), ir)
    ir = read_file("5_bit_random_number.ll")
    src = Module.from_ir(Context(), ir)
    with pytest.raises(ValueError) as ex:
        dest.link(src)
    assert (
        str(ex.value)
        == "Cannot link modules from different contexts. Modules are untouched."
    )


def test_link_module_with_src_minor_version_less() -> None:
    context = Context()
    ir = read_file("profile_v1.0_compat.ll")
    dest = Module.from_ir(context, ir)
    ir = read_file("profile_v1.1_compat.ll")
    src = Module.from_ir(context, ir)
    dest.link(src)
    assert dest.get_flag("qir_minor_version").value.value == 1


def test_link_module_with_src_minor_version_greater() -> None:
    context = Context()
    ir = read_file("profile_v1.1_compat.ll")
    dest = Module.from_ir(context, ir)
    ir = read_file("profile_v1.0_compat.ll")
    src = Module.from_ir(context, ir)
    dest.link(src)
    assert dest.get_flag("qir_minor_version").value.value == 1


def test_link_module_with_src_major_version_less() -> None:
    context = Context()
    ir = read_file("profile_v2.0_compat.ll")
    dest = Module.from_ir(context, ir)
    ir = read_file("profile_v1.0_compat.ll")
    src = Module.from_ir(context, ir)
    with pytest.raises(ValueError) as ex:
        dest.link(src)

    assert (
        "linking module flags 'qir_major_version': IDs have conflicting values"
        in str(ex)
    )


def test_link_module_with_src_major_version_greater() -> None:
    context = Context()
    ir = read_file("profile_v1.0_compat.ll")
    dest = Module.from_ir(context, ir)
    ir = read_file("profile_v2.0_compat.ll")
    src = Module.from_ir(context, ir)
    with pytest.raises(ValueError) as ex:
        dest.link(src)

    assert (
        "linking module flags 'qir_major_version': IDs have conflicting values"
        in str(ex)
    )

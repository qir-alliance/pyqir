# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pathlib import Path

from pyqir import (
    Context,
    Module,
)


def test_link_modules_with_same_context() -> None:
    context = Context()
    ir = Path("tests/random_bit.ll").read_text()
    dest = Module.from_ir(context, ir)
    ir = Path("tests/5_bit_random_number.ll").read_text()
    src = Module.from_ir(context, ir)
    assert dest.link(src) is None
    assert dest.verify() is None
    actual_ir = str(dest)
    expected_ir = str(Path("tests/combined_module.ll").read_text())
    assert actual_ir == expected_ir


def test_link_modules_with_different_contexts() -> None:
    ir = Path("tests/random_bit.ll").read_text()
    dest = Module.from_ir(Context(), ir)
    ir = Path("tests/5_bit_random_number.ll").read_text()
    src = Module.from_ir(Context(), ir)
    message = dest.link(src)
    assert (
        message == "Cannot link modules from different contexts. Modules are untouched."
    )

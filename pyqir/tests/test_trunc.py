# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os

import pytest
from pathlib import Path

import pyqir


def test_trunc() -> None:
    module = pyqir.SimpleModule("trunc", 1, 1)
    context = module.context
    builder = module.builder
    entry = module.entry_block
    builder.insert_at_end(entry)
    i16 = pyqir.IntType(context, 16)
    i32 = pyqir.IntType(context, 32)
    random_int = module.add_external_function(
        "random_int",
        pyqir.FunctionType(
            i32,
            [i32],
        ),
    )
    const = builder.call(random_int, [pyqir.const(i32, 0)])
    builder.trunc(const, i16)
    ir = module.ir()

    file = os.path.join(os.path.dirname(__file__), "resources/test_trunc.ll")
    expected = Path(file).read_text()
    assert ir == expected

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os

import pytest
from pathlib import Path

import pyqir


def test_zext() -> None:
    module = pyqir.SimpleModule("zext", 1, 1)
    context = module.context
    builder = module.builder
    entry = module.entry_block
    builder.insert_at_end(entry)
    i16 = pyqir.IntType(context, 16)
    i32 = pyqir.IntType(context, 32)
    random_int = module.add_external_function(
            "random_int",
            pyqir.FunctionType(
                i16,
                [i16],
            ),
        )
    const1 = builder.call(
                random_int, [pyqir.const(i16, 0)]
            )
    builder.zext(const1, i32)
    ir = module.ir()

    file = os.path.join(os.path.dirname(__file__), "resources/test_zext.ll")
    expected = Path(file).read_text()
    assert ir == expected

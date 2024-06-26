# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os

import pytest
from pathlib import Path

import pyqir


def test_constants() -> None:
    module = pyqir.SimpleModule("phi_constants", 1, 1)
    context = module.context
    builder = module.builder
    entry_point = module.entry_point

    entry = module.entry_block
    body = pyqir.BasicBlock(context, "body", entry_point)
    footer = pyqir.BasicBlock(context, "footer", entry_point)

    builder.insert_at_end(entry)
    const0 = pyqir.Constant.null(pyqir.IntType(context, 1))
    builder.condbr(const0, body, footer)

    builder.insert_at_end(body)
    builder.br(footer)

    builder.insert_at_end(footer)
    i32 = pyqir.IntType(context, 32)
    phi = builder.phi(i32)
    const_taken = pyqir.const(i32, 4)
    const_not_taken = pyqir.const(i32, 100)
    phi.add_incoming(const_taken, body)
    phi.add_incoming(const_not_taken, entry)

    ir = module.ir()

    file = os.path.join(os.path.dirname(__file__), "resources/test_phi_constants.ll")
    expected = Path(file).read_text()
    assert ir == expected


def test_add() -> None:
    module = pyqir.SimpleModule("phi_add", 1, 1)
    context = module.context
    builder = module.builder
    entry_point = module.entry_point

    entry = module.entry_block
    body = pyqir.BasicBlock(context, "body", entry_point)
    footer = pyqir.BasicBlock(context, "footer", entry_point)

    builder.insert_at_end(entry)
    i32 = pyqir.IntType(context, 32)
    const1 = pyqir.const(i32, 1)
    const2 = pyqir.const(i32, 2)
    sum_two = builder.add(const1, const1)
    cmp = builder.icmp(pyqir.IntPredicate.EQ, sum_two, const2)
    builder.condbr(cmp, body, footer)

    builder.insert_at_end(body)
    sum_three = builder.add(sum_two, const1)
    builder.br(footer)

    builder.insert_at_end(footer)
    phi = builder.phi(i32)
    phi.add_incoming(sum_two, entry)
    phi.add_incoming(sum_three, body)
    sum_four = builder.add(phi, const1)

    ir = module.ir()

    file = os.path.join(os.path.dirname(__file__), "resources/test_phi_add.ll")
    expected = Path(file).read_text()
    assert ir == expected

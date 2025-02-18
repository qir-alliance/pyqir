# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os

import pytest
from pathlib import Path

import pyqir


def test_reordering_instrs() -> None:
    module = pyqir.SimpleModule("reordering", 1, 1)
    builder = module.builder

    basic_builder = pyqir.BasicQisBuilder(builder)
    basic_builder.x(module.qubits[0])
    basic_builder.y(module.qubits[0])
    basic_builder.z(module.qubits[0])
    before_ir = module.ir()
    file = os.path.join(
        os.path.dirname(__file__), "resources/test_reordering_before.ll"
    )
    expected = Path(file).read_text()
    assert before_ir == expected

    # Reorder to be z, y, x
    x_instr = module.entry_block.instructions[0]
    y_instr = module.entry_block.instructions[1]
    z_instr = module.entry_block.instructions[2]
    builder.insert_before(x_instr)
    y_instr.remove()
    builder.instr(y_instr)
    builder.insert_before(y_instr)
    z_instr.remove()
    builder.instr(z_instr)
    builder.insert_at_end(module.entry_block)
    after_ir = module.ir()
    file = os.path.join(os.path.dirname(__file__), "resources/test_reordering_after.ll")
    expected = Path(file).read_text()
    assert after_ir == expected


def test_function_delete() -> None:
    module = pyqir.SimpleModule("delete functions", 1, 1)
    extfunc1 = module.add_external_function(
        "extfunc1",
        pyqir.FunctionType(
            pyqir.Type.void(module.context),
            [],
        ),
    )
    extfunc2 = module.add_external_function(
        "extfunc2",
        pyqir.FunctionType(
            pyqir.Type.void(module.context),
            [],
        ),
    )
    before_ir = module.ir()
    file = os.path.join(
        os.path.dirname(__file__), "resources/test_delete_func_before.ll"
    )
    expected = Path(file).read_text()
    assert before_ir == expected

    # Delete the first external function
    extfunc1.delete()
    after_ir = module.ir()
    file = os.path.join(
        os.path.dirname(__file__), "resources/test_delete_func_after.ll"
    )
    expected = Path(file).read_text()
    assert after_ir == expected

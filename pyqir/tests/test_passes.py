# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os

import pytest
from pathlib import Path

import pyqir

current_file_path = Path(__file__)
# Get the directory of the current file
current_dir = current_file_path.parent


def read_file(file_name: str) -> str:
    return Path(current_dir / file_name).read_text(encoding="utf-8")


def test_elimination_pass():
    ir_before = read_file("resources/test_passes_before.ll")
    module = pyqir.Module.from_ir(pyqir.Context(), ir_before)

    # Create a pass by inheriting from QIRPass
    class RemoveH(pyqir.QIRPass):
        def _on_qis_h(self, call, target):
            call.erase()

    RemoveH().run(module)
    ir_transformed = str(module)
    ir_after = read_file("resources/test_passes_remove_after.ll")
    assert ir_transformed == ir_after


def test_duplicate_gate_with_builder():
    ir_before = read_file("resources/test_passes_before.ll")
    module = pyqir.Module.from_ir(pyqir.Context(), ir_before)

    # Create a pass by inheriting from QIRPass
    class DuplicateH(pyqir.QIRPass):
        def _on_qis_h(self, call, target):
            self._builder.insert_before(call)
            self._builder.call(call.callee, call.args)

    DuplicateH().run(module)
    ir_transformed = str(module)
    ir_after = read_file("resources/test_passes_duplicate_after.ll")
    assert ir_transformed == ir_after


def test_reorder_gates_with_builder():
    ir_before = read_file("resources/test_passes_before.ll")
    module = pyqir.Module.from_ir(pyqir.Context(), ir_before)

    # Create a pass by inheriting from QIRPass
    class ReverseOrder(pyqir.QIRPass):
        def _on_block(self, block):
            self.gates = []
            super()._on_block(block)
            self._builder.insert_before(block.instructions[0])
            for gate in reversed(self.gates):
                self._builder.instr(gate)

        def _on_qis_h(self, call, target):
            self.gates.append(call)
            call.remove()

        def _on_qis_m(self, call, target, result):
            self.gates.append(call)
            call.remove()

    ReverseOrder().run(module)
    ir_transformed = str(module)
    ir_after = read_file("resources/test_passes_reverse_after.ll")
    assert ir_transformed == ir_after

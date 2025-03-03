# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List
from pathlib import Path

import pyqir

current_file_path = Path(__file__)
# Get the directory of the current file
current_dir = current_file_path.parent


def read_file(file_name: str) -> str:
    return Path(current_dir / file_name).read_text(encoding="utf-8")


def test_elimination_pass() -> None:
    ir_before = read_file("resources/test_passes_before.ll")
    module = pyqir.Module.from_ir(pyqir.Context(), ir_before)

    # Create a pass by inheriting from QirModuleVisitor
    class RemoveH(pyqir.QirModuleVisitor):
        def _on_qis_h(self, call: pyqir.Call, target: pyqir.Value) -> None:
            call.erase()

    RemoveH().run(module)
    ir_transformed = str(module)
    ir_after = read_file("resources/test_passes_remove_after.ll")
    assert ir_transformed == ir_after


def test_duplicate_gate_with_builder() -> None:
    ir_before = read_file("resources/test_passes_before.ll")
    module = pyqir.Module.from_ir(pyqir.Context(), ir_before)

    # Create a pass by inheriting from QirModuleVisitor
    class DuplicateH(pyqir.QirModuleVisitor):
        def _on_qis_h(self, call: pyqir.Call, target: pyqir.Value) -> None:
            self.builder.insert_before(call)
            self.builder.call(call.callee, call.args)

    DuplicateH().run(module)
    ir_transformed = str(module)
    ir_after = read_file("resources/test_passes_duplicate_after.ll")
    assert ir_transformed == ir_after


def test_reorder_gates_with_builder() -> None:
    ir_before = read_file("resources/test_passes_before.ll")
    module = pyqir.Module.from_ir(pyqir.Context(), ir_before)

    # Create a pass by inheriting from QirModuleVisitor
    class ReverseOrder(pyqir.QirModuleVisitor):
        def _on_block(self, block: pyqir.BasicBlock) -> None:
            self.gates: List[pyqir.Call] = []
            super()._on_block(block)
            self.builder.insert_before(block.instructions[0])
            for gate in reversed(self.gates):
                self.builder.instr(gate)

        def _on_qis_h(self, call: pyqir.Call, target: pyqir.Value) -> None:
            self.gates.append(call)
            call.remove()

        def _on_qis_m(
            self, call: pyqir.Call, target: pyqir.Value, result: pyqir.Value
        ) -> None:
            self.gates.append(call)
            call.remove()

    ReverseOrder().run(module)
    ir_transformed = str(module)
    ir_after = read_file("resources/test_passes_reverse_after.ll")
    assert ir_transformed == ir_after

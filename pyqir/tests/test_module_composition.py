# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    BasicQisBuilder,
    Builder,
    Context,
    Function,
    Module,
    create_entry_point,
    qubit,
    result,
)


def bell(module: Module) -> Function:
    entry = create_entry_point(module, "bell", 2, 2)
    builder = Builder(module)
    builder.set_insert_point(entry.basic_blocks[0])
    qis = BasicQisBuilder(builder)
    qis.h(qubit(builder, 0))
    qis.cx(qubit(builder, 0), qubit(builder, 1))
    qis.mz(qubit(builder, 0), result(builder, 0))
    qis.mz(qubit(builder, 1), result(builder, 1))
    return entry


def bell_no_measure(module: Module) -> Function:
    entry = create_entry_point(module, "bell_no_measure", 2, 0)
    builder = Builder(module)
    builder.set_insert_point(entry.basic_blocks[0])
    qis = BasicQisBuilder(builder)
    qis.h(qubit(builder, 0))
    qis.cx(qubit(builder, 0), qubit(builder, 1))
    return entry


def test_multiple_entry_points() -> None:
    module = Module(Context(), "bells")
    entries = [bell(module), bell_no_measure(module)]
    print(module)
    for i, entry in enumerate(entries):
        print(f"{i + 1}. {entry.name}")

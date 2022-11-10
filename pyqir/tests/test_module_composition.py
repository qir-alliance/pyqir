# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    BasicQisBuilder,
    Builder,
    Context,
    Function,
    FunctionType,
    Linkage,
    Module,
    Type,
    create_entry_point,
    qubit,
    result,
)


def bell(module: Module) -> Function:
    entry = create_entry_point(module, "bell", 2, 2)
    context = module.context
    builder = Builder(context)
    builder.set_insert_point(entry.basic_blocks[0])
    qis = BasicQisBuilder(builder)
    qis.h(qubit(context, 0))
    qis.cx(qubit(context, 0), qubit(context, 1))
    qis.mz(qubit(context, 0), result(context, 0))
    qis.mz(qubit(context, 1), result(context, 1))
    return entry


def bell_no_measure(module: Module) -> Function:
    entry = create_entry_point(module, "bell_no_measure", 2, 0)
    context = module.context
    builder = Builder(context)
    builder.set_insert_point(entry.basic_blocks[0])
    qis = BasicQisBuilder(builder)
    qis.h(qubit(context, 0))
    qis.cx(qubit(context, 0), qubit(context, 1))
    return entry


def using_external(module: Module) -> Function:
    entry = create_entry_point(module, "using_external", 1, 0)
    context = module.context
    builder = Builder(context)
    builder.set_insert_point(entry.basic_blocks[0])
    barrier = Function(
        FunctionType(Type.void(context), []),
        Linkage.EXTERNAL,
        "__quantum__qis__barrier__body",
        module,
    )
    builder.call(barrier, [])
    return entry


def test_multiple_entry_points() -> None:
    module = Module(Context(), "multiple")
    entries = [bell(module), bell_no_measure(module), using_external(module)]
    print(module)
    for i, entry in enumerate(entries):
        print(f"{i + 1}. {entry.name}")

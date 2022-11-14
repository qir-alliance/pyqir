# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    BasicBlock,
    BasicQisBuilder,
    Builder,
    Context,
    Function,
    FunctionType,
    Linkage,
    Module,
    Type,
    entry_point,
    is_entry_point,
    qubit,
    result,
)


def bell(module: Module) -> Function:
    entry = entry_point(module, "bell", 2, 2)
    context = module.context
    builder = Builder(context)
    builder.insert_from_end(BasicBlock(context, "", entry))
    qis = BasicQisBuilder(builder)
    qis.h(qubit(context, 0))
    qis.cx(qubit(context, 0), qubit(context, 1))
    qis.mz(qubit(context, 0), result(context, 0))
    qis.mz(qubit(context, 1), result(context, 1))
    return entry


def external(module: Module) -> Function:
    context = module.context
    barrier = Function(
        FunctionType(Type.void(context), []),
        Linkage.EXTERNAL,
        "__quantum__qis__barrier__body",
        module,
    )

    entry = entry_point(module, "barrier", 0, 0)
    builder = Builder(context)
    builder.insert_from_end(BasicBlock(context, "", entry))
    builder.call(barrier, [])
    return entry


def test_multiple_entry_points() -> None:
    module = Module(Context(), "test")
    bell_function = bell(module)
    external_function = external(module)
    entry_points = list(filter(is_entry_point, module.functions))
    names = list(map(lambda f: f.name, entry_points))
    assert len(entry_points) == 2
    assert bell_function.name in names
    assert external_function.name in names

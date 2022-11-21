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
    verify_module,
)


def bell(module: Module) -> Function:
    entry = entry_point(module, "bell", 2, 2)
    context = module.context
    builder = Builder(context)
    builder.insert_at_end(BasicBlock(context, "", entry))
    qis = BasicQisBuilder(builder)
    qis.h(qubit(context, 0))
    qis.cx(qubit(context, 0), qubit(context, 1))
    qis.mz(qubit(context, 0), result(context, 0))
    qis.mz(qubit(context, 1), result(context, 1))
    builder.ret(None)
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
    builder.insert_at_end(BasicBlock(context, "", entry))
    builder.call(barrier, [])
    builder.ret(None)
    return entry


def test_multiple() -> None:
    module = Module(Context(), "test")
    bell_function = bell(module)
    external_function = external(module)
    assert verify_module(module) is None

    names = list(map(lambda f: f.name, filter(is_entry_point, module.functions)))
    assert len(names) == 2
    assert bell_function.name in names
    assert external_function.name in names


def test_duplicate_name() -> None:
    module = Module(Context(), "test")
    bell1 = bell(module)
    bell2 = bell(module)
    assert verify_module(module) is None
    assert bell1.name != bell2.name

    names = list(map(lambda f: f.name, filter(is_entry_point, module.functions)))
    assert len(names) == 2
    assert bell1.name in names
    assert bell2.name in names


def test_invalid_module() -> None:
    context = Context()
    module = Module(context, "test")
    main = entry_point(module, "main", 0, 0)
    BasicBlock(context, "", main)
    assert (
        verify_module(module)
        == "Basic Block in function 'main' does not have terminator!\nlabel %0\n"
    )


def test_multiple_blocks() -> None:
    context = Context()
    module = Module(context, "test")
    main = entry_point(module, "main", 0, 0)

    builder = Builder(context)
    builder.insert_at_end(BasicBlock(context, "entry", main))
    exit = BasicBlock(context, "exit", main)
    builder.br(exit)
    builder.insert_at_end(exit)
    builder.ret(None)

    assert verify_module(module) is None
    assert "br label %exit" in str(module)

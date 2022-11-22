# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest

import pyqir
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
)


def _bell(module: Module) -> Function:
    entry = pyqir.entry_point(module, "bell", 2, 2)
    context = module.context
    builder = Builder(context)
    builder.insert_at_end(BasicBlock(context, "", entry))
    qis = BasicQisBuilder(builder)
    qis.h(pyqir.qubit(context, 0))
    qis.cx(pyqir.qubit(context, 0), pyqir.qubit(context, 1))
    qis.mz(pyqir.qubit(context, 0), pyqir.result(context, 0))
    qis.mz(pyqir.qubit(context, 1), pyqir.result(context, 1))
    builder.ret(None)
    return entry


def _external(module: Module) -> Function:
    context = module.context
    barrier = Function(
        FunctionType(Type.void(context), []),
        Linkage.EXTERNAL,
        "__quantum__qis__barrier__body",
        module,
    )

    entry = pyqir.entry_point(module, "barrier", 0, 0)
    builder = Builder(context)
    builder.insert_at_end(BasicBlock(context, "", entry))
    builder.call(barrier, [])
    builder.ret(None)
    return entry


def test_multiple() -> None:
    module = Module(Context(), "test")
    bell_function = _bell(module)
    external_function = _external(module)
    assert pyqir.verify_module(module) is None

    names = list(map(lambda f: f.name, filter(pyqir.is_entry_point, module.functions)))
    assert len(names) == 2
    assert bell_function.name in names
    assert external_function.name in names


def test_duplicate_name() -> None:
    module = Module(Context(), "test")
    bell1 = _bell(module)
    bell2 = _bell(module)
    assert pyqir.verify_module(module) is None
    assert bell1.name != bell2.name

    names = list(map(lambda f: f.name, filter(pyqir.is_entry_point, module.functions)))
    assert len(names) == 2
    assert bell1.name in names
    assert bell2.name in names


def test_invalid_module() -> None:
    context = Context()
    module = Module(context, "test")
    main = pyqir.entry_point(module, "main", 0, 0)
    BasicBlock(context, "", main)
    assert (
        pyqir.verify_module(module)
        == "Basic Block in function 'main' does not have terminator!\nlabel %0\n"
    )


def test_append_blocks() -> None:
    context = Context()
    builder = Builder(context)
    module = Module(context, "test")
    main = pyqir.entry_point(module, "main", 0, 0)

    builder.insert_at_end(BasicBlock(context, "entry", main))
    exit = BasicBlock(context, "exit", main)
    builder.br(exit)
    builder.insert_at_end(exit)
    builder.ret(None)

    assert pyqir.verify_module(module) is None
    assert list(map(lambda b: b.name, main.basic_blocks)) == ["entry", "exit"]


@pytest.mark.parametrize("with_parent", [False, True])
def test_prepend_block(with_parent: bool) -> None:
    context = Context()
    builder = Builder(context)
    module = Module(context, "test")
    main = pyqir.entry_point(module, "main", 0, 0)

    exit = BasicBlock(context, "exit", main)
    builder.insert_at_end(exit)
    builder.ret(None)
    builder.insert_at_end(
        BasicBlock(context, "entry", parent=main if with_parent else None, before=exit)
    )
    builder.br(exit)

    assert pyqir.verify_module(module) is None
    assert list(map(lambda b: b.name, main.basic_blocks)) == ["entry", "exit"]


def test_prepend_block_with_invalid_parent() -> None:
    context = Context()
    module = Module(context, "test")
    foo = pyqir.entry_point(module, "foo", 0, 0)
    bar = pyqir.entry_point(module, "bar", 0, 0)

    block = BasicBlock(context, "", foo)
    with pytest.raises(
        ValueError, match=r"^Insert before block isn't in parent function\.$"
    ):
        BasicBlock(context, "entry", parent=bar, before=block)


def test_orphan_block() -> None:
    with pytest.raises(ValueError, match=r"^Can't create block without parent\.$"):
        BasicBlock(Context(), "")

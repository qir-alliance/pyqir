# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from typing import Callable

import pytest
from pathlib import Path

import iqm_pyqir
from iqm_pyqir import (
    BasicQisBuilder,
    Context,
    Function,
    FunctionType,
    IntType,
    SimpleModule,
)


def define_read_result(context: Context, module: SimpleModule) -> Function:
    read_result = module.add_external_function(
        "__quantum__qis__read_result__body",
        FunctionType(IntType(context, 1), [iqm_pyqir.result_type(context)]),
    )
    return read_result


def test_empty_blocks() -> None:
    context = Context()
    module = SimpleModule("test_if", 0, 1, context=context)
    read_result = define_read_result(context, module)
    cond = module.builder.call(read_result, [module.results[0]])
    module.builder.if_(cond)
    ir = module.ir()

    context = Context()
    module = SimpleModule("test_if", 0, 1, context=context)
    qis = BasicQisBuilder(module.builder)
    qis.if_result(module.results[0])

    ir2 = module.ir()
    assert ir == ir2

    file = os.path.join(os.path.dirname(__file__), "resources/test_if_empty_blocks.ll")
    expected = Path(file).read_text()
    assert ir == expected


Action = Callable[[BasicQisBuilder, SimpleModule], Callable[[], None]]


def test_empty_false_block() -> None:
    block: Action = lambda qis, module: (lambda: qis.x(module.qubits[0]))

    context = Context()
    module = SimpleModule("test_if", 1, 1, context=context)
    read_result = define_read_result(context, module)
    cond = module.builder.call(read_result, [module.results[0]])
    qis = BasicQisBuilder(module.builder)
    module.builder.if_(cond, true=block(qis, module))
    ir = module.ir()

    context = Context()
    module = SimpleModule("test_if", 1, 1, context=context)
    qis = BasicQisBuilder(module.builder)
    qis.if_result(module.results[0], one=block(qis, module))

    ir2 = module.ir()
    assert ir == ir2

    file = os.path.join(
        os.path.dirname(__file__), "resources/test_empty_false_block.ll"
    )
    expected = Path(file).read_text()
    assert ir == expected


def test_empty_true_block() -> None:
    block: Action = lambda qis, module: (lambda: qis.x(module.qubits[0]))

    context = Context()
    module = SimpleModule("test_if", 1, 1, context=context)
    read_result = define_read_result(context, module)
    cond = module.builder.call(read_result, [module.results[0]])
    qis = BasicQisBuilder(module.builder)
    module.builder.if_(cond, false=block(qis, module))
    ir = module.ir()

    context = Context()
    module = SimpleModule("test_if", 1, 1, context=context)
    qis = BasicQisBuilder(module.builder)
    qis.if_result(module.results[0], zero=block(qis, module))

    ir2 = module.ir()
    assert ir == ir2

    file = os.path.join(os.path.dirname(__file__), "resources/test_empty_true_block.ll")
    expected = Path(file).read_text()
    assert ir == expected


def test_nested_blocks() -> None:
    xblock: Action = lambda qis, module: (lambda: qis.x(module.qubits[0]))
    yblock: Action = lambda qis, module: (lambda: qis.y(module.qubits[0]))
    zblock: Action = lambda qis, module: (lambda: qis.z(module.qubits[0]))
    tblock: Action = lambda qis, module: (lambda: qis.t(module.qubits[0]))

    context = Context()
    module = SimpleModule("test_if", 1, 3, context=context)
    read_result = define_read_result(context, module)
    qis = BasicQisBuilder(module.builder)
    module.builder.if_(
        module.builder.call(read_result, [module.results[0]]),
        true=lambda: module.builder.if_(
            module.builder.call(read_result, [module.results[1]]),
            xblock(qis, module),
            yblock(qis, module),
        ),
        false=lambda: module.builder.if_(
            module.builder.call(read_result, [module.results[2]]),
            zblock(qis, module),
            tblock(qis, module),
        ),
    )
    ir = module.ir()

    context = Context()
    module = SimpleModule("test_if", 1, 3, context=context)
    qis = BasicQisBuilder(module.builder)
    qis.if_result(
        module.results[0],
        one=lambda: qis.if_result(
            module.results[1], one=xblock(qis, module), zero=yblock(qis, module)
        ),
        zero=lambda: qis.if_result(
            module.results[2], one=zblock(qis, module), zero=tblock(qis, module)
        ),
    )

    ir2 = module.ir()
    assert ir == ir2

    file = os.path.join(os.path.dirname(__file__), "resources/test_nested_blocks.ll")
    expected = Path(file).read_text()
    assert ir == expected

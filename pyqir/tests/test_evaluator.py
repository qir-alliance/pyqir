# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import tempfile
from typing import List
from pyqir import (
    BasicQisBuilder,
    Constant,
    IntType,
    PointerType,
    SimpleModule,
    const,
)
from pyqir.evaluator import GateLogger, GateSet, NonadaptiveEvaluator

import pyqir.rt as rt


def test_bell_qir() -> None:
    file = "tests/bell_qir_measure.bc"
    evaluator = NonadaptiveEvaluator()
    logger = GateLogger()
    evaluator.eval(file, logger)

    logger.print()

    assert len(logger.instructions) == 4
    assert str(logger.instructions[0]) == "h qubit[0]"
    assert str(logger.instructions[1]) == "cx qubit[0], qubit[1]"
    assert str(logger.instructions[2]) == "m qubit[0] => out[0]"
    assert str(logger.instructions[3]) == "m qubit[1] => out[1]"


def test_barrier() -> None:
    name = "barrier"
    mod = SimpleModule(f"test_{name}", 0, 0)
    qis = BasicQisBuilder(mod.builder)
    qis.barrier()

    logger = GateLogger()
    _eval(mod, logger, [])
    assert logger.instructions == [
        "barrier",
    ]


def test_swap() -> None:
    name = "swap"
    mod = SimpleModule(f"test_{name}", 2, 0)
    qis = BasicQisBuilder(mod.builder)
    qis.swap(mod.qubits[1], mod.qubits[0])

    logger = GateLogger()
    _eval(mod, logger, [])
    assert logger.instructions == [
        "swap qubit[1], qubit[0]",
    ]


def test_rt_calls_are_noop() -> None:
    name = "rt_calls"
    mod = SimpleModule(f"test_{name}", 2, 1)
    qis = BasicQisBuilder(mod.builder)

    i8p = PointerType(IntType(mod.context, 8))
    rt.initialize(mod.builder, Constant.null(i8p))
    qis.h(mod.qubits[1])
    label = mod.add_byte_string(b"some tag")
    rt.array_record_output(mod.builder, const(IntType(mod.context, 64), 42), label)
    rt.array_record_output(
        mod.builder, const(IntType(mod.context, 64), 7), Constant.null(i8p)
    )
    rt.result_record_output(mod.builder, mod.results[0], label)
    rt.result_record_output(mod.builder, mod.results[0], Constant.null(i8p))
    rt.tuple_record_output(mod.builder, const(IntType(mod.context, 64), 43), label)
    rt.tuple_record_output(
        mod.builder, const(IntType(mod.context, 64), 16), Constant.null(i8p)
    )
    qis.h(mod.qubits[0])

    logger = GateLogger()
    _eval(mod, logger, [])
    assert logger.instructions == ["h qubit[1]", "h qubit[0]"]


def _eval(module: SimpleModule, gates: GateSet, results: List[bool]) -> None:
    with tempfile.NamedTemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        f.flush()
        NonadaptiveEvaluator().eval(f.name, gates, None, results)

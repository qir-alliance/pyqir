# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from functools import partial
from pyqir.generator import BasicQisBuilder, SimpleModule
from pyqir.evaluator import GateLogger, GateSet, NonadaptiveEvaluator
import pytest
import tempfile
from typing import List, Optional

# Combinations of static qubit and result code generation
# first element => use_static_qubit_alloc
# second element => use_static_result_alloc
static_generator_variations = [
    [False, False],
    [False, True],
    [True, False],
    [True, True]
]


def _deprecated_if_result(if_result, *args, **kwargs):
    with pytest.deprecated_call():
        if_result(*args, **kwargs)


ifs = [
    lambda mod, _: mod.if_result,
    lambda _, qis: partial(_deprecated_if_result, qis.if_result),
]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_one_block_executes_on_one(matrix, if_) -> None:
    module = SimpleModule("test_if", num_qubits=1, num_results=1)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    if_(module, qis)(module.results[0], lambda: qis.x(module.qubits[0]))
    print(module.ir())
    logger = GateLogger()
    _eval(module, logger, [True])
    assert logger.instructions == ["m qubit[0] => out[0]", "x qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_zero_block_executes_on_zero(matrix, if_) -> None:
    module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    if_(module, qis)(module.results[0], zero=lambda: qis.x(module.qubits[0]))

    logger = GateLogger()
    _eval(module, logger)
    assert logger.instructions == ["m qubit[0] => out[0]", "x qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_execution_continues_after_hit_conditional_one(matrix, if_) -> None:
    module = SimpleModule("test_if", num_qubits=1, num_results=1)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    if_(module, qis)(module.results[0], lambda: qis.x(module.qubits[0]))
    qis.h(module.qubits[0])

    logger = GateLogger()
    _eval(module, logger, [True])
    assert logger.instructions == [
        "m qubit[0] => out[0]", "x qubit[0]", "h qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_execution_continues_after_missed_conditional_one(matrix, if_) -> None:
    module = SimpleModule("test_if", num_qubits=1, num_results=1)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    if_(module, qis)(module.results[0], lambda: qis.x(module.qubits[0]))
    qis.h(module.qubits[0])

    logger = GateLogger()
    _eval(module, logger, [False])
    assert logger.instructions == ["m qubit[0] => out[0]", "h qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_execution_continues_after_hit_conditional_zero(matrix, if_) -> None:
    module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    if_(module, qis)(module.results[0], zero=lambda: qis.x(module.qubits[0]))
    qis.h(module.qubits[0])

    logger = GateLogger()
    _eval(module, logger, [False])
    assert logger.instructions == [
        "m qubit[0] => out[0]", "x qubit[0]", "h qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_execution_continues_after_missed_conditional_zero(matrix, if_) -> None:
    module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    if_(module, qis)(module.results[0], zero=lambda: qis.x(module.qubits[0]))
    qis.h(module.qubits[0])

    logger = GateLogger()
    _eval(module, logger, [True])
    assert logger.instructions == ["m qubit[0] => out[0]", "h qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_execution_continues_after_conditional_if_else(matrix, if_) -> None:
    module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    if_(module, qis)(module.results[0],
                     lambda: qis.x(module.qubits[0]),
                     lambda: qis.y(module.qubits[0]))
    qis.h(module.qubits[0])

    logger = GateLogger()
    _eval(module, logger)
    assert logger.instructions == [
        "m qubit[0] => out[0]", "y qubit[0]", "h qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_nested_if(matrix, if_) -> None:
    module = SimpleModule("test_if", num_qubits=1, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    qis.m(module.qubits[0], module.results[1])

    if_(module, qis)(
        module.results[0],
        lambda: module.if_result(
            module.results[1],
            lambda: qis.x(module.qubits[0])
        )
    )

    logger = GateLogger()
    _eval(module, logger, [True, True])
    assert logger.instructions == [
        "m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_nested_if_not(matrix, if_) -> None:
    module = SimpleModule("test_if", num_qubits=1, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    qis.m(module.qubits[0], module.results[1])

    if_(module, qis)(
        module.results[0],
        zero=lambda: module.if_result(
            module.results[1],
            zero=lambda: qis.x(module.qubits[0])
        )
    )

    logger = GateLogger()
    _eval(module, logger, [False, False])
    assert logger.instructions == [
        "m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_nested_if_then_else(matrix, if_) -> None:
    module = SimpleModule("test_if_then_else", num_qubits=1, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    qis.m(module.qubits[0], module.results[1])

    if_(module, qis)(
        module.results[0],
        one=lambda: module.if_result(
            module.results[1],
            zero=lambda: qis.x(module.qubits[0])
        )
    )

    logger = GateLogger()
    _eval(module, logger, [True, False])
    assert logger.instructions == [
        "m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_nested_else_then_if(matrix, if_) -> None:
    module = SimpleModule("test_else_then_if", num_qubits=1, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)
    qis.m(module.qubits[0], module.results[0])
    qis.m(module.qubits[0], module.results[1])

    if_(module, qis)(
        module.results[0],
        zero=lambda: module.if_result(
            module.results[1],
            one=lambda: qis.x(module.qubits[0])
        )
    )

    logger = GateLogger()
    _eval(module, logger, [False, True])
    assert logger.instructions == [
        "m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"]


@pytest.mark.parametrize("matrix", static_generator_variations)
@pytest.mark.parametrize("if_", ifs)
def test_results_default_to_zero_if_not_measured(matrix, if_) -> None:
    module = SimpleModule(
        "test_if_not_measured", num_qubits=1, num_results=1
    )
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)

    if_(module, qis)(
        module.results[0],
        one=lambda: qis.x(module.qubits[0]),
        zero=lambda: qis.h(module.qubits[0])
    )

    logger = GateLogger()
    _eval(module, logger)
    assert logger.instructions == ["h qubit[0]"]


def _eval(module: SimpleModule,
          gates: GateSet,
          result_stream: Optional[List[bool]] = None) -> None:
    with tempfile.NamedTemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        f.flush()
        NonadaptiveEvaluator().eval(f.name, gates, None, result_stream)

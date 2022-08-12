# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.evaluator import GateLogger, GateSet, NonadaptiveEvaluator
from pyqir.generator import BasicQisBuilder, ResultRef, SimpleModule, Value
import tempfile
from typing import List, Optional
import pytest


# Combinations of static qubit and result code generation
# first element => use_static_qubit_alloc
# second element => use_static_result_alloc
static_generator_variations = [
    [False, False],
    [False, True],
    [True, False],
    [True, True]
]

def teleport(qis: BasicQisBuilder, qubits: List[Value], results: List[ResultRef]) -> None:
    msg = qubits[0]
    target = qubits[1]
    register = qubits[2]

    # Create some entanglement that we can use to send our message.
    qis.h(register)
    qis.cx(register, target)

    # Encode the message into the entangled pair.
    qis.cx(msg, register)
    qis.h(msg)

    # Measure the qubits to extract the classical data we need to decode the
    # message by applying the corrections on the target qubit accordingly.
    qis.m(msg, results[0])
    qis.reset(msg)
    qis.if_result(results[0], one=lambda: qis.z(target))

    qis.m(register, results[1])
    qis.reset(register)
    qis.if_result(results[1], one=lambda: qis.x(target))


@pytest.mark.parametrize("matrix", static_generator_variations)
def test_teleport_measures_zero_zero(matrix) -> None:
    module = SimpleModule("teleport00", num_qubits=3, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)

    teleport(qis, module.qubits, module.results)

    logger = GateLogger()
    _eval(module, logger, [False, False])
    assert logger.instructions == [
            "h qubit[2]",
            "cx qubit[2], qubit[1]",
            "cx qubit[0], qubit[2]",
            "h qubit[0]",
            "m qubit[0] => out[0]",
            "reset 0",
            "m qubit[2] => out[1]",
            "reset 2",
        ]

@pytest.mark.parametrize("matrix", static_generator_variations)
def test_teleport_measures_zero_one(matrix) -> None:
    module = SimpleModule("teleport01", num_qubits=3, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)

    teleport(qis, module.qubits, module.results)

    logger = GateLogger()
    _eval(module, logger, [False, True])
    assert logger.instructions == [
            "h qubit[2]",
            "cx qubit[2], qubit[1]",
            "cx qubit[0], qubit[2]",
            "h qubit[0]",
            "m qubit[0] => out[0]",
            "reset 0",
            "m qubit[2] => out[1]",
            "reset 2",
            "x qubit[1]",
        ]

@pytest.mark.parametrize("matrix", static_generator_variations)
def test_teleport_measures_one_zero(matrix) -> None:
    module = SimpleModule("teleport10", num_qubits=3, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)

    teleport(qis, module.qubits, module.results)

    logger = GateLogger()
    _eval(module, logger, [True, False])
    assert logger.instructions == [
            "h qubit[2]",
            "cx qubit[2], qubit[1]",
            "cx qubit[0], qubit[2]",
            "h qubit[0]",
            "m qubit[0] => out[0]",
            "reset 0",
            "z qubit[1]",
            "m qubit[2] => out[1]",
            "reset 2",
        ]

@pytest.mark.parametrize("matrix", static_generator_variations)
def test_teleport_measures_one_one(matrix) -> None:
    module = SimpleModule("teleport11", num_qubits=3, num_results=2)
    module.use_static_qubit_alloc(matrix[0])
    module.use_static_result_alloc(matrix[1])
    qis = BasicQisBuilder(module.builder)

    teleport(qis, module.qubits, module.results)

    logger = GateLogger()
    _eval(module, logger, [True, True])
    assert logger.instructions == [
            "h qubit[2]",
            "cx qubit[2], qubit[1]",
            "cx qubit[0], qubit[2]",
            "h qubit[0]",
            "m qubit[0] => out[0]",
            "reset 0",
            "z qubit[1]",
            "m qubit[2] => out[1]",
            "reset 2",
            "x qubit[1]",
        ]


def _eval(module: SimpleModule,
          gates: GateSet,
          result_stream: Optional[List[bool]] = None) -> None:
    with tempfile.NamedTemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        f.flush()
        NonadaptiveEvaluator().eval(f.name, gates, None, result_stream)

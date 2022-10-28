# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.evaluator import GateLogger, NonadaptiveEvaluator
from pyqir.generator import BasicQisBuilder, SimpleModule, Value
import tempfile
from typing import List


def teleport(qis: BasicQisBuilder, qubits: List[Value], results: List[Value]) -> None:
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
    qis.mz(msg, results[0])
    qis.reset(msg)
    qis.if_result(results[0], one=lambda: qis.z(target))

    qis.mz(register, results[1])
    qis.reset(register)
    qis.if_result(results[1], one=lambda: qis.x(target))


def eval(path: str, results: List[bool]) -> None:
    logger = GateLogger()
    NonadaptiveEvaluator().eval(path, logger, None, results)
    logger.print()


module = SimpleModule("teleport", num_qubits=3, num_results=2)
qis = BasicQisBuilder(module.builder)
teleport(qis, module.qubits, module.results)

with tempfile.NamedTemporaryFile(suffix=".ll") as teleport_ll:
    teleport_ll.write(module.ir().encode("utf-8"))
    teleport_ll.flush()

    print("# Evaluating both results as 0's", flush=True)
    eval(teleport_ll.name, [False, False])

    print("# Evaluating first result as 0, second as 1", flush=True)
    eval(teleport_ll.name, [False, True])

    print("# Evaluating first result as 1, second as 0", flush=True)
    eval(teleport_ll.name, [True, False])

    print("# Evaluating both results as 1's", flush=True)
    eval(teleport_ll.name, [True, True])

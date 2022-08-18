# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.evaluator import GateLogger, GateSet, NonadaptiveEvaluator
from pyqir.generator import BasicQisBuilder, ResultRef, SimpleModule, Value
import tempfile
from typing import List, Optional


def teleport(module: SimpleModule, qis: BasicQisBuilder, qubits: List[Value], results: List[ResultRef]) -> None:
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
    module.if_result(results[0], one=lambda: qis.z(target))

    qis.m(register, results[1])
    qis.reset(register)
    module.if_result(results[1], one=lambda: qis.x(target))


def _eval(module: SimpleModule,
          gates: GateSet,
          result_stream: Optional[List[bool]] = None) -> None:
    with tempfile.NamedTemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        f.flush()
        NonadaptiveEvaluator().eval(f.name, gates, None, result_stream)


module = SimpleModule("teleport-example", num_qubits=3, num_results=2)
qis = BasicQisBuilder(module.builder)

teleport(module, qis, module.qubits, module.results)

print("# Evaluating both results as 0's", flush=True)
logger = GateLogger()
_eval(module, logger, [False, False])
logger.print()

print("# Evaluating first result as 0, second as 1", flush=True)
logger = GateLogger()
_eval(module, logger, [False, True])
logger.print()

print("# Evaluating first result as 1, second as 0", flush=True)
logger = GateLogger()
_eval(module, logger, [True, False])
logger.print()

print("# Evaluating both results as 1's", flush=True)
logger = GateLogger()
_eval(module, logger, [True, True])
logger.print()

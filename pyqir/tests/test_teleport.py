# # Copyright (c) Microsoft Corporation.
# # Licensed under the MIT License.

# from pyqir import BasicQisBuilder, SimpleModule, Value
# from pyqir.evaluator import GateLogger, GateSet, NonadaptiveEvaluator
# import tempfile
# from typing import List, Sequence


# def _teleport(
#     qis: BasicQisBuilder, qubits: Sequence[Value], results: Sequence[Value]
# ) -> None:
#     msg = qubits[0]
#     target = qubits[1]
#     register = qubits[2]

#     # Create some entanglement that we can use to send our message.
#     qis.h(register)
#     qis.cx(register, target)

#     # Encode the message into the entangled pair.
#     qis.cx(msg, register)
#     qis.h(msg)

#     # Measure the qubits to extract the classical data we need to decode the
#     # message by applying the corrections on the target qubit accordingly.
#     qis.mz(msg, results[0])
#     qis.reset(msg)
#     qis.if_result(results[0], one=lambda: qis.z(target))

#     qis.mz(register, results[1])
#     qis.reset(register)
#     qis.if_result(results[1], one=lambda: qis.x(target))


# def _eval(module: SimpleModule, gates: GateSet, results: List[bool]) -> None:
#     with tempfile.NamedTemporaryFile(suffix=".ll") as f:
#         f.write(module.ir().encode("utf-8"))
#         f.flush()
#         NonadaptiveEvaluator().eval(f.name, gates, None, results)


# def test_teleport_measures_zero_zero() -> None:
#     module = SimpleModule("teleport00", num_qubits=3, num_results=2)
#     qis = BasicQisBuilder(module.builder)
#     _teleport(qis, module.qubits, module.results)

#     logger = GateLogger()
#     _eval(module, logger, [False, False])
#     assert logger.instructions == [
#         "h qubit[2]",
#         "cx qubit[2], qubit[1]",
#         "cx qubit[0], qubit[2]",
#         "h qubit[0]",
#         "m qubit[0] => out[0]",
#         "reset 0",
#         "m qubit[2] => out[1]",
#         "reset 2",
#     ]


# def test_teleport_measures_zero_one() -> None:
#     module = SimpleModule("teleport01", num_qubits=3, num_results=2)
#     qis = BasicQisBuilder(module.builder)
#     _teleport(qis, module.qubits, module.results)

#     logger = GateLogger()
#     _eval(module, logger, [False, True])
#     assert logger.instructions == [
#         "h qubit[2]",
#         "cx qubit[2], qubit[1]",
#         "cx qubit[0], qubit[2]",
#         "h qubit[0]",
#         "m qubit[0] => out[0]",
#         "reset 0",
#         "m qubit[2] => out[1]",
#         "reset 2",
#         "x qubit[1]",
#     ]


# def test_teleport_measures_one_zero() -> None:
#     module = SimpleModule("teleport10", num_qubits=3, num_results=2)
#     qis = BasicQisBuilder(module.builder)
#     _teleport(qis, module.qubits, module.results)

#     logger = GateLogger()
#     _eval(module, logger, [True, False])
#     assert logger.instructions == [
#         "h qubit[2]",
#         "cx qubit[2], qubit[1]",
#         "cx qubit[0], qubit[2]",
#         "h qubit[0]",
#         "m qubit[0] => out[0]",
#         "reset 0",
#         "z qubit[1]",
#         "m qubit[2] => out[1]",
#         "reset 2",
#     ]


# def test_teleport_measures_one_one() -> None:
#     module = SimpleModule("teleport11", num_qubits=3, num_results=2)
#     qis = BasicQisBuilder(module.builder)
#     _teleport(qis, module.qubits, module.results)

#     logger = GateLogger()
#     _eval(module, logger, [True, True])
#     assert logger.instructions == [
#         "h qubit[2]",
#         "cx qubit[2], qubit[1]",
#         "cx qubit[0], qubit[2]",
#         "h qubit[0]",
#         "m qubit[0] => out[0]",
#         "reset 0",
#         "z qubit[1]",
#         "m qubit[2] => out[1]",
#         "reset 2",
#         "x qubit[1]",
#     ]

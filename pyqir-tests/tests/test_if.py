# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule
from pyqir.evaluator import GateLogger, GateSet, NonadaptiveEvaluator
import tempfile
from typing import List, Optional
import unittest


class IfTestCase(unittest.TestCase):
    def test_one_block_executes_on_one(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], lambda: qis.x(module.qubits[0]))

        logger = GateLogger()
        _eval(module, logger, [True])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "x qubit[0]"])

    def test_zero_block_executes_on_zero(self) -> None:
        module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], zero=lambda: qis.x(module.qubits[0]))

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "x qubit[0]"])

    def test_execution_continues_after_hit_conditional_one(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], lambda: qis.x(module.qubits[0]))
        qis.h(module.qubits[0])

        logger = GateLogger()
        _eval(module, logger, [True])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "x qubit[0]", "h qubit[0]"])

    def test_execution_continues_after_missed_conditional_one(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], lambda: qis.x(module.qubits[0]))
        qis.h(module.qubits[0])

        logger = GateLogger()
        _eval(module, logger, [False])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "h qubit[0]"])

    def test_execution_continues_after_hit_conditional_zero(self) -> None:
        module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], zero=lambda: qis.x(module.qubits[0]))
        qis.h(module.qubits[0])

        logger = GateLogger()
        _eval(module, logger, [False])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "x qubit[0]", "h qubit[0]"])

    def test_execution_continues_after_missed_conditional_zero(self) -> None:
        module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0], zero=lambda: qis.x(module.qubits[0]))
        qis.h(module.qubits[0])

        logger = GateLogger()
        _eval(module, logger, [True])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "h qubit[0]"])

    def test_execution_continues_after_conditional_if_else(self) -> None:
        module = SimpleModule("test_if_not", num_qubits=1, num_results=1)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.if_result(module.results[0],
                      lambda: qis.x(module.qubits[0]),
                      lambda: qis.y(module.qubits[0]))
        qis.h(module.qubits[0])

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "y qubit[0]", "h qubit[0]"])

    def test_nested_if(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=2)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.m(module.qubits[0], module.results[1])

        qis.if_result(
            module.results[0],
            lambda: qis.if_result(
                module.results[1],
                lambda: qis.x(module.qubits[0])
            )
        )

        logger = GateLogger()
        _eval(module, logger, [True, True])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"])

    def test_nested_if_not(self) -> None:
        module = SimpleModule("test_if", num_qubits=1, num_results=2)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.m(module.qubits[0], module.results[1])

        qis.if_result(
            module.results[0],
            zero=lambda: qis.if_result(
                module.results[1],
                zero=lambda: qis.x(module.qubits[0])
            )
        )

        logger = GateLogger()
        _eval(module, logger, [False, False])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"])

    def test_nested_if_then_else(self) -> None:
        module = SimpleModule("test_if_then_else", num_qubits=1, num_results=2)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.m(module.qubits[0], module.results[1])

        qis.if_result(
            module.results[0],
            one=lambda: qis.if_result(
                module.results[1],
                zero=lambda: qis.x(module.qubits[0])
            )
        )

        logger = GateLogger()
        _eval(module, logger, [True, False])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"])

    def test_nested_else_then_if(self) -> None:
        module = SimpleModule("test_else_then_if", num_qubits=1, num_results=2)
        qis = BasicQisBuilder(module.builder)
        qis.m(module.qubits[0], module.results[0])
        qis.m(module.qubits[0], module.results[1])

        qis.if_result(
            module.results[0],
            zero=lambda: qis.if_result(
                module.results[1],
                one=lambda: qis.x(module.qubits[0])
            )
        )

        logger = GateLogger()
        _eval(module, logger, [False, True])
        self.assertEqual(logger.instructions,
                         ["m qubit[0] => out[0]", "m qubit[0] => out[1]", "x qubit[0]"])

    def test_results_default_to_zero_if_not_measured(self) -> None:
        module = SimpleModule(
            "test_if_not_measured", num_qubits=1, num_results=1
        )
        qis = BasicQisBuilder(module.builder)

        qis.if_result(
            module.results[0],
            one=lambda: qis.x(module.qubits[0]),
            zero=lambda: qis.h(module.qubits[0])
        )

        logger = GateLogger()
        _eval(module, logger)
        self.assertEqual(logger.instructions, ["h qubit[0]"])


def _eval(module: SimpleModule,
          gates: GateSet,
          result_stream: Optional[List[bool]] = None) -> None:
    with tempfile.NamedTemporaryFile(suffix=".ll") as f:
        f.write(module.ir().encode("utf-8"))
        f.flush()
        NonadaptiveEvaluator().eval(f.name, gates, None, result_stream)


if __name__ == "__main__":
    unittest.main()

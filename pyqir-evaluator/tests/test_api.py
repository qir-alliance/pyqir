# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.evaluator import GateLogger, NonadaptiveJit


def test_bell_qir():
    file = "tests/bell_qir_measure.bc"
    jit = NonadaptiveJit()
    logger = GateLogger()
    jit.eval(file, logger)

    logger.print()

    assert len(logger.instructions) == 4
    assert str(logger.instructions[0]) == "h qubit[0]"
    assert str(logger.instructions[1]) == "cx qubit[0], qubit[1]"
    assert str(logger.instructions[2]) == "m qubit[0] => out[0]"
    assert str(logger.instructions[3]) == "m qubit[1] => out[1]"

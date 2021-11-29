# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_jit import NonadaptiveJit, GateLogger


def test_bell_qir():
    file = "tests/bell_qir_measure.ll"
    jit = NonadaptiveJit()
    logger = GateLogger()
    jit.eval(file, logger)

    logger.print()

    assert len(logger.instructions) == 2
    assert str(logger.instructions[0]).startswith("h")
    assert str(logger.instructions[1]).startswith("cx")


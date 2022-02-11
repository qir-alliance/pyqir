# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.jit.nonadaptivejit import NonadaptiveJit
from pyqir.jit.gatelogger import GateLogger

def test_bell_qir():
    file = "tests/bell_qir_measure.bc"
    jit = NonadaptiveJit()
    logger = GateLogger()
    jit.eval(file, logger)

    logger.print()

    assert len(logger.instructions) == 2
    assert str(logger.instructions[0]).startswith("h")
    assert str(logger.instructions[1]).startswith("cx")

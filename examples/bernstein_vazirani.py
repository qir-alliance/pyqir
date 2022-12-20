#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from pathlib import Path
from pyqir.evaluator import GateLogger, NonadaptiveEvaluator

path = Path(__file__).parent
file = os.path.join(path, "bernstein_vazirani.bc")

evaluator = NonadaptiveEvaluator()
logger = GateLogger()

evaluator.eval(file, logger)

print("# output from GateLogger", flush=True)
logger.print()

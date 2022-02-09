#!/usr/bin/env python3

# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_jit import NonadaptiveJit, GateLogger

from pathlib import Path
import os

path = Path(__file__).parent
file = os.path.join(path, "bernstein_vazirani.bc")

jit = NonadaptiveJit()
logger = GateLogger()

jit.eval(file, logger)

print("# output from GateLogger", flush=True)
logger.print()

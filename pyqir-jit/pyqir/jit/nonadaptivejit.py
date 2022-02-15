# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List, Optional
from pyqir.jit.gateset import GateSet
from pyqir.jit._native import PyNonadaptiveJit


class NonadaptiveJit(object):
    """
    The NonadaptiveJit object loads bitcode/QIR for evaluation and processing

    jit: PyNonadaptiveJit
    """

    def __init__(self):
        self.jit = PyNonadaptiveJit()

    def eval(self,
             file_path: str,
             gateset: GateSet,
             entry_point: Optional[str] = None,
             result_stream: Optional[List[bool]] = None):
        """
        JIT compiles the circuit delegating quantum operations to the supplied
        GateSet

        :param file_path: file path of existing QIR in a ll or bc file
        :param gateset: python GateSet based object defining the operations
        :param entry_point: entry point name; required if QIR contains multiple entry points
        :param result_stream: list of boolean result values representing the QIS measure results
        """
        self.jit.eval(file_path, gateset, entry_point, result_stream)

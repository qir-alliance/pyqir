# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Optional
from pyqir_jit import GateSet
from .pyqir_jit import PyNonadaptiveJit


class NonadaptiveJit(object):
    """
    The NonadaptiveJit object loads bitcode/QIR for evaluation and processing
    with no classical logic such as branching based on measurement.

    jit: PyNonadaptiveJit
    """

    def __init__(self):
        self.jit = PyNonadaptiveJit()

    def eval(self,
             file_path: str,
             gateset: GateSet,
             entry_point: Optional[str] = None
             ) -> None:
        """
        JIT compiles the circuit delegating quantum operations to the supplied
        GateSet

        :param file_path: file path of existing QIR in a ll or bc file
        :param gateset: python GateSet based object defining the operations
        :param entry_point: entry point name; required if QIR contains multiple entry points
        """
        self.jit.eval(file_path, gateset, entry_point)

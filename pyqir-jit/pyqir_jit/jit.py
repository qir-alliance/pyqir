# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_jit import GateSet
from .pyqir_jit import PyNonadaptiveJit


class NonadaptiveJit(object):
    """
    The NonadaptiveJit object loads bitcode/QIR for evaluation and processing

    jit: PyNonadaptiveJit
    """

    def __init__(self):
        self.jit = PyNonadaptiveJit()

    def eval(self, file_path: str, gateset: GateSet):
        """
        JIT compiles the circuit delegating quantum operations to the supplied
        GateSet

        :param file_path: file path of existing QIR in a ll or bc file
        :type file_path: str

        :param gateset: python GateSet based object defining the operations
        :type gateset: GateSet
        """
        self.jit.eval(file_path, gateset)

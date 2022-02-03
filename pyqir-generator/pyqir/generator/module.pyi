# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator.builder import Builder
from pyqir.generator.value import Qubit, Ref
from typing import Tuple


class SimpleModule:
    """
    A simple module represents an executable QIR program with these
    restrictions:

    - There is one global quantum register and one global classical register.
      Both are allocated automatically with a fixed size before the program
      starts.
    - There is only a single function that runs as the entry point.
    """

    def __init__(
        self,
        name: str,
        num_qubits: int,
        num_results: int,
    ) -> None:
        """
        Initializes the module with a name and the number of qubits and results
        in the quantum and classical registers.

        :param name: The name of the module.
        :param num_qubits: The size of the global quantum register.
        :param num_results: The size of the global classical register.
        """
        ...

    @property
    def qubits(self) -> Tuple[Qubit, ...]:
        """A sequence of qubits representing the global quantum register."""
        ...

    @property
    def results(self) -> Tuple[Ref, ...]:
        """A sequence of references to results representing the global classical register."""
        ...

    @property
    def builder(self) -> Builder:
        """The instruction builder."""
        ...

    def ir(self) -> str:
        """Emits the LLVM IR for the module as plain text."""
        ...

    def bitcode(self) -> bytes:
        """Emits the LLVM bitcode for the module as a sequence of bytes."""
        ...

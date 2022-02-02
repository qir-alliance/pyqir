# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir_generator.builder import Builder
from pyqir_generator.value import Qubit, Ref
from typing import Tuple


class SimpleModule:
    def __init__(
        self,
        name: str,
        num_qubits: int,
        num_results: int,
    ) -> None: ...

    @property
    def qubits(self) -> Tuple[Qubit, ...]: ...

    @property
    def results(self) -> Tuple[Ref, ...]: ...

    @property
    def builder(self) -> Builder: ...

    def ir(self) -> str: ...

    def bitcode(self) -> bytes: ...

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.evaluator._gateset import GateSet
from typing import List, Optional

class PyNonadaptiveJit:
    def eval(
        self,
        file_path: str,
        gateset: GateSet,
        entry_point: Optional[str],
        result_stream: Optional[List[bool]],
    ) -> None: ...

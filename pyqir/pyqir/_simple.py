# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List, Optional

import pyqir
from pyqir import (
    BasicBlock,
    Builder,
    Constant,
    Context,
    Function,
    FunctionType,
    Linkage,
    Module,
    Value,
)


class SimpleModule:
    """
    A simple module represents an executable program with these restrictions:

    - There is one global qubit register and one global result register. Both
      are statically allocated with a fixed size.
    - There is only a single function that runs as the entry point.
    """

    def __init__(
        self,
        name: str,
        num_qubits: int,
        num_results: int,
        context: Optional[Context] = None,
    ) -> None:
        """
        Initializes a simple module.

        :param name: The name of the module.
        :param num_qubits: The number of statically allocated qubits.
        :param num_results: The number of statically allocated results.
        :param context: The global context.
        """

        if context is None:
            context = Context()

        self._module = Module(context, name)
        self._builder = Builder(context)
        self._num_qubits = num_qubits
        self._num_results = num_results

        entry_point = pyqir.entry_point(self._module, "main", num_qubits, num_results)
        self._builder.insert_at_end(BasicBlock(context, "entry", entry_point))

    @property
    def context(self) -> Context:
        """The global context."""
        return self._module.context

    @property
    def qubits(self) -> List[Value]:
        """The global qubit register."""
        return [pyqir.qubit(self.context, id) for id in range(self._num_qubits)]

    @property
    def results(self) -> List[Value]:
        """The global result register."""
        return [pyqir.result(self.context, id) for id in range(self._num_results)]

    @property
    def builder(self) -> Builder:
        """The instruction builder."""
        return self._builder

    def ir(self) -> str:
        """Emits the LLVM IR for the module as plain text."""

        ret = self._builder.ret(None)
        error = self._module.verify()
        if error is not None:
            raise ValueError(error)

        ir = str(self._module)
        ret.erase()
        return ir

    def bitcode(self) -> bytes:
        """Emits the LLVM bitcode for the module as a sequence of bytes."""

        ret = self._builder.ret(None)
        error = self._module.verify()
        if error is not None:
            raise ValueError(error)

        bitcode = self._module.bitcode
        ret.erase()
        return bitcode

    def add_external_function(self, name: str, ty: FunctionType) -> Function:
        """
        Adds a declaration for an externally linked function to the module.

        :param name: The name of the function.
        :param ty: The type of the function.
        :return: The function value.
        """
        return Function(ty, Linkage.EXTERNAL, name, self._module)

    def add_byte_string(self, value: bytes) -> Constant:
        """Adds a global null-terminated byte string constant to the module.

        :param bytes Value: The byte string value without a null terminator.
        :returns: A pointer to the start of the null-terminated byte string.
        """
        return pyqir.global_byte_string(self._module, value)

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
    ModuleFlagBehavior,
    Value,
)


class SimpleModule:
    """
    A simple module represents a QIR program with the following assumptions:

    - All qubits and results are statically allocated.
    - There is exactly one function that is not externally linked, which is the entry point.
    """

    def __init__(
        self,
        name: str,
        num_qubits: int,
        num_results: int,
        context: Optional[Context] = None,
        entry_point_name: str = "main",
    ) -> None:
        """
        Initializes a simple module.

        :param str name: The name of the module.
        :param str num_qubits: The number of statically allocated qubits.
        :param int num_results: The number of statically allocated results.
        :param Optional[Context] context: The LLVM context.
        :param str entry_point_name: The name of the entry point function.
        """

        if context is None:
            context = Context()

        self._module = pyqir.qir_module(
            context,
            name,
            qir_major_version=2,
            qir_minor_version=0,
            dynamic_qubit_management=False,
            dynamic_result_management=False,
        )
        self._builder = Builder(context)
        self._num_qubits = num_qubits
        self._num_results = num_results

        self._entry_point = pyqir.entry_point(
            self._module, entry_point_name, num_qubits, num_results
        )
        self._entry_block = BasicBlock(context, "entry", self._entry_point)
        self._builder.insert_at_end(self._entry_block)

    @property
    def context(self) -> Context:
        """The LLVM context."""
        return self._module.context

    @property
    def qubits(self) -> List[Value]:
        """The list of statically allocated qubits indexed by their numeric ID."""
        return [pyqir.qubit(self.context, id) for id in range(self._num_qubits)]

    @property
    def results(self) -> List[Value]:
        """The list of statically allocated results indexed by their numeric ID."""
        return [pyqir.result(self.context, id) for id in range(self._num_results)]

    @property
    def builder(self) -> Builder:
        """The instruction builder."""
        return self._builder

    @property
    def entry_point(self) -> Function:
        """The entry point function (automatically generated)."""
        return self._entry_point

    @property
    def entry_block(self) -> BasicBlock:
        """The first basic block of the entry point (automatically generated)."""
        return self._entry_block

    def add_external_function(self, name: str, ty: FunctionType) -> Function:
        """
        Adds a declaration for an externally linked function to the module.

        :param name: The name of the function.
        :param ty: The type of the function.
        :returns: The function value.
        """
        return Function(ty, Linkage.EXTERNAL, name, self._module)

    def add_byte_string(self, value: bytes) -> Constant:
        """
        Adds a global null-terminated byte string constant to the module.

        :param Value: The byte string value without a null terminator.
        :returns: A pointer to the start of the null-terminated byte string.
        """
        return pyqir.global_byte_string(self._module, value)

    def ir(self) -> str:
        """Emits the LLVM IR for the module as plain text."""
        ret = self._builder.ret(None)
        try:
            error = self._module.verify()
            if error is not None:
                raise ValueError(error)
            return str(self._module)
        finally:
            ret.erase()

    def bitcode(self) -> bytes:
        """Emits the LLVM bitcode for the module as a sequence of bytes."""
        ret = self._builder.ret(None)
        try:
            error = self._module.verify()
            if error is not None:
                raise ValueError(error)
            return self._module.bitcode
        finally:
            ret.erase()

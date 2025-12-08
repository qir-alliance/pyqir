# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import Optional, List, OrderedDict
from pyqir._native import (
    Builder,
    Module,
    Function,
    BasicBlock,
    Instruction,
    Call,
    Opcode,
    Value,
    is_entry_point,
)


class QirModuleVisitor:
    """
    Base class for all QIR passes that visits each ofthe underlying components of the module.
    A QIR visitor can be used to analyze or transform a QIR module. Each method in the visitor pattern
    where each method corresponds to a specific QIR construct that it can handle. A specific pass can be
    implemented by subclassing this class and overriding the methods of interest. The default implementation
    of each method does nothing besides calling the corresponding methods on its constituent parts, traversing
    the remainder of the module.

    When invoked on a module, this base class initializes a `Builder` instance in the `_builder` member that
    can be used to create new instructions and types for use in transformations on the module.
    """

    _builder: Optional[Builder] = None
    """
    The builder instance used to create new instructions and types for the current module, if any.
    """

    def __init__(self) -> None:
        """
        Initializes the QIR pass. Base implementation does nothing.
        """
        pass

    @property
    def builder(self) -> Builder:
        """
        Returns the builder instance used to create new instructions and types for the current module.

        :return: The builder instance.
        """
        assert self._builder is not None
        return self._builder

    def run(self, qir: Module) -> None:
        """
        Applies this pass to the given QIR. The pass is applied in-place to the given module.

        :param qir: The QIR to apply the pass to as a `Module`.
        """
        self._on_module(qir)

    def _on_module(self, module: Module) -> None:
        """
        Invoked when the pass is run on a module.
        This is the starting point for the pass. Each function in the module is visited in the order of
        declarations, followed by non-entry definitions, followed by QIR entry point(s).

        During this traversal, the `builder` attribute is initialized to a new `Builder` instance
        for the module's context. This `Builder` instance can be used to create new instructions and types
        for use in transformations on the module.
        """
        err = module.verify()
        if err is not None:
            raise ValueError(err)
        self._builder = Builder(module.context)
        for function in filter(
            lambda f: not is_entry_point(f) and len(f.basic_blocks) == 0,
            module.functions,
        ):
            self._on_function(function)
        for function in filter(
            lambda f: not is_entry_point(f) and len(f.basic_blocks) > 0,
            module.functions,
        ):
            self._on_function(function)
        for function in filter(lambda f: is_entry_point(f), module.functions):
            self._on_function(function)
        err = module.verify()
        if err is not None:
            raise ValueError(err)
        self._builder = None

    def _on_function(self, function: Function) -> None:
        """
        Invoked for each function definition and declaration in the module.
        Each block in the function is visited, starting with the entry block and proceeding
        to other blocks in order determined by the topology of the control flow graph. This
        attempts (but is not guaranteed) to visit a given block after its predecessors.
        """
        blocks: List[BasicBlock] = []
        if len(function.basic_blocks):
            blocks.append(function.basic_blocks[0])
        to_visit = OrderedDict()
        for block in blocks:
            to_visit[block] = block
            if block.terminator is not None:
                # Note: successors are in reverse order in the QIR, so to visit in a
                # topological order for programs that happen to be a Directed Acyclic Graph,
                # we need to reverse the order of successors.
                for b in reversed(block.terminator.successors):
                    if b not in to_visit:
                        blocks.append(b)
        for block in to_visit.values():
            self._on_block(block)

    def _on_block(self, block: BasicBlock) -> None:
        """
        Invoked for each basic block in a function.
        Each instruction in the block is visited in order, including the terminator instruction.
        """
        for instruction in block.instructions:
            self._on_instruction(instruction)

    def _on_instruction(self, instruction: Instruction) -> None:
        """
        Invoked for each instruction in a basic block.
        Each instruction is dispatched to a handler based on its opcode, if available.
        """
        opcode = instruction.opcode
        if opcode == Opcode.CALL:
            assert isinstance(instruction, Call)
            self._on_call_instr(instruction)
        else:
            pass

    def _on_call_instr(self, call: Call) -> None:
        """
        Invoked for each call instruction in a basic block.
        Will dispatch to a handler based on the recognized __quantum__qis__* or __quantum__rt__* callee, if available.
        """
        callee_name = call.callee.name
        if callee_name == "__quantum__qis__ccx__body":
            self._on_qis_ccx(call, call.args[0], call.args[1], call.args[2])
        elif callee_name == "__quantum__qis__cx__body":
            self._on_qis_cx(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__cy__body":
            self._on_qis_cy(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__cz__body":
            self._on_qis_cz(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__swap__body":
            self._on_qis_swap(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__rx__body":
            self._on_qis_rx(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__rxx__body":
            self._on_qis_rxx(call, call.args[0], call.args[1], call.args[2])
        elif callee_name == "__quantum__qis__ry__body":
            self._on_qis_ry(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__ryy__body":
            self._on_qis_ryy(call, call.args[0], call.args[1], call.args[2])
        elif callee_name == "__quantum__qis__rz__body":
            self._on_qis_rz(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__rzz__body":
            self._on_qis_rzz(call, call.args[0], call.args[1], call.args[2])
        elif callee_name == "__quantum__qis__h__body":
            self._on_qis_h(call, call.args[0])
        elif callee_name == "__quantum__qis__s__body":
            self._on_qis_s(call, call.args[0])
        elif callee_name == "__quantum__qis__s__adj":
            self._on_qis_s_adj(call, call.args[0])
        elif callee_name == "__quantum__qis__t__body":
            self._on_qis_t(call, call.args[0])
        elif callee_name == "__quantum__qis__t__adj":
            self._on_qis_t_adj(call, call.args[0])
        elif callee_name == "__quantum__qis__x__body":
            self._on_qis_x(call, call.args[0])
        elif callee_name == "__quantum__qis__y__body":
            self._on_qis_y(call, call.args[0])
        elif callee_name == "__quantum__qis__z__body":
            self._on_qis_z(call, call.args[0])
        elif callee_name == "__quantum__qis__m__body":
            self._on_qis_m(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__mz__body":
            self._on_qis_mz(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__mresetz__body":
            self._on_qis_mresetz(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__qis__reset__body":
            self._on_qis_reset(call, call.args[0])
        elif callee_name == "__quantum__qis__read_result__body":
            self._on_qis_read_result(call, call.args[0])
        elif callee_name == "__quantum__rt__result_record_output":
            self._on_rt_result_record_output(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__rt__bool_record_output":
            self._on_rt_bool_record_output(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__rt__int_record_output":
            self._on_rt_int_record_output(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__rt__double_record_output":
            self._on_rt_double_record_output(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__rt__tuple_record_output":
            self._on_rt_tuple_record_output(call, call.args[0], call.args[1])
        elif callee_name == "__quantum__rt__array_record_output":
            self._on_rt_array_record_output(call, call.args[0], call.args[1])
        else:
            pass

    def _on_qis_ccx(
        self, call: Call, ctrl1: Value, ctrl2: Value, target: Value
    ) -> None:
        """
        Invoked for each call instruction to a CCX gate in a basic block.
        """
        pass

    def _on_qis_cx(self, call: Call, ctrl: Value, target: Value) -> None:
        """
        Invoked for each call instruction to a CX gate in a basic block.
        """
        pass

    def _on_qis_cy(self, call: Call, ctrl: Value, target: Value) -> None:
        """
        Invoked for each call instruction to a CY gate in a basic block.
        """
        pass

    def _on_qis_cz(self, call: Call, ctrl: Value, target: Value) -> None:
        """
        Invoked for each call instruction to a CZ gate in a basic block.
        """
        pass

    def _on_qis_swap(self, call: Call, target1: Value, target2: Value) -> None:
        """
        Invoked for each call instruction to a SWAP gate in a basic block.
        """
        pass

    def _on_qis_rx(self, call: Call, angle: Value, target: Value) -> None:
        """
        Invoked for each call instruction to an Rx gate in a basic block.
        """
        pass

    def _on_qis_rxx(
        self, call: Call, angle: Value, target1: Value, target2: Value
    ) -> None:
        """
        Invoked for each call instruction to an Rxx gate in a basic block.
        """
        pass

    def _on_qis_ry(self, call: Call, angle: Value, target: Value) -> None:
        """
        Invoked for each call instruction to an Ry gate in a basic block.
        """
        pass

    def _on_qis_ryy(
        self, call: Call, angle: Value, target1: Value, target2: Value
    ) -> None:
        """
        Invoked for each call instruction to an Ryy gate in a basic block.
        """
        pass

    def _on_qis_rz(self, call: Call, angle: Value, target: Value) -> None:
        """
        Invoked for each call instruction to an Rz gate in a basic block.
        """
        pass

    def _on_qis_rzz(
        self, call: Call, angle: Value, target1: Value, target2: Value
    ) -> None:
        """
        Invoked for each call instruction to an Rzz gate in a basic block.
        """
        pass

    def _on_qis_h(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to an H gate in a basic block.
        """
        pass

    def _on_qis_s(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to an S gate in a basic block.
        """
        pass

    def _on_qis_s_adj(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to an adjoint S gate in a basic block.
        """
        pass

    def _on_qis_t(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to a T gate in a basic block.
        """
        pass

    def _on_qis_t_adj(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to an adjoint T gate in a basic block.
        """
        pass

    def _on_qis_x(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to an X gate in a basic block.
        """
        pass

    def _on_qis_y(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to a Y gate in a basic block.
        """
        pass

    def _on_qis_z(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to a Z gate in a basic block.
        """
        pass

    def _on_qis_m(self, call: Call, target: Value, result: Value) -> None:
        """
        Invoked for each call instruction to non-destructive measurement M in a basic block.
        """
        pass

    def _on_qis_mz(self, call: Call, target: Value, result: Value) -> None:
        """
        Invoked for each call instruction to non-destructive measurement Mz in a basic block.
        """
        pass

    def _on_qis_mresetz(self, call: Call, target: Value, result: Value) -> None:
        """
        Invoked for each call instruction to destructive measurement MResetZ in a basic block.
        """
        pass

    def _on_qis_reset(self, call: Call, target: Value) -> None:
        """
        Invoked for each call instruction to Reset in a basic block.
        """
        pass

    def _on_qis_read_result(self, call: Call, result: Value) -> None:
        """
        Invoked for each call instruction to read a result value in a basic block.
        """
        pass

    def _on_rt_result_record_output(
        self, call: Call, result: Value, target: Value
    ) -> None:
        """
        Invoked for each call instruction to record a result value in a basic block.
        """
        pass

    def _on_rt_bool_record_output(
        self, call: Call, value: Value, target: Value
    ) -> None:
        """
        Invoked for each call instruction to record a boolean value in a basic block.
        """
        pass

    def _on_rt_int_record_output(self, call: Call, value: Value, target: Value) -> None:
        """
        Invoked for each call instruction to record an integer value in a basic block.
        """
        pass

    def _on_rt_double_record_output(
        self, call: Call, value: Value, target: Value
    ) -> None:
        """
        Invoked for each call instruction to record a double value in a basic block.
        """
        pass

    def _on_rt_tuple_record_output(
        self, call: Call, value: Value, target: Value
    ) -> None:
        """
        Invoked for each call instruction to record a tuple in a basic block.
        """
        pass

    def _on_rt_array_record_output(
        self, call: Call, value: Value, target: Value
    ) -> None:
        """
        Invoked for each call instruction to record an array in a basic block.
        """
        pass

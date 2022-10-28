# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pathlib import Path
from pyqir.generator import (
    BasicBlock,
    Call,
    Function,
    IntConstant,
    IntType,
    Module,
    Opcode,
    constant_bytes,
    is_entry_point,
    is_interop_friendly,
    qubit_id,
    result_id,
)
import pytest


def test_parser() -> None:
    bitcode = Path("../pyqir-parser/tests/teleportchain.baseprofile.bc").read_bytes()
    mod = Module.from_bitcode(bitcode)
    assert len(mod.functions) == 21

    func_name = (
        "TeleportChain__DemonstrateTeleportationUsingPresharedEntanglement__Interop"
    )
    func = next(filter(lambda f: f.name == func_name, mod.functions))
    interop_funcs = list(filter(is_interop_friendly, mod.functions))
    assert len(interop_funcs) == 1

    blocks = func.basic_blocks
    assert len(blocks) == 9
    assert blocks[0].name == "entry"

    term = blocks[0].terminator
    assert term is not None
    assert term.opcode == Opcode.BR
    assert term.operands[0].name == ""
    assert term.successors[0].name == "continue__1.i.i.i"
    assert term.successors[1].name == "then0__1.i.i.i"

    term = blocks[1].terminator
    assert term is not None
    assert term.opcode == Opcode.BR
    assert len(term.operands) == 1
    assert term.successors[0].name == "continue__1.i.i.i"

    term = blocks[8].terminator
    assert term is not None
    assert term.opcode == Opcode.RET

    term_type = term.operands[0].type
    assert isinstance(term_type, IntType)
    assert term_type.width == 8

    block = next(filter(lambda b: b.name == "then0__2.i.i3.i", func.basic_blocks))
    assert isinstance(block.instructions[0], Call)
    assert qubit_id(block.instructions[0].operands[0]) == 5

    block = next(filter(lambda b: b.name == "continue__1.i.i2.i", func.basic_blocks))
    term = block.terminator
    assert term is not None
    assert term.opcode == Opcode.BR
    assert len(term.operands) == 3

    inst = term.operands[0]
    assert isinstance(inst, Call)
    assert inst.callee.name == "__quantum__qir__read_result"
    assert result_id(inst.args[0]) == 3

    inst_type = inst.type
    assert isinstance(inst_type, IntType)
    assert inst_type.width == 1

    exit = func.basic_blocks[-1]
    call_mz = exit.instructions[0]
    assert isinstance(call_mz, Call)
    assert call_mz.callee.name == "__quantum__qis__mz__body"
    assert call_mz.type.is_void
    assert call_mz.name == ""


def test_parser_select_support() -> None:
    bitcode = Path("../pyqir-parser/tests/select.bc").read_bytes()
    mod = Module.from_bitcode(bitcode)
    func = next(filter(is_entry_point, mod.functions))
    block = func.basic_blocks[0]
    select = block.instructions[5]
    assert select.opcode == Opcode.SELECT
    assert select.name == "spec.select"

    cond = select.operands[0]
    assert isinstance(cond, Call)
    assert cond.callee.name == "__quantum__qis__read_result__body"
    assert result_id(cond.args[0]) == 0

    true = select.operands[1]
    assert isinstance(true, IntConstant)
    assert true.value == 2
    assert true.type.width == 64

    false = select.operands[2]
    assert isinstance(false, IntConstant)
    assert false.value == 0
    assert false.type.width == 64

    select2 = block.instructions[9]
    assert select2.opcode == Opcode.SELECT
    assert select2.name == "val.i.2"
    assert select2.operands[1].name == "spec.select"
    assert select2.operands[2].name == "val.i.1"


def test_global_string() -> None:
    bitcode = Path("../pyqir-parser/tests/hello.bc").read_bytes()
    mod = Module.from_bitcode(bitcode)
    func_name = "program__main__body"
    func = next(filter(lambda f: f.name == func_name, mod.functions))
    assert isinstance(func, Function)
    assert isinstance(func.basic_blocks[0], BasicBlock)
    assert func.basic_blocks[0].name == "entry"

    call = func.basic_blocks[0].instructions[0]
    assert isinstance(call, Call)
    assert call.callee.name == "__quantum__rt__string_create"

    value = constant_bytes(call.args[0])
    assert value is not None
    assert value.decode("utf-8") == "Hello World!\0"


# def test_parser_zext_support() -> None:
#     mod = QirModule("tests/select.bc")
#     func = mod.get_funcs_by_attr("EntryPoint")[0]
#     block = func.blocks[0]
#     instr = block.instructions[7]
#     assert isinstance(instr, QirZExtInstr)
#     assert isinstance(instr.type, QirIntegerType)
#     assert instr.type.width == 64
#     assert instr.output_name == "2"
#     assert len(instr.target_operands) == 1
#     assert isinstance(instr.target_operands[0], QirLocalOperand)
#     assert instr.target_operands[0].name == "1"
#     assert isinstance(instr.target_operands[0].type, QirIntegerType)
#     assert instr.target_operands[0].type.width == 1


# def test_loading_invalid_bitcode() -> None:
#     path = "tests/teleportchain.ll.reference"
#     with pytest.raises(RuntimeError) as exc_info:
#         _ = module_from_bitcode(path)
#     assert str(exc_info.value).lower() == "invalid bitcode signature"


# def test_loading_bad_bitcode_file_path() -> None:
#     path = "tests/does_not_exist.bc"
#     with pytest.raises(RuntimeError) as exc_info:
#         module_from_bitcode(path)
#     assert str(exc_info.value).lower() == "no such file or directory"


# def test_parser_internals() -> None:
#     mod = module_from_bitcode("tests/teleportchain.baseprofile.bc")
#     func_name = (
#         "TeleportChain__DemonstrateTeleportationUsingPresharedEntanglement__Interop"
#     )
#     func = mod.get_func_by_name(func_name)
#     assert func is not None
#     assert func.name == func_name
#     assert len(func.parameters) == 0
#     assert func.return_type.is_integer
#     func_list = mod.functions
#     assert len(func_list) == 1
#     assert func_list[0].name == func_name
#     interop_funcs = mod.get_funcs_by_attr("InteropFriendly")
#     assert len(interop_funcs) == 1
#     assert interop_funcs[0].name == func_name
#     assert interop_funcs[0].get_attribute_value("requiredQubits") == "6"
#     assert interop_funcs[0].required_qubits == 6
#     blocks = func.blocks
#     assert len(blocks) == 9
#     assert blocks[0].name == "entry"
#     entry_block = func.get_block_by_name("entry")
#     assert entry_block is not None
#     assert entry_block.name == "entry"
#     assert entry_block.terminator.is_condbr
#     assert not entry_block.terminator.is_ret
#     assert entry_block.terminator.condbr_true_dest == "then0__1.i.i.i"
#     assert entry_block.terminator.condbr_false_dest == "continue__1.i.i.i"
#     assert blocks[1].terminator.is_br
#     assert blocks[1].terminator.br_dest == "continue__1.i.i.i"
#     assert blocks[8].terminator.is_ret
#     assert len(entry_block.instructions) == 11
#     assert entry_block.instructions[0].is_call
#     assert entry_block.instructions[0].call_func_name == "__quantum__qis__h__body"
#     assert entry_block.instructions[0].is_qis_call
#     param_list = entry_block.instructions[0].call_func_params
#     assert param_list is not None
#     assert len(param_list) == 1
#     assert param_list[0].is_constant
#     assert param_list[0].constant is not None
#     assert param_list[0].constant.is_qubit
#     assert param_list[0].constant.qubit_static_id == 0
#     assert entry_block.instructions[8].is_qis_call
#     assert entry_block.instructions[8].call_func_name == "__quantum__qis__mz__body"
#     assert entry_block.instructions[8].call_func_params is not None
#     assert entry_block.instructions[8].call_func_params[0].constant is not None
#     assert entry_block.instructions[8].call_func_params[0].constant.qubit_static_id == 1
#     assert entry_block.instructions[8].call_func_params[1].constant is not None
#     assert (
#         entry_block.instructions[8].call_func_params[1].constant.result_static_id == 0
#     )
#     branch_cond = entry_block.terminator.condbr_condition
#     assert branch_cond is not None
#     assert branch_cond.local_name == "0"
#     assert entry_block.instructions[10].is_qir_call
#     assert entry_block.instructions[10].call_func_name == "__quantum__qir__read_result"
#     assert entry_block.instructions[10].call_func_params is not None
#     assert entry_block.instructions[10].call_func_params[0].constant is not None
#     assert (
#         entry_block.instructions[10].call_func_params[0].constant.result_static_id == 0
#     )
#     assert entry_block.instructions[10].has_output
#     assert entry_block.instructions[10].output_name == "0"
#     source_instr = func.get_instruction_by_output_name(branch_cond.local_name)
#     assert source_instr is not None
#     assert source_instr.call_func_params is not None
#     assert source_instr.call_func_params[0].constant is not None
#     assert source_instr.call_func_params[0].constant.result_static_id == 0

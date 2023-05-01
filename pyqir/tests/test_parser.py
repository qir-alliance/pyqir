# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pathlib import Path

import pytest

from pyqir import (
    BasicBlock,
    Call,
    Constant,
    Context,
    Function,
    IntConstant,
    IntType,
    Module,
    Opcode,
    extract_byte_string,
    is_entry_point,
    is_interop_friendly,
    is_qubit_type,
    qubit_id,
    required_num_qubits,
    result_id,
)


def test_parser() -> None:
    bitcode = Path("tests/teleportchain.baseprofile.bc").read_bytes()
    mod = Module.from_bitcode(Context(), bitcode)
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
    bitcode = Path("tests/select.bc").read_bytes()
    mod = Module.from_bitcode(Context(), bitcode)
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
    bitcode = Path("tests/hello.bc").read_bytes()
    mod = Module.from_bitcode(Context(), bitcode)
    func_name = "program__main__body"
    func = next(filter(lambda f: f.name == func_name, mod.functions))
    assert isinstance(func, Function)
    assert isinstance(func.basic_blocks[0], BasicBlock)
    assert func.basic_blocks[0].name == "entry"

    call = func.basic_blocks[0].instructions[0]
    assert isinstance(call, Call)
    assert call.callee.name == "__quantum__rt__string_create"

    value = extract_byte_string(call.args[0])
    assert value is not None
    assert value.decode("utf-8") == "Hello World!\0"


def test_null_i8ptr_string() -> None:
    llvm_ir = """
    define void @main() {
      call void @a(i8* null)
      ret void
    }
    declare void @a(i8*)
    """

    module = Module.from_ir(Context(), llvm_ir, "module")
    null_ptr_value = module.functions[0].basic_blocks[0].instructions[0].operands[0]
    string = extract_byte_string(null_ptr_value)
    assert string is None


def test_parser_zext_support() -> None:
    bitcode = Path("tests/select.bc").read_bytes()
    mod = Module.from_bitcode(Context(), bitcode)
    func = next(filter(is_entry_point, mod.functions))
    block = func.basic_blocks[0]
    inst = block.instructions[7]
    assert inst.opcode == Opcode.ZEXT

    ty = inst.type
    assert isinstance(ty, IntType)
    assert ty.width == 64
    assert inst.name == ""
    assert len(inst.operands) == 1

    operand = inst.operands[0]
    assert operand.name == ""

    operand_ty = operand.type
    assert isinstance(operand_ty, IntType)
    assert operand_ty.width == 1


def test_loading_invalid_bitcode() -> None:
    bitcode = Path("tests/teleportchain.ll.reference").read_bytes()
    with pytest.raises(ValueError) as e:
        Module.from_bitcode(Context(), bitcode)
    assert e.value.args[0] == "Invalid bitcode signature"


def test_parser_internals() -> None:
    bitcode = Path("tests/teleportchain.baseprofile.bc").read_bytes()
    mod = Module.from_bitcode(Context(), bitcode)
    func_name = (
        "TeleportChain__DemonstrateTeleportationUsingPresharedEntanglement__Interop"
    )
    func = next(filter(lambda f: f.name == func_name, mod.functions))
    assert len(func.params) == 0
    assert isinstance(func.type.ret, IntType)

    func_list = mod.functions
    assert len(func_list) == 21
    assert func_list[0].name == func_name

    interop_funcs = list(filter(is_interop_friendly, mod.functions))
    assert len(interop_funcs) == 1
    assert interop_funcs[0].name == func_name
    assert required_num_qubits(interop_funcs[0]) == 6

    attribute = interop_funcs[0].attributes.func["requiredQubits"]
    assert attribute.string_value == "6"

    blocks = func.basic_blocks
    assert len(blocks) == 9

    entry_block = blocks[0]
    assert entry_block.name == "entry"
    assert len(entry_block.instructions) == 12

    term = entry_block.terminator
    assert term is not None
    assert term.opcode == Opcode.BR
    assert term.operands[2].name == "then0__1.i.i.i"
    assert term.operands[1].name == "continue__1.i.i.i"

    term = blocks[1].terminator
    assert term is not None
    assert term.opcode == Opcode.BR
    assert isinstance(term.operands[0], BasicBlock)
    assert term.operands[0].name == "continue__1.i.i.i"

    term = blocks[8].terminator
    assert term is not None
    assert term.opcode == Opcode.RET

    call = entry_block.instructions[0]
    assert isinstance(call, Call)
    assert call.callee.name == "__quantum__qis__h__body"

    args = call.args
    assert len(args) == 1

    arg = args[0]
    assert isinstance(arg, Constant)
    assert is_qubit_type(arg.type)
    assert qubit_id(arg) == 0

    call = entry_block.instructions[8]
    assert isinstance(call, Call)
    assert call.callee.name == "__quantum__qis__mz__body"

    arg = call.args[0]
    assert isinstance(arg, Constant)
    assert qubit_id(arg) == 1

    arg = call.args[1]
    assert isinstance(arg, Constant)
    assert result_id(arg) == 0

    term = entry_block.terminator
    assert term is not None
    assert term.opcode == Opcode.BR

    branch_cond = term.operands[0]
    assert isinstance(branch_cond, Call)

    arg = branch_cond.args[0]
    assert isinstance(arg, Constant)
    assert result_id(arg) == 0
    assert branch_cond.name == ""

    call = entry_block.instructions[10]
    assert isinstance(call, Call)
    assert call.callee.name == "__quantum__qir__read_result"

    arg = call.args[0]
    assert isinstance(arg, Constant)
    assert result_id(arg) == 0

    assert not entry_block.instructions[10].type.is_void
    assert entry_block.instructions[10].name == ""


def test_attribute_values() -> None:
    ir = Path("tests/attributes.ll").read_text()
    module = Module.from_ir(Context(), ir)
    attributes = module.functions[0].attributes
    assert attributes.ret["ret_attr"].string_value == "ret value"
    assert attributes.param(0)["param0_attr"].string_value == "param0 value"
    assert attributes.param(2)["param2_attr"].string_value == "param2 value"
    assert attributes.func["fn_attr"].string_value == "fn value"


def test_contains_attribute() -> None:
    ir = Path("tests/attributes.ll").read_text()
    module = Module.from_ir(Context(), ir)
    attributes = module.functions[0].attributes
    assert "ret_attr" in attributes.ret
    assert attributes.ret["ret_attr"] is not None


def test_not_contains_attribute() -> None:
    ir = Path("tests/attributes.ll").read_text()
    module = Module.from_ir(Context(), ir)
    attributes = module.functions[0].attributes
    assert "foo" not in attributes.ret
    with pytest.raises(KeyError):
        attributes.ret["foo"]

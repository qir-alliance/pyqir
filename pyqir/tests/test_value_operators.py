# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List, Optional, Tuple

from pyqir import (
    Context,
    Instruction,
    Module,
    Value,
    Opcode,
    Linkage,
    FloatPredicate,
    IntPredicate
)
import pytest


def get_module() -> Module:
    llvm_ir = """
    define void @program_main() {
    entry:
    %0 = add i64 1, 2
    %1 = mul i64 %0, %0
    ret void
    }
    """

    mod = Module.from_ir(Context(), llvm_ir, "module")
    return mod

def get_opcodes(module: Optional[Module] = None) -> List[Opcode]:
    mod: Module = module if module else get_module()
    return map(lambda x : x.opcode, mod.functions[0].basic_blocks[0].instructions)


def get_instructions(module: Optional[Module] = None) -> List[Instruction]:
    mod: Module = module if module else get_module()
    return mod.functions[0].basic_blocks[0].instructions


def get_first_instruction_operands() -> Tuple[Value, Value]:
    insts = get_instructions()
    operand10: Value = insts[1].operands[0]
    operand11: Value = insts[1].operands[1]
    return (operand10, operand11)


def test_instruction_equals_instruction() -> None:
    inst = get_instructions()[0]
    assert inst == inst


def test_instruction__eq__instruction() -> None:
    inst = get_instructions()[0]
    assert inst.__eq__(inst)


def test_instruction_is_self_instruction() -> None:
    inst = get_instructions()[0]
    assert inst is inst


def test_instruction_not_equals_instruction() -> None:
    insts = get_instructions()
    assert insts[0] != insts[1]


def test_instruction__ne__instruction() -> None:
    insts = get_instructions()
    assert insts[0].__ne__(insts[1])


def test_instruction_is_not_other_instruction() -> None:
    insts = get_instructions()
    assert insts[0] is not insts[1]


def test_instruction_hash_equals_self_hash() -> None:
    inst = get_instructions()[0]
    assert hash(inst) == hash(inst)


def test_value_equals_value() -> None:
    ops = get_first_instruction_operands()
    assert ops[0] == ops[1]


def test_value__eq__value() -> None:
    ops = get_first_instruction_operands()
    assert ops[0].__eq__(ops[1])


def test_value_equal_operands_are_not_same_value() -> None:
    ops = get_first_instruction_operands()
    assert ops[0] is not ops[1]


def test_value_hash_equals_value() -> None:
    ops = get_first_instruction_operands()
    assert hash(ops[0]) == hash(ops[1])


def test_operand_equals_source_instruction() -> None:
    insts = get_instructions()
    assert insts[1].operands[0] == insts[0]


def test_operand_is_not_source_instruction() -> None:
    insts = get_instructions()
    assert insts[1].operands[0] is not insts[0]


def test_operand_hash_equals_source_instruction_hash() -> None:
    insts = get_instructions()
    assert hash(insts[1].operands[0]) == hash(insts[0])


def test_hash_and_equality_are_not_stable_across_module_instances() -> None:
    insts0 = get_instructions()
    insts1 = get_instructions()

    for i0, i1 in zip(insts0, insts1):
        assert str(i0) == str(i1)
        assert not i0 == i1
        assert not i0.__eq__(i1)
        assert not hash(i0) == hash(i1)

        for o0, o1 in zip(i0.operands, i1.operands):
            assert str(o0) == str(o1)
            assert not o0 == o1
            assert not o0.__eq__(o1)
            assert not hash(o0) == hash(o1)


def test_lt_op_not_supported_on_value() -> None:
    (o0, o1) = get_first_instruction_operands()
    with pytest.raises(TypeError):
        o0 < o1  # type: ignore


def test_le_op_not_supported_on_value() -> None:
    (o0, o1) = get_first_instruction_operands()
    with pytest.raises(TypeError):
        o0 <= o1  # type: ignore


def test_gt_op_not_supported_on_value() -> None:
    (o0, o1) = get_first_instruction_operands()
    with pytest.raises(TypeError):
        o0 > o1  # type: ignore


def test_ge_op_not_supported_on_value() -> None:
    (o0, o1) = get_first_instruction_operands()
    with pytest.raises(TypeError):
        o0 >= o1  # type: ignore


def test_instruction_equals_instruction_when_from_same_module() -> None:
    mod = get_module()
    first = get_instructions(mod)[1]
    second = get_instructions(mod)[1]
    assert first == second


def test_instruction_is_not_same_instruction_when_from_same_module() -> None:
    mod = get_module()
    first = get_instructions(mod)[1]
    second = get_instructions(mod)[1]
    assert first is not second


def test_instruction_hash_is_same_when_from_same_module() -> None:
    mod = get_module()
    first = get_instructions(mod)[1]
    second = get_instructions(mod)[1]
    assert hash(first) == hash(second)


def test_opcode_equals_opcode() -> None:
    inst = get_opcodes()[0]
    assert inst == inst


def test_opcode__eq__opcode() -> None:
    inst = get_opcodes()[0]
    assert inst.__eq__(inst)


def test_opcode_is_self_opcode() -> None:
    inst = get_opcodes()[0]
    assert inst is inst


def test_opcode_not_equals_opcode() -> None:
    insts = get_opcodes()
    assert insts[0] != insts[1]


def test_opcode__ne__opcode() -> None:
    insts = get_opcodes()
    assert insts[0].__ne__(insts[1])


def test_opcode_is_not_other_opcode() -> None:
    insts = get_opcodes()
    assert insts[0] is not insts[1]


def test_opcode_hash_equals_self_hash() -> None:
    inst = get_opcodes()[0]
    assert hash(inst) == hash(inst)
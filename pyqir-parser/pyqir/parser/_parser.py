# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pyqir.parser._native as native
from pyqir.parser._native import module_from_bitcode
from typing import List, Optional, Tuple

__all__ = [
    "QirType",
    "QirVoidType",
    "QirIntegerType",
    "QirPointerType",
    "QirDoubleType",
    "QirArrayType",
    "QirStructType",
    "QirNamedStructType",
    "QirQubitType",
    "QirResultType",
    "QirOperand",
    "QirLocalOperand",
    "QirConstant",
    "QirIntConstant",
    "QirDoubleConstant",
    "QirNullConstant",
    "QirQubitConstant",
    "QirResultConstant",
    "QirGlobalByteArrayConstant",
    "QirTerminator",
    "QirRetTerminator",
    "QirBrTerminator",
    "QirCondBrTerminator",
    "QirSwitchTerminator",
    "QirUnreachableTerminator",
    "QirInstr",
    "QirOpInstr",
    "QirAddInstr",
    "QirSubInstr",
    "QirMulInstr",
    "QirUDivInstr",
    "QirSDivInstr",
    "QirURemInstr",
    "QirSRemInstr",
    "QirAndInstr",
    "QirOrInstr",
    "QirXorInstr",
    "QirShlInstr",
    "QirLShrInstr",
    "QirAShrInstr",
    "QirFAddInstr",
    "QirFSubInstr",
    "QirFMulInstr",
    "QirFDivInstr",
    "QirFRemInstr",
    "QirFNegInstr",
    "QirICmpInstr",
    "QirFCmpInstr",
    "QirZExtInstr",
    "QirSelectInstr",
    "QirPhiInstr",
    "QirCallInstr",
    "QirQisCallInstr",
    "QirRtCallInstr",
    "QirQirCallInstr",
    "QirBlock",
    "QirParameter",
    "QirFunction",
    "QirModule",
    "module_from_bitcode",
]


class QirType:
    """
    Instances of QirType represent a type description in QIR. Specific subclasses may contain
    additional properties of that type.
    """

    def __new__(cls, ty: native.PyQirType) -> "QirType":
        if ty.is_qubit:
            return super().__new__(QirQubitType)
        elif ty.is_result:
            return super().__new__(QirResultType)
        elif ty.is_void:
            return super().__new__(QirVoidType)
        elif ty.is_integer:
            return super().__new__(QirIntegerType)
        elif ty.is_pointer:
            return super().__new__(QirPointerType)
        elif ty.is_double:
            return super().__new__(QirDoubleType)
        elif ty.is_array:
            return super().__new__(QirArrayType)
        elif ty.is_struct:
            return super().__new__(QirStructType)
        elif ty.is_named_struct:
            return super().__new__(QirNamedStructType)
        else:
            return super().__new__(cls)

    def __init__(self, ty: native.PyQirType):
        self.ty = ty


class QirVoidType(QirType):
    """
    Instances of QirVoidType represent a void type in QIR.
    """

    pass


class QirIntegerType(QirType):
    """
    Instances of QirIntegerType represent a signed integer in QIR. Note that there is no unsigned
    integer type, just unsigned arithmetic instructions.
    """

    @property
    def width(self) -> int:
        """
        Gets the bit width of this integer type.
        """
        width = self.ty.integer_width
        assert width is not None
        return width


class QirPointerType(QirType):
    """
    Instances of QirPointerType represent a pointer to some other type in QIR.
    """

    @property
    def type(self) -> QirType:
        """
        Gets the QirType this to which this pointer points.
        """
        if not hasattr(self, "_type"):
            ty = self.ty.pointer_type
            assert ty is not None
            self._type = QirType(ty)
        return self._type

    @property
    def addrspace(self) -> Optional[int]:
        """
        Gets the address space to which this pointer points.
        """
        return self.ty.pointer_addrspace


class QirDoubleType(QirType):
    """
    Instances of QirDoubleType represent the double-sized floating point type in a QIR program.
    """

    pass


class QirArrayType(QirType):
    """
    Instances of the QirArrayType represent the native LLVM fixed-length array type in a QIR program.
    """

    @property
    def element_type(self) -> QirType:
        """
        Gets the QirType representing the underlying array type.
        """
        if not hasattr(self, "_element_type"):
            element_type = self.ty.array_element_type
            assert element_type is not None
            self._element_type = QirType(element_type)
        return self._element_type

    @property
    def element_count(self) -> int:
        """
        Gets the count of elements in the array.
        """
        element_count = self.ty.array_num_elements
        assert element_count is not None
        return element_count


class QirStructType(QirType):
    """
    Instances of QirStructType represent an anonymous struct with inline defined types in QIR.
    """

    @property
    def struct_element_types(self) -> List[QirType]:
        """
        Gets the ordered list of QirTypes representing the underlying struct types.
        """
        if not hasattr(self, "_struct_element_types"):
            element_types = self.ty.struct_element_types
            assert element_types is not None
            self._struct_element_types = list(map(QirType, element_types))
        return self._struct_element_types


class QirNamedStructType(QirType):
    """
    Instances of QirNamedStruct represent a globally defined struct, often used to represent opaque
    poitners.
    """

    @property
    def name(self) -> str:
        """
        Gets the name of this struct.
        """
        name = self.ty.named_struct_name
        assert name is not None
        return name


class QirQubitType(QirNamedStructType):
    """
    Instances of QirQubitType are specific QIR opaque pointer corresponding to the Qubit special
    type.
    """

    pass


class QirResultType(QirNamedStructType):
    """
    Instances of QirResultType are specific QIR opaque pointer corresponding to the Result special
    type.
    """

    pass


class QirOperand:
    """
    Instances of QirOperand represent an instance in a QIR program, either a local operand (variable)
    or constant.
    """

    def __new__(cls, op: native.PyQirOperand) -> "QirOperand":
        if op.is_local:
            return super().__new__(QirLocalOperand)
        elif op.is_constant:
            constant = op.constant
            assert constant is not None
            if constant.is_qubit:
                return super().__new__(QirQubitConstant)
            elif constant.is_result:
                return super().__new__(QirResultConstant)
            elif constant.is_int:
                return super().__new__(QirIntConstant)
            elif constant.is_float:
                return super().__new__(QirDoubleConstant)
            elif constant.is_null:
                return super().__new__(QirNullConstant)
            elif constant.is_global_byte_array:
                return super().__new__(QirGlobalByteArrayConstant)
            else:
                return super().__new__(cls)
        else:
            return super().__new__(cls)

    def __init__(self, op: native.PyQirOperand):
        self.op = op
        self.const = op.constant


class QirLocalOperand(QirOperand):
    """
    Instances of QirLocalOperand represent a typed local variable in a QIR program.
    """

    @property
    def name(self) -> str:
        """
        Gets the name identifier for this operand. This could be an identifier from the original
        source language, a generated name based on an identifier, or a generated integer name.
        """
        name = self.op.local_name
        assert name is not None
        return name

    @property
    def type(self) -> QirType:
        """
        Gets the QirType instance representing the type for this operand.
        """
        if not hasattr(self, "_type"):
            ty = self.op.local_type
            assert ty is not None
            self._type = QirType(ty)
        return self._type


class QirConstant(QirOperand):
    """
    Instances of QirConstant represent a constant value in a QIR program.
    """

    @property
    def type(self) -> QirType:
        """
        Gets the QirType instance representing the type of this constant.
        """
        if not hasattr(self, "_type"):
            const = self.const
            assert const is not None
            self._type = QirType(const.type)
        return self._type


class QirIntConstant(QirConstant):
    """
    Instances of QirIntConstant represent a constant integer value in a QIR program.
    """

    @property
    def value(self) -> int:
        """
        Gets the integer value for this constant.
        """
        const = self.const
        assert const is not None
        int_value = const.int_value
        assert int_value is not None
        return int_value

    @property
    def width(self) -> int:
        """
        Gets the bit width for this integer constant.
        """
        const = self.const
        assert const is not None
        int_width = const.int_width
        assert int_width is not None
        return int_width


class QirDoubleConstant(QirConstant):
    """
    Instances of QirDoubleConstant represent a constant double-sized float value in a QIR program.
    """

    @property
    def value(self) -> float:
        """
        Gets the double-sized float value for this constant.
        """
        const = self.const
        assert const is not None
        value = const.float_double_value
        assert value is not None
        return value


class QirNullConstant(QirConstant):
    """
    Instances of QirNullConstant represent a constant null pointer in a QIR program. Use the type
    property to inspect which pointer type this null represents.
    """

    @property
    def value(self) -> None:
        """
        The value of QirNullConstant instances is always None.
        """
        return None


class QirQubitConstant(QirConstant):
    """
    Instances of QirQubitConstant represent a statically allocated qubit id in a QIR program.
    """

    @property
    def value(self) -> int:
        """
        Gets the integer identifier for this qubit constant.
        """
        const = self.const
        assert const is not None
        value = const.qubit_static_id
        assert value is not None
        return value

    @property
    def id(self) -> int:
        """
        Gets the integer identifier for this qubit constant.
        """
        return self.value


class QirResultConstant(QirConstant):
    """
    Instances of QirResultConstant represent a statically allocated result id in a QIR program.
    """

    @property
    def value(self) -> int:
        """
        Gets the integer identifier for the is result constant.
        """
        const = self.const
        assert const is not None
        value = const.result_static_id
        assert value is not None
        return value

    @property
    def id(self) -> int:
        """
        gets the integer identifier for this result constant.
        """
        return self.value


class QirGlobalByteArrayConstant(QirConstant):
    """
    Instances of QirGlobalByteArrayConstant represent a globally defined array of bytes in a QIR program.
    """

    pass


class QirTerminator:
    """
    Instances of QirTerminator represent the special final instruction at the end of a block that
    indicates how control flow should transfer.
    """

    def __new__(cls, term: native.PyQirTerminator) -> "QirTerminator":
        if term.is_ret:
            return super().__new__(QirRetTerminator)
        elif term.is_br:
            return super().__new__(QirBrTerminator)
        elif term.is_condbr:
            return super().__new__(QirCondBrTerminator)
        elif term.is_switch:
            return super().__new__(QirSwitchTerminator)
        elif term.is_unreachable:
            return super().__new__(QirUnreachableTerminator)
        else:
            return super().__new__(cls)

    def __init__(self, term: native.PyQirTerminator) -> None:
        self.term = term


class QirRetTerminator(QirTerminator):
    """
    Instances of QirRetTerminator represent the ret instruction in a QIR program.
    """

    @property
    def operand(self) -> Optional[QirOperand]:
        """
        Gets the operand that will be returned by the ret instruction or None for a void return.
        """
        if not hasattr(self, "_operand"):
            self._operand = (
                None
                if self.term.ret_operand is None
                else QirOperand(self.term.ret_operand)
            )
        return self._operand


class QirBrTerminator(QirTerminator):
    """
    Instances of QirBrTerminator represent a branch terminator instruction that unconditionally
    jumps execution to the named destination block.
    """

    @property
    def dest(self) -> str:
        """
        Gets the name of the block this branch jumps to.
        """
        dest = self.term.br_dest
        assert dest is not None
        return dest


class QirCondBrTerminator(QirTerminator):
    """
    Instances of QirCondBrTerminator represent a conditional branch terminator instruction that
    decides which named block to jump to based on an given operand.
    """

    @property
    def condition(self) -> QirOperand:
        """
        Gets the QirOperand representing the condition used to determine the block to jump to.
        """
        if not hasattr(self, "_condition"):
            condition = self.term.condbr_condition
            assert condition is not None
            self._condition = QirOperand(condition)
        return self._condition

    @property
    def true_dest(self) -> str:
        """
        Gets the name of the block that will be jumped to if the condition evaluates to true.
        """
        true_dest = self.term.condbr_true_dest
        assert true_dest is not None
        return true_dest

    @property
    def false_dest(self) -> str:
        """
        Gets the name of the block that will be jumped to if the condition evaluates to false.
        """
        false_dest = self.term.condbr_false_dest
        assert false_dest is not None
        return false_dest


class QirSwitchTerminator(QirTerminator):
    """
    Instances of QirSwitchTerminator represent a switch terminator instruction that can jump
    to one or more blocks based on matching values of a given operand, or jump to a fallback block
    in the case that no matches are found.
    """

    @property
    def operand(self) -> QirLocalOperand:
        """
        Gets the operand variable of the switch statement.
        """
        if not hasattr(self, "_operand"):
            operand = self.term.switch_operand
            assert operand is not None
            self._operand = QirLocalOperand(operand)
        return self._operand

    @property
    def dest_pairs(self) -> List[Tuple[QirConstant, str]]:
        """
        Gets a list of pairs representing the constant values to compare the operand against and the
        matching block name to jump to if the comparison succeeds.
        """
        if not hasattr(self, "_dest_pairs"):
            dest_pairs = self.term.switch_dests
            assert dest_pairs is not None
            self._dest_pairs = [(QirConstant(p[0]), p[1]) for p in dest_pairs]
        return self._dest_pairs

    @property
    def default_dest(self) -> str:
        """
        Gets the name of the default block that the switch will jump to if no values match the given
        operand.
        """
        default_dest = self.term.switch_default_dest
        assert default_dest is not None
        return default_dest


class QirUnreachableTerminator(QirTerminator):
    """
    Instances of QirUnreachableTerminator represent an unreachable terminator instruction. As the name
    implies, this terminator is not expected to be reached such that some instruction in the block
    before this terminator should halt program execution.
    """

    pass


class QirInstr:
    """
    Instances of QirInstr represent an instruction within a block of a QIR program. See the subclasses
    of this type for specifically supported instructions.
    """

    def __new__(cls, instr: native.PyQirInstruction) -> "QirInstr":
        if instr.is_qis_call:
            return super().__new__(QirQisCallInstr)
        elif instr.is_rt_call:
            return super().__new__(QirRtCallInstr)
        elif instr.is_qir_call:
            return super().__new__(QirQirCallInstr)
        elif instr.is_call:
            return super().__new__(QirCallInstr)
        elif instr.is_add:
            return super().__new__(QirAddInstr)
        elif instr.is_sub:
            return super().__new__(QirSubInstr)
        elif instr.is_mul:
            return super().__new__(QirMulInstr)
        elif instr.is_udiv:
            return super().__new__(QirUDivInstr)
        elif instr.is_sdiv:
            return super().__new__(QirSDivInstr)
        elif instr.is_urem:
            return super().__new__(QirURemInstr)
        elif instr.is_srem:
            return super().__new__(QirSRemInstr)
        elif instr.is_and:
            return super().__new__(QirAndInstr)
        elif instr.is_or:
            return super().__new__(QirOrInstr)
        elif instr.is_xor:
            return super().__new__(QirXorInstr)
        elif instr.is_shl:
            return super().__new__(QirShlInstr)
        elif instr.is_lshr:
            return super().__new__(QirLShrInstr)
        elif instr.is_ashr:
            return super().__new__(QirAShrInstr)
        elif instr.is_fadd:
            return super().__new__(QirFAddInstr)
        elif instr.is_fsub:
            return super().__new__(QirFSubInstr)
        elif instr.is_fmul:
            return super().__new__(QirFMulInstr)
        elif instr.is_fdiv:
            return super().__new__(QirFDivInstr)
        elif instr.is_frem:
            return super().__new__(QirFRemInstr)
        elif instr.is_fneg:
            return super().__new__(QirFNegInstr)
        elif instr.is_icmp:
            return super().__new__(QirICmpInstr)
        elif instr.is_fcmp:
            return super().__new__(QirFCmpInstr)
        elif instr.is_phi:
            return super().__new__(QirPhiInstr)
        elif instr.is_select:
            return super().__new__(QirSelectInstr)
        elif instr.is_zext:
            return super().__new__(QirZExtInstr)
        else:
            return super().__new__(cls)

    def __init__(self, instr: native.PyQirInstruction):
        self.instr = instr
        self._type: Optional[QirType] = None

    @property
    def output_name(self) -> Optional[str]:
        """
        Gets the name of the local operand that receives the output of this instruction, or
        None if the instruction does not return a value.
        """
        output_name = self.instr.output_name
        assert output_name is not None
        return output_name

    @property
    def type(self) -> QirType:
        """
        Gets the QirType instance representing the output of this instruction. If the instruction
        has no output, the type will be an instance of QirVoidType.
        """
        if self._type is None:
            self._type = QirType(self.instr.type)
        return self._type


class QirOpInstr(QirInstr):
    """
    Instances of QirOpInstr represent the class of instructions that have one or more operands that
    they operate on.
    """

    @property
    def target_operands(self) -> List[QirOperand]:
        """
        Gets the list of operands that this instruction operates on.
        """
        if not hasattr(self, "_target_operands"):
            self._target_operands = list(map(QirOperand, self.instr.target_operands))
        return self._target_operands


class QirAddInstr(QirOpInstr):
    """
    Instances of QirAddInstr represent an integer addition instruction that takes two operands.
    """

    pass


class QirSubInstr(QirOpInstr):
    """
    Instances of QirSubInstr represent an integer subtraction instruction that takes two operands.
    """

    pass


class QirMulInstr(QirOpInstr):
    """
    Instances of QirMulInstr represent an integer multiplication instruction that takes two operands.
    """

    pass


class QirUDivInstr(QirOpInstr):
    """
    Instances of QirUDivInstr represent an unsigned integer division instruction that takes two operands.
    """

    pass


class QirSDivInstr(QirOpInstr):
    """
    Instances of QirSDivInstr represent a signed integer division instruction that takes two operands.
    """

    pass


class QirURemInstr(QirOpInstr):
    """
    Instances of QirURemInstr represent an unsigned integer remainder instruction that takes two operands.
    """

    pass


class QirSRemInstr(QirOpInstr):
    """
    Instances of QirSRemInstr represent a signed integer remainder instruction that takes two operands.
    """

    pass


class QirAndInstr(QirOpInstr):
    """
    Instances of QirAndInstr represent a boolean and instruction that takes two operands.
    """

    pass


class QirOrInstr(QirOpInstr):
    """
    Instances of QirOrInstr represent a boolean or instruction that takes two operands.
    """

    pass


class QirXorInstr(QirOpInstr):
    """
    Instances of QirXorInstr represent a boolean xor instruction that takes two operands.
    """

    pass


class QirShlInstr(QirOpInstr):
    """
    Instances of QirShlInstr represent a bitwise shift left instruction that takes two operands.
    """

    pass


class QirLShrInstr(QirOpInstr):
    """
    Instances of QirLShrInstr represent a logical bitwise shift right instruction that takes two operands.
    """

    pass


class QirAShrInstr(QirOpInstr):
    """
    Instances of QirAShrInstr represent an arithmetic bitwise shift right instruction that takes two operands.
    """

    pass


class QirFAddInstr(QirOpInstr):
    """
    Instances of QirFAddInstr represent a floating-point addition instruction that takes two operands.
    """

    pass


class QirFSubInstr(QirOpInstr):
    """
    Instances of QirFSubInstr represent a floating-point subtraction instruction that takes two operands.
    """

    pass


class QirFMulInstr(QirOpInstr):
    """
    Instances of QirFMulInstr represent a floating-point multiplication instruction that takes two operands.
    """

    pass


class QirFDivInstr(QirOpInstr):
    """
    Instances of QirFDivInstr represent a floating-point division instruction that takes two operands.
    """

    pass


class QirFRemInstr(QirOpInstr):
    """
    Instances of QirFRemInstr represent a floating-point remainder instruction that takes two operands.
    """

    pass


class QirFNegInstr(QirOpInstr):
    """
    Instances of QirFNegInstr represent a floating-point negation instruction that takes one operand.
    """

    pass


class QirICmpInstr(QirOpInstr):
    """
    Instances of QirICmpInstr represent an integer comparison instruction that takes two operands,
    and uses a specific predicate to output the boolean result of the comparison.
    """

    @property
    def predicate(self) -> str:
        """
        Gets a string representing the predicate operation to perform. Possible values are
        "eq", "ne", "ugt", "uge", "ult", "ule", "sgt", "sge", "slt", and "sle".
        """
        predicate = self.instr.icmp_predicate
        assert predicate is not None
        return predicate


class QirFCmpInstr(QirOpInstr):
    """
    Instances of QirFCmpInstr represent a floating-point comparison instruction that takes two operands,
    and uses a specific predicate to output the boolean result of the comparison.
    """

    @property
    def predicate(self) -> str:
        """
        Gets a string representing the predicate operation to perform. Possible values are
        "false", "oeq", "ogt", "oge", "olt", "ole", "one", "ord", "uno", "ueq", "ugt", "uge", "ult",
        "ule", "une", and "true"
        """
        predicate = self.instr.fcmp_predicate
        assert predicate is not None
        return predicate


class QirZExtInstr(QirOpInstr):
    """
    Instances of QirZExtInstr represent a zero-extension instruction that expands the bitwidth
    of the given integer operand to match the width of the output operand.
    """

    pass


class QirSelectInstr(QirInstr):
    """
    Instances of QirSelectInstr represent a select instruction that chooses a value to output based
    on a boolean operand.
    """

    @property
    def condition(self) -> QirOperand:
        """
        Gets the condition operand that the select instruction will use to choose with result to output.
        """
        if not hasattr(self, "_condition"):
            condition = self.instr.select_condition
            assert condition is not None
            self._condition = QirOperand(condition)
        return self._condition

    @property
    def true_value(self) -> QirOperand:
        """
        Gets the operand that will be the result of the select if the condition is true.
        """
        if not hasattr(self, "_true_value"):
            true_value = self.instr.select_true_value
            assert true_value is not None
            self._true_value = QirOperand(true_value)
        return self._true_value

    @property
    def false_value(self) -> QirOperand:
        """
        Gets the operand that will be the result of the select if the condition is false.
        """
        if not hasattr(self, "_false_value"):
            false_value = self.instr.select_false_value
            assert false_value is not None
            self._false_value = QirOperand(false_value)
        return self._false_value


class QirPhiInstr(QirInstr):
    """
    Instances of QirPhiInstr represent a phi instruction that selects a value for an operand based
    on the name of the block that transferred execution to the current block.
    """

    @property
    def incoming_values(self) -> List[Tuple[QirOperand, str]]:
        """
        Gets a list of all the incoming value pairs for this phi node, where each pair is the QirOperand
        for the value to use and the string name of the originating block.
        """
        if not hasattr(self, "_incoming_values"):
            incoming_values = self.instr.phi_incoming_values
            assert incoming_values is not None
            self._incoming_values = [(QirOperand(v[0]), v[1]) for v in incoming_values]
        return self._incoming_values

    def get_incoming_value_for_name(self, name: str) -> Optional[QirOperand]:
        """
        Gets the QirOperand representing the value for a given originating block, or None if that
        name is not found.
        :param name: the block name to search for.
        """
        op = self.instr.get_phi_incoming_value_for_name(name)
        if isinstance(op, native.PyQirOperand):
            return QirOperand(op)
        else:
            return None


class QirCallInstr(QirInstr):
    """
    Instances of QirCallInstr represent a call instruction in a QIR program.
    """

    @property
    def func_name(self) -> str:
        """
        Gets the name of the function called by this instruction.
        """
        func_name = self.instr.call_func_name
        assert func_name is not None
        return func_name

    @property
    def func_args(self) -> List[QirOperand]:
        """
        Gets the list of QirOperand instances that are passed as arguments to the function call.
        """
        if not hasattr(self, "_func_args"):
            func_args = self.instr.call_func_params
            assert func_args is not None
            self._func_args = list(map(QirOperand, func_args))
        return self._func_args


class QirQisCallInstr(QirCallInstr):
    """
    Instances of QirQisCallInstr represent a call instruction where the function name begins with
    "__quantum__qis__" indicating that it is a function from the QIR quantum intrinsic set.
    """

    pass


class QirRtCallInstr(QirCallInstr):
    """
    Instances of QirRtCallInstr represent a call instruction where the function name begins with
    "__quantum__rt__" indicating that it is a function from the QIR runtime.
    """

    pass


class QirQirCallInstr(QirCallInstr):
    """
    Instances of QirQirCallInstr represent a call instruction where the function name begins with
    "__quantum__qir__" indicating that it is a function from the QIR base profile.
    """

    pass


class QirBlock:
    """
    Instances of the QirBlock type represent a basic block within a function body. Each basic block is
    comprised of a list of instructions executed in sequence and a single, special final instruction
    called a terminator that indicates where execution should jump at the end of the block.
    """

    def __init__(self, block: native.PyQirBasicBlock):
        self.block = block
        self._instructions: Optional[List[QirInstr]] = None
        self._terminator: Optional[QirTerminator] = None
        self._phi_nodes: Optional[List[QirPhiInstr]] = None

    @property
    def name(self) -> str:
        """
        Gets the identifying name for this block. This is unique within a given function and acts
        as a label for any branches that transfer execution to this block.
        """
        return self.block.name

    @property
    def instructions(self) -> List[QirInstr]:
        """
        Gets the list of instructions that make up this block. The list is ordered; instructions are
        executed from first to last unconditionally. This list does not include the special
        terminator instruction (see QirBlock.terminator).
        """
        if self._instructions is None:
            self._instructions = list(map(QirInstr, self.block.instructions))
        return self._instructions

    @property
    def terminator(self) -> QirTerminator:
        """
        Gets the terminator instruction for this block. Every block has exactly one terminator
        and it is the last intruction in the block.
        """
        if self._terminator is None:
            self._terminator = QirTerminator(self.block.terminator)
        return self._terminator

    @property
    def phi_nodes(self) -> List[QirPhiInstr]:
        """
        Gets any phi nodes defined for this block. Phi nodes are a special instruction that defines
        variables based on which block transferred execution to this block. A block may have any number
        of phi nodes, but they are always the first instructions in any given block. A block with no
        phi nodes will return an empty list.
        """
        if self._phi_nodes is None:
            self._phi_nodes = list(map(QirPhiInstr, self.block.phi_nodes))
        return self._phi_nodes

    def get_phi_pairs_by_source_name(self, name: str) -> List[Tuple[str, QirOperand]]:
        """
        Gets the variable name, variable value pairs for any phi nodes in this block that correspond
        to the given name. If the name doesn't match a block that can branch to this block or if
        this block doesn't include any phi nodes, the list will be empty.
        """
        return [
            (p[0], QirOperand(p[1]))
            for p in self.block.get_phi_pairs_by_source_name(name)
        ]


class QirParameter:
    """
    Instances of the QirParameter type describe a parameter in a function definition or declaration. They
    include a type and a name, where the name is used in the function body as a variable.
    """

    def __init__(self, param: native.PyQirParameter):
        self.param = param
        self._type: Optional[QirType] = None

    @property
    def name(self) -> str:
        """
        Gets the name of this parameter, used as the variable identifier within the body of the
        function.
        """
        return self.param.name

    @property
    def type(self) -> QirType:
        """
        Gets the type of this parameter as represented in the QIR.
        """
        if self._type is None:
            self._type = QirType(self.param.type)
        return self._type


class QirFunction:
    """
    Instances of the QirFunction type represent a single function in the QIR program. They
    are made up of one or more blocks that represent function execution flow.
    """

    def __init__(self, func: native.PyQirFunction):
        self.func = func
        self._parameters: Optional[List[QirParameter]] = None
        self._return_type: Optional[QirType] = None
        self._blocks: Optional[List[QirBlock]] = None

    @property
    def name(self) -> str:
        """
        Gets the string name for this function.
        """
        return self.func.name

    @property
    def parameters(self) -> List[QirParameter]:
        """
        Gets the list of parameters used when calling this function.
        """
        if self._parameters is None:
            self._parameters = list(map(QirParameter, self.func.parameters))
        return self._parameters

    @property
    def return_type(self) -> QirType:
        """
        Gets the return type for this function.
        """
        if self._return_type is None:
            self._return_type = QirType(self.func.return_type)
        return self._return_type

    @property
    def blocks(self) -> List[QirBlock]:
        """
        Gets all the basic blocks for this function.
        """
        if self._blocks is None:
            self._blocks = list(map(QirBlock, self.func.blocks))
        return self._blocks

    @property
    def required_qubits(self) -> Optional[int]:
        """
        Gets the number of qubits needed to execute this function based on the
        "RequiredQubits" attribute, or None if that attribute is not present.
        """
        return self.func.required_qubits

    @property
    def required_results(self) -> Optional[int]:
        """
        Gets the number of result bits needed to execute this function based on the
        "RequiredResults" attribute, or None if that attribute is not present.
        """
        return self.func.required_results

    def get_attribute_value(self, name: str) -> Optional[str]:
        """
        Gets the string value of the given attribute key name, or None if that attribute
        is missing or has no defined value.
        :param name: the name of the attribute to look for
        """
        return self.func.get_attribute_value(name)

    def get_block_by_name(self, name: str) -> Optional[QirBlock]:
        """
        Gets the block with the given name, or None if no block with that name is found.
        :param name: the name of the block to look for
        """
        b = self.func.get_block_by_name(name)
        if b is not None:
            return QirBlock(b)
        return None

    def get_instruction_by_output_name(self, name: str) -> Optional[QirInstr]:
        """
        Gets the instruction anywhere in the function where the variable with a given name
        is set. Since LLVM requires any variable is defined by only one instruction, this is
        guaranteed to be no more than one instruction. This will return None if no such instruction
        can be found.
        :param name: the name of the variable to search for
        """
        instr = self.func.get_instruction_by_output_name(name)
        if instr is not None:
            return QirInstr(instr)
        return None


class QirModule:
    """
    Instances of QirModule parse a QIR program from bitcode into an in-memory
    representation for iterating over the program structure. They contain all the
    functions and global definitions from the program.
    """

    _functions: Optional[List[QirFunction]]
    _interop_funcs: Optional[List[QirFunction]]
    _entrypoint_funcs: Optional[List[QirFunction]]

    def __init__(self, name: str) -> None:
        self.module = module_from_bitcode(name)
        self._functions = None
        self._interop_funcs = None
        self._entrypoint_funcs = None

    @property
    def functions(self) -> List[QirFunction]:
        """
        Gets all the functions defined in this module.
        """
        if self._functions is None:
            self._functions = list(map(QirFunction, self.module.functions))
        return self._functions

    def get_func_by_name(self, name: str) -> Optional[QirFunction]:
        """
        Gets the function with the given name, or None if no matching function is found.
        :param name: the name of the function to get
        """
        f = self.module.get_func_by_name(name)
        if isinstance(f, native.PyQirFunction):
            return QirFunction(f)
        else:
            return None

    def get_funcs_by_attr(self, attr: str) -> List[QirFunction]:
        """
        Gets any functions that have an attribute whose name matches the provided string.
        :param attr: the attribute to use when looking for functions
        """
        return list(map(QirFunction, self.module.get_funcs_by_attr(attr)))

    @property
    def entrypoint_funcs(self) -> List[QirFunction]:
        """
        Gets any functions with the "EntryPoint" attribute.
        """
        if self._entrypoint_funcs is None:
            self._entrypoint_funcs = list(
                map(QirFunction, self.module.get_entrypoint_funcs())
            )
        return self._entrypoint_funcs

    @property
    def interop_funcs(self) -> List[QirFunction]:
        """
        Gets any functions with the "InteropFriendly" attribute.
        """
        if self._interop_funcs is None:
            self._interop_funcs = list(
                map(QirFunction, self.module.get_interop_funcs())
            )
        return self._interop_funcs

    def get_global_bytes_value(
        self, global_ref: QirGlobalByteArrayConstant
    ) -> Optional[bytes]:
        """
        Gets any globally defined bytes values matching the given global constant.
        :param global_ref: the global constant whose bytes should be retrieved.
        """
        const = global_ref.const
        if const is None:
            return None
        else:
            return const.get_global_byte_array_value(self.module)

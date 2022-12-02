# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum
from pyqir.evaluator import GateSet
from typing import Callable, List, Optional, Sequence, Tuple, Union

class ArrayType(Type):
    """An array type."""

    @property
    def element(self) -> Type:
        """The type of the array elements."""
        ...
    @property
    def count(self) -> int:
        """The number of elements in the array."""
        ...

class Attribute:
    """An attribute."""

    @property
    def value(self) -> str:
        """The value of the attribute as a string."""
        ...

class BasicBlock(Value):
    """A basic block."""

    def __init__(
        self,
        context: Context,
        name: str,
        parent: Optional[Function] = None,
        before: Optional[BasicBlock] = None,
    ) -> None:
        """
        Initializes a basic block.

        If the `before` block is given, this basic block is inserted directly before it. If no
        `before` block is given, a `parent` function must be given, and this basic block is appended
        to the end of that function.

        :param context: The LLVM context.
        :param name: The block name.
        :param parent: The parent function.
        :param before: The block to insert this block before.
        """
        ...
    @property
    def instructions(self) -> List[Instruction]:
        """The instructions in this basic block."""
        ...
    @property
    def terminator(self) -> Optional[Instruction]:
        """The terminating instruction of this basic block if there is one."""
        ...

class BasicQisBuilder:
    """An instruction builder that generates instructions from the basic quantum instruction set."""

    def __init__(self, builder: Builder) -> None:
        """
        Initializes a basic QIS builder.

        :param builder: The underlying builder used to build QIS instructions.
        """
        ...
    def cx(self, control: Value, target: Value) -> None:
        """
        Inserts a controlled Pauli :math:`X` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...
    def cz(self, control: Value, target: Value) -> None:
        """
        Inserts a controlled Pauli :math:`Z` gate.

        :param control: The control qubit.
        :param target: The target qubit.
        """
        ...
    def h(self, qubit: Value) -> None:
        """
        Inserts a Hadamard gate.

        :param qubit: The target qubit.
        """
        ...
    def mz(self, qubit: Value, result: Value) -> None:
        """
        Inserts a Z-basis measurement operation.

        :param qubit: The qubit to measure.
        :param result: A result where the measurement result will be written to.
        """
        ...
    def reset(self, qubit: Value) -> None:
        """
        Inserts a reset operation.

        :param qubit: The qubit to reset.
        """
        ...
    def rx(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`x` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...
    def ry(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`y` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...
    def rz(self, theta: Union[Value, float], qubit: Value) -> None:
        """
        Inserts a rotation gate about the :math:`z` axis.

        :param theta: The angle to rotate by.
        :param qubit: The qubit to rotate.
        """
        ...
    def s(self, qubit: Value) -> None:
        """
        Inserts an :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...
    def s_adj(self, qubit: Value) -> None:
        """
        Inserts an adjoint :math:`S` gate.

        :param qubit: The target qubit.
        """
        ...
    def t(self, qubit: Value) -> None:
        """
        Inserts a :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...
    def t_adj(self, qubit: Value) -> None:
        """
        Inserts an adjoint :math:`T` gate.

        :param qubit: The target qubit.
        """
        ...
    def x(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`X` gate.

        :param qubit: The target qubit.
        """
        ...
    def y(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`Y` gate.

        :param qubit: The target qubit.
        """
        ...
    def z(self, qubit: Value) -> None:
        """
        Inserts a Pauli :math:`Z` gate.

        :param qubit: The target qubit.
        """
        ...
    def if_result(
        self,
        cond: Value,
        one: Callable[[], None] = ...,
        zero: Callable[[], None] = ...,
    ) -> None:
        """
        Inserts a branch conditioned on a measurement result.

        Instructions inserted when ``one`` is called will be inserted into the one branch.
        Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
        and zero callables should use this module's builder to build instructions.

        :param cond: The result condition to branch on.
        :param one: A callable that inserts instructions for the branch where the result is one.
        :param zero: A callable that inserts instructions for the branch where the result is zero.
        """
        ...

class Builder:
    """An instruction builder."""

    def __init__(self, context: Context) -> None:
        """
        Initializes a builder.

        :param context: The LLVM context.
        """
        ...
    def insert_at_end(self, block: BasicBlock) -> None:
        """
        Tells this builder to insert subsequent instructions at the end of the block.

        :param block: The block to insert into.
        """
        ...
    def and_(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a bitwise logical and instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The result.
        """
        ...
    def or_(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a bitwise logical or instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The result.
        """
        ...
    def xor(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a bitwise logical exclusive or instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The result.
        """
        ...
    def add(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts an addition instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The sum.
        """
        ...
    def sub(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a subtraction instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The difference.
        """
        ...
    def mul(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a multiplication instruction.

        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The product.
        """
        ...
    def shl(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a shift left instruction.

        :param lhs: The value to shift.
        :param rhs: The number of bits to shift by.
        :returns: The result.
        """
        ...
    def lshr(self, lhs: Value, rhs: Value) -> Value:
        """
        Inserts a logical (zero fill) shift right instruction.

        :param lhs: The value to shift.
        :param rhs: The number of bits to shift by.
        :returns: The result.
        """
        ...
    def icmp(self, pred: IntPredicate, lhs: Value, rhs: Value) -> Value:
        """
        Inserts an integer comparison instruction.

        :param pred: The predicate to compare by.
        :param lhs: The left-hand side.
        :param rhs: The right-hand side.
        :returns: The boolean result.
        """
        ...
    def call(
        self,
        callee: Value,
        args: Sequence[Union[Value, bool, int, float]],
    ) -> Optional[Value]:
        """
        Inserts a call instruction.

        :param value: The value to call.
        :param args: The arguments to the function.
        :returns: The return value, or None if the function has a void return type.
        """
        ...
    def if_(
        self,
        cond: Value,
        true: Callable[[], None] = ...,
        false: Callable[[], None] = ...,
    ) -> None:
        """
        Inserts a branch conditioned on a boolean.

        Instructions inserted when ``true`` is called will be inserted into the true branch.
        Instructions inserted when ``false`` is called will be inserted into the false branch. The
        true and false callables should use this module's builder to build instructions.

        :param cond: The boolean condition to branch on.
        :param true:
            A callable that inserts instructions for the branch where the condition is true.
        :param false:
            A callable that inserts instructions for the branch where the condition is false.
        """
        ...
    def br(self, dest: BasicBlock) -> Instruction:
        """
        Inserts an unconditional branch instruction.

        :param dest: The destination block.
        :returns: The branch instruction.
        """
        ...
    def ret(self, value: Optional[Value]) -> Instruction:
        """
        Inserts a return instruction.

        :param value: The value to return. If `None`, returns void.
        :returns: The return instruction.
        """
        ...

class Call(Instruction):
    """A call instruction."""

    @property
    def callee(self) -> Value:
        """The value being called."""
        ...
    @property
    def args(self) -> List[Value]:
        """The arguments to the call."""
        ...

class Constant(Value):
    """A constant value."""

    @staticmethod
    def null(ty: Type) -> Constant:
        """
        Creates the null or zero constant for the given type.

        :param type: The type of the constant.
        :returns: The null or zero constant.
        """
        ...
    @property
    def is_null(self) -> bool:
        """Whether this value is the null value for its type."""
        ...

class Context:
    """The context owns global state needed by most LLVM objects."""

    def __init__(self) -> None:
        """Initializes a context."""
        ...

class FCmp(Instruction):
    """A floating-point comparison instruction."""

    @property
    def predicate(self) -> FloatPredicate:
        """The comparison predicate."""
        ...

class FloatConstant(Constant):
    """A constant floating-point value."""

    @property
    def value(self) -> float:
        """The value."""
        ...

class FloatPredicate(Enum):
    """A floating-point comparison predicate."""

    FALSE: FloatPredicate
    OEQ: FloatPredicate
    OGT: FloatPredicate
    OGE: FloatPredicate
    OLT: FloatPredicate
    OLE: FloatPredicate
    ONE: FloatPredicate
    ORD: FloatPredicate
    UNO: FloatPredicate
    UEQ: FloatPredicate
    UGT: FloatPredicate
    UGE: FloatPredicate
    ULT: FloatPredicate
    ULE: FloatPredicate
    UNE: FloatPredicate
    TRUE: FloatPredicate

class Function(Constant):
    """A function value."""

    def __init__(
        self, ty: FunctionType, linkage: Linkage, name: str, module: Module
    ) -> None:
        """
        Creates a new function.

        :param ty: The function type.
        :param linkage: The linkage kind.
        :param name: The function name.
        :param module: The parent module.
        """
        ...
    @property
    def type(self) -> FunctionType: ...
    @property
    def params(self) -> List[Value]:
        """The parameters to this function."""
        ...
    @property
    def basic_blocks(self) -> List[BasicBlock]:
        """The basic blocks in this function."""
        ...
    def attribute(self, name: str) -> Optional[Attribute]:
        """
        Gets an attribute of this function with the given name if it has one.

        :param name: The name of the attribute.
        :returns: The attribute.
        """
        ...

class FunctionType(Type):
    """A function type."""

    def __init__(self, ret: Type, params: Sequence[Type]) -> None:
        """
        Initializes a function type.

        :param ret: The return type.
        :param params: The parameter types.
        """
        ...
    @property
    def ret(self) -> Type:
        """The return type of the function."""
        ...
    @property
    def params(self) -> List[Type]:
        """The types of the function parameters."""
        ...

class ICmp(Instruction):
    """An integer comparison instruction."""

    @property
    def predicate(self) -> IntPredicate:
        """The comparison predicate."""
        ...

class Instruction(Value):
    """An instruction."""

    @property
    def opcode(self) -> Opcode:
        """The instruction opcode."""
        ...
    @property
    def operands(self) -> List[Value]:
        """The operands to the instruction."""
        ...
    @property
    def successors(self) -> List[BasicBlock]:
        """
        The basic blocks that are successors to this instruction. If this is not a terminator, the
        list is empty.
        """
        ...
    def erase(self) -> None:
        """
        Removes this instruction from its parent basic block, then deletes it from memory.

        Warning: Using this instruction after erasing it is undefined behavior.
        """
        ...

class IntConstant(Constant):
    """A constant integer value."""

    @property
    def type(self) -> IntType: ...
    @property
    def value(self) -> int:
        """The value."""
        ...

class IntPredicate(Enum):
    """An integer comparison predicate."""

    EQ: IntPredicate
    NE: IntPredicate
    UGT: IntPredicate
    UGE: IntPredicate
    ULT: IntPredicate
    ULE: IntPredicate
    SGT: IntPredicate
    SGE: IntPredicate
    SLT: IntPredicate
    SLE: IntPredicate

class IntType(Type):
    """An integer type."""

    def __init__(self, context: Context, width: int) -> None:
        """
        Initializes an integer type.

        :param context: The LLVM context.
        :param width: The number of bits in the integer.
        """
        ...
    @property
    def width(self) -> int:
        """The number of bits in the integer."""
        ...

class Linkage(Enum):
    """The linkage kind for a global value in a module."""

    APPENDING: Linkage
    AVAILABLE_EXTERNALLY: Linkage
    COMMON: Linkage
    EXTERNAL: Linkage
    EXTERNAL_WEAK: Linkage
    INTERNAL: Linkage
    LINK_ONCE_ANY: Linkage
    LINK_ONCE_ODR: Linkage
    PRIVATE: Linkage
    WEAK_ANY: Linkage
    WEAK_ODR: Linkage

class Module:
    """A module is a collection of global values."""

    def __init__(self, context: Context, name: str) -> None:
        """
        Initializes a module.

        :param context: The LLVM context.
        :param name: The module name.
        """
        ...
    @staticmethod
    def from_ir(context: Context, ir: str, name: str = "") -> Module:
        """
        Creates a module from LLVM IR.

        :param ir: The LLVM IR for a module.
        :param name: The name of the module.
        :returns: The module.
        """
        ...
    @staticmethod
    def from_bitcode(context: Context, bitcode: bytes, name: str = "") -> Module:
        """
        Creates a module from LLVM bitcode.

        :param bitcode: The LLVM bitcode for a module.
        :param name: The name of the module.
        :returns: The module.
        """
        ...
    @property
    def source_filename(self) -> str:
        """The name of the original source file that this module was compiled from."""
        ...
    @source_filename.setter
    def source_filename(self, value: str) -> None: ...
    @property
    def functions(self) -> List[Function]:
        """The functions declared in this module."""
        ...
    @property
    def bitcode(self) -> bytes:
        """The LLVM bitcode for this module."""
        ...
    @property
    def context(self) -> Context:
        """The LLVM context."""
        ...
    def verify(self) -> Optional[str]:
        """
        Verifies that this module is valid.

        :returns: An error description if this module is invalid or `None` if this module is valid.
        """
        ...
    def __str__(self) -> str:
        """Converts this module into an LLVM IR string."""
        ...

class ModuleFlagBehavior(Enum):
    ERROR: ModuleFlagBehavior
    WARNING: ModuleFlagBehavior
    REQUIRE: ModuleFlagBehavior
    OVERRIDE: ModuleFlagBehavior
    APPEND: ModuleFlagBehavior
    APPEND_UNIQUE: ModuleFlagBehavior

class Opcode(Enum):
    """An instruction opcode."""

    ADD: Opcode
    ADDR_SPACE_CAST: Opcode
    ALLOCA: Opcode
    AND: Opcode
    ASHR: Opcode
    ATOMIC_CMP_XCHG: Opcode
    ATOMIC_RMW: Opcode
    BIT_CAST: Opcode
    BR: Opcode
    CALL_BR: Opcode
    CALL: Opcode
    CATCH_PAD: Opcode
    CATCH_RET: Opcode
    CATCH_SWITCH: Opcode
    CLEANUP_PAD: Opcode
    CLEANUP_RET: Opcode
    EXTRACT_ELEMENT: Opcode
    EXTRACT_VALUE: Opcode
    FADD: Opcode
    FCMP: Opcode
    FDIV: Opcode
    FENCE: Opcode
    FMUL: Opcode
    FNEG: Opcode
    FP_EXT: Opcode
    FP_TO_SI: Opcode
    FP_TO_UI: Opcode
    FP_TRUNC: Opcode
    FREEZE: Opcode
    FREM: Opcode
    FSUB: Opcode
    GET_ELEMENT_PTR: Opcode
    ICMP: Opcode
    INDIRECT_BR: Opcode
    INSERT_ELEMENT: Opcode
    INSERT_VALUE: Opcode
    INT_TO_PTR: Opcode
    INVOKE: Opcode
    LANDING_PAD: Opcode
    LOAD: Opcode
    LSHR: Opcode
    MUL: Opcode
    OR: Opcode
    PHI: Opcode
    PTR_TO_INT: Opcode
    RESUME: Opcode
    RET: Opcode
    SDIV: Opcode
    SELECT: Opcode
    SEXT: Opcode
    SHL: Opcode
    SHUFFLE_VECTOR: Opcode
    SI_TO_FP: Opcode
    SREM: Opcode
    STORE: Opcode
    SUB: Opcode
    SWITCH: Opcode
    TRUNC: Opcode
    UDIV: Opcode
    UI_TO_FP: Opcode
    UNREACHABLE: Opcode
    UREM: Opcode
    USER_OP_1: Opcode
    USER_OP_2: Opcode
    VA_ARG: Opcode
    XOR: Opcode
    ZEXT: Opcode

class Phi(Instruction):
    """A phi node instruction."""

    @property
    def incoming(self) -> List[Tuple[Value, BasicBlock]]:
        """The incoming values and their preceding basic blocks."""
        ...

class PointerType(Type):
    """A pointer type."""

    def __init__(self, pointee: Type) -> None:
        """
        Initializes a pointer type.

        :param pointee: The type being pointed to.
        """
        ...
    @property
    def pointee(self) -> Type:
        """The type being pointed to."""
        ...
    @property
    def address_space(self) -> int:
        """The pointer address space."""
        ...

class PyNonadaptiveJit:
    def eval(
        self,
        file_path: str,
        gateset: GateSet,
        entry_point: Optional[str],
        result_stream: Optional[List[bool]],
    ) -> None: ...

class StructType(Type):
    """A structure type."""

    @property
    def name(self) -> Optional[str]:
        """The name of the structure or the empty string if the structure is anonymous."""
        ...
    @property
    def fields(self) -> List[Type]:
        """The types of the structure fields."""
        ...

class Switch(Instruction):
    """A switch instruction."""

    @property
    def cond(self) -> Value:
        """The condition of the switch."""
        ...
    @property
    def default(self) -> BasicBlock:
        """The default successor block if none of the cases match."""
        ...
    @property
    def cases(self) -> List[Tuple[IntConstant, BasicBlock]]:
        """The switch cases."""
        ...

class Type:
    """A type."""

    @staticmethod
    def void(context: Context) -> Type:
        """
        The void type.

        :param context: The LLVM context.
        :returns: The void type.
        """
        ...
    @staticmethod
    def double(context: Context) -> Type:
        """
        The double type.

        :param context: The LLVM context.
        :returns: The double type.
        """
        ...
    @property
    def is_void(self) -> bool:
        """Whether this type is the void type."""
        ...
    @property
    def is_double(self) -> bool:
        """Whether this type is the bool type."""
        ...

class Value:
    """A value."""

    @property
    def type(self) -> Type:
        """The type of this value."""
        ...
    @property
    def name(self) -> str:
        """The name of this value or the empty string if this value is anonymous."""
        ...

def const(ty: Type, value: Union[bool, int, float]) -> Constant:
    """
    Creates a constant value.

    :param ty: The type of the value.
    :param value: The value of the constant.
    :returns: The constant value.
    """
    ...

def entry_point(
    module: Module, name: str, required_num_qubits: int, required_num_results: int
) -> Function:
    """
    Creates an entry point.

    :param module: The parent module.
    :param name: The entry point name.
    :param required_num_qubits: The number of qubits required by the entry point.
    :param required_num_results: The number of results required by the entry point.
    :returns: An entry point.
    """
    ...

def extract_byte_string(value: Value) -> Optional[bytes]:
    """
    If the value is a pointer to a constant byte string, extracts it.

    :param value: The value.
    :returns: The constant byte string.
    """
    ...

def global_byte_string(module: Module, value: bytes) -> Constant:
    """
    Creates a global null-terminated byte string constant in a module.

    :param module: The parent module.
    :param value: The byte string value without a null terminator.
    :returns: A pointer to the start of the null-terminated byte string.
    """
    ...

def is_entry_point(function: Function) -> bool:
    """
    Whether the function is an entry point.

    :param function: The function.
    :returns: True if the function is an entry point.
    """
    ...

def is_interop_friendly(function: Function) -> bool:
    """
    Whether the function is interop-friendly.

    :param function: The function.
    :returns: True if the function is interop-friendly.
    """
    ...

def is_qubit_type(ty: Type) -> bool:
    """
    Whether the type is the QIR qubit type.

    :param Type ty: The type.
    :returns: True if the type is the QIR qubit type.
    """
    ...

def is_result_type(ty: Type) -> bool:
    """
    Whether the type is the QIR result type.

    :param ty: The type.
    :returns: True if the type is the QIR result type.
    """
    ...

def qubit(context: Context, id: int) -> Constant:
    """
    Creates a static qubit value.

    :param context: The LLVM context.
    :param id: The static qubit ID.
    :returns: A static qubit value.
    """
    ...

def qubit_id(value: Value) -> Optional[int]:
    """
    If the value is a static qubit ID, extracts it.

    :param value: The value.
    :returns: The static qubit ID.
    """
    ...

def qubit_type(context: Context) -> Type:
    """
    The QIR qubit type.

    :param context: The LLVM context.
    :returns: The qubit type.
    """
    ...

def required_num_qubits(function: Function) -> Optional[int]:
    """
    If the function declares a required number of qubits, extracts it.

    :param function: The function.
    :returns: The required number of qubits.
    """
    ...

def required_num_results(function: Function) -> Optional[int]:
    """
    If the function declares a required number of results, extracts it.

    :param function: The function.
    :returns: The required number of results.
    """
    ...

def result(context: Context, id: int) -> Constant:
    """
    Creates a static result value.

    :param context: The LLVM context.
    :param id: The static result ID.
    :returns: A static result value.
    """
    ...

def result_id(value: Value) -> Optional[int]:
    """
    If the value is a static result ID, extracts it.

    :param value: The value.
    :returns: The static result ID.
    """
    ...

def result_type(context: Context) -> Type:
    """
    The QIR result type.

    :param Context context: The LLVM context.
    :returns: The result type.
    """
    ...

def get_flag(module: Module, name: str) -> Optional[Value]: ...
def add_value_flag(
    module: Module, name: str, behavior: ModuleFlagBehavior, value: Value
) -> Optional[str]: ...
def add_metadata_flag(
    module: Module, name: str, behavior: ModuleFlagBehavior, value: Value
) -> Optional[str]: ...

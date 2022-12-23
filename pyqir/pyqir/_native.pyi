# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum
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
    def string_value(self) -> Optional[str]:
        """The value of this attribute as a string, or `None` if this is not a string attribute."""
        ...

class AttributeList:
    """The attribute list for a function."""

    def param(self, n: int) -> AttributeSet:
        """
        The attributes for a parameter.

        :param n: The parameter number, starting from zero.
        :returns: The parameter attributes.
        """
        ...
    @property
    def ret(self) -> AttributeSet:
        """The attributes for the return type."""
        ...
    @property
    def func(self) -> AttributeSet:
        """The attributes for the function itself."""
        ...

class AttributeSet:
    """A set of attributes for a specific part of a function."""

    def __contains__(self, item: str) -> bool:
        """
        Tests if an attribute is a member of the set.

        :param item: The attribute kind.
        :returns: True if the group has an attribute with the given kind.
        """
        ...
    def __getitem__(self, key: str) -> Attribute:
        """
        Gets an attribute based on its kind.

        :param key: The attribute kind.
        :returns: The attribute.
        """
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
    ) -> Value:
        """
        Inserts a call instruction.

        :param value: The value to call.
        :param args: The arguments to the function.
        :returns: The instruction value.
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
    @property
    def attributes(self) -> AttributeList:
        """The attributes for this function."""
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
    def add_flag(
        self, behavior: ModuleFlagBehavior, id: str, flag: Union[Metadata, Constant]
    ) -> None:
        """
        Adds a flag to the llvm.module.flags metadata

        See https://llvm.org/docs/LangRef.html#module-flags-metadata

        :param ModuleFlagBehavior behavior: flag specifying the behavior when two (or more) modules are merged together
        :param str id: string that is a unique ID for the metadata.
        :param Union[Metadata, Constant] flag: value of the flag
        """
        ...
    def get_flag(self, id: str) -> Optional[Metadata]:
        """
        Gets the flag value from the llvm.module.flags metadata for a given id

        See https://llvm.org/docs/LangRef.html#module-flags-metadata

        :param id: metadata string that is a unique ID for the metadata.
        :returns: value of the flag if found, otherwise None
        """
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
    """Module flag behavior choices"""

    ERROR: ModuleFlagBehavior
    WARNING: ModuleFlagBehavior
    REQUIRE: ModuleFlagBehavior
    OVERRIDE: ModuleFlagBehavior
    APPEND: ModuleFlagBehavior
    APPEND_UNIQUE: ModuleFlagBehavior
    MAX: ModuleFlagBehavior

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

class Metadata:
    """A metadata value."""

    ...

class MetadataString(Metadata):
    """A metadata string"""

    def __init__(self, context: Context, string: str) -> None:
        """
        Creates a metadata string

        :param context: The LLVM context.
        :param string: the value of the metadata string to create
        """
        ...
    @property
    def value(self) -> str:
        """The underlying metadata string value."""
        ...

class ConstantAsMetadata(Metadata):
    """A metadata constant value."""

    @property
    def value(self) -> Constant:
        """The underlying metadata constant value."""
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
    module: Module,
    name: str,
    required_num_qubits: int,
    required_num_results: int,
    qir_profiles: Optional[str] = "custom",
    output_labeling_schema: Optional[str] = "",
) -> Function:
    """
    Creates an entry point.

    :param Module module: The parent module.
    :param str name: The entry point name.
    :param int required_num_qubits: The number of qubits required by the entry point.
    :param int required_num_results: The number of results required by the entry point.
    :param Optional[str] qir_profiles: Value identifying the profile the entry point has been compiled for. Use base_profile when QIR is compliant.
    :param Optional[str] output_labeling_schema: An arbitrary string value that identifies the schema used by a compiler frontend that produced the IR to label the recorded output
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

# Runtime

def array_record_output(builder: Builder, num_elements: Value, label: Value) -> None:
    """
    Inserts a marker in the generated output that indicates the start
    of an array and how many array elements it has.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value num_elements: How many array elements the array has
    :param Value label: A string label for the array. Depending on the output schema, the label is included in the output or omitted.
    """
    ...

def initialize(builder: Builder, data: Value) -> None:
    """
    Initializes the execution environment. Sets all qubits to a zero-state
    if they are not dynamically managed.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value data: For base profile QIR, a const null i8* Value should be passed.
    """
    ...

def result_record_output(builder: Builder, result: Value, label: Value) -> None:
    """
    Adds a measurement result to the generated output.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value result: A result measurement to record
    :param Value label: A string label for the result value. Depending on the output schema, the label is included in the output or omitted.
    """
    ...

def tuple_record_output(builder: Builder, num_elements: Value, label: Value) -> None:
    """
    Inserts a marker in the generated output that indicates the start
    of a tuple and how many tuple elements it has.

    :param Builder builder: The IR Builder used to create the instructions
    :param Value num_elements: How many tuple elements the tuple has
    :param Value label: A string label for the tuple. Depending on the output schema, the label is included in the output or omitted.
    """
    ...

# QIS

def barrier(builder: Builder) -> None:
    """
    Inserts a barrier instruction

    :param builder: The underlying builder used to build QIS instructions.
    :rtype: None
    """
    ...

def swap(builder: Builder, qubit1: Value, qubit2: Value) -> None:
    """
    Inserts a swap gate

    :param builder: The underlying builder used to build QIS instructions.
    :param Value qubit1: The first qubit to apply the gate to.
    :param Value qubit2: The second qubit to apply the gate to.
    :rtype: None
    """
    ...

def ccx(builder: Builder, control1: Value, control2: Value, target: Value) -> None:
    """
    Inserts Toffoli or doubly-controlled :math:`X` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param Value control1: The first control qubit.
    :param Value control2: The second control qubit.
    :param Value target: The target qubit.
    :rtype: None
    """
    ...

def cx(builder: Builder, control: Value, target: Value) -> None:
    """
    Inserts a controlled Pauli :math:`X` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param control: The control qubit.
    :param target: The target qubit.
    """
    ...

def cz(builder: Builder, control: Value, target: Value) -> None:
    """
    Inserts a controlled Pauli :math:`Z` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param control: The control qubit.
    :param target: The target qubit.
    """
    ...

def h(builder: Builder, qubit: Value) -> None:
    """
    Inserts a Hadamard gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def mz(builder: Builder, qubit: Value, result: Value) -> None:
    """
    Inserts a Z-basis measurement operation.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The qubit to measure.
    :param result: A result where the measurement result will be written to.
    """
    ...

def reset(builder: Builder, qubit: Value) -> None:
    """
    Inserts a reset operation.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The qubit to reset.
    """
    ...

def rx(builder: Builder, theta: Union[Value, float], qubit: Value) -> None:
    """
    Inserts a rotation gate about the :math:`x` axis.

    :param builder: The underlying builder used to build QIS instructions.
    :param theta: The angle to rotate by.
    :param qubit: The qubit to rotate.
    """
    ...

def ry(builder: Builder, theta: Union[Value, float], qubit: Value) -> None:
    """
    Inserts a rotation gate about the :math:`y` axis.

    :param builder: The underlying builder used to build QIS instructions.
    :param theta: The angle to rotate by.
    :param qubit: The qubit to rotate.
    """
    ...

def rz(builder: Builder, theta: Union[Value, float], qubit: Value) -> None:
    """
    Inserts a rotation gate about the :math:`z` axis.

    :param builder: The underlying builder used to build QIS instructions.
    :param theta: The angle to rotate by.
    :param qubit: The qubit to rotate.
    """
    ...

def s(builder: Builder, qubit: Value) -> None:
    """
    Inserts an :math:`S` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def s_adj(builder: Builder, qubit: Value) -> None:
    """
    Inserts an adjoint :math:`S` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def t(builder: Builder, qubit: Value) -> None:
    """
    Inserts a :math:`T` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def t_adj(builder: Builder, qubit: Value) -> None:
    """
    Inserts an adjoint :math:`T` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def x(builder: Builder, qubit: Value) -> None:
    """
    Inserts a Pauli :math:`X` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def y(builder: Builder, qubit: Value) -> None:
    """
    Inserts a Pauli :math:`Y` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def z(builder: Builder, qubit: Value) -> None:
    """
    Inserts a Pauli :math:`Z` gate.

    :param builder: The underlying builder used to build QIS instructions.
    :param qubit: The target qubit.
    """
    ...

def if_result(
    builder: Builder,
    cond: Value,
    one: Callable[[], None] = ...,
    zero: Callable[[], None] = ...,
) -> None:
    """
    Inserts a branch conditioned on a measurement result.

    Instructions inserted when ``one`` is called will be inserted into the one branch.
    Instructions inserted when ``zero`` is called will be inserted into the zero branch. The one
    and zero callables should use this module's builder to build instructions.

    :param builder: The underlying builder used to build QIS instructions.
    :param cond: The result condition to branch on.
    :param one: A callable that inserts instructions for the branch where the result is one.
    :param zero: A callable that inserts instructions for the branch where the result is zero.
    """
    ...

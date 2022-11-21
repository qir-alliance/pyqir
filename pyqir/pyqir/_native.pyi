# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from enum import Enum
from pyqir.evaluator import GateSet
from typing import Callable, List, Optional, Sequence, Tuple, Union

class Context:
    def __init__(self) -> None: ...

class Builder:
    def __init__(self, context: Context) -> None: ...
    def insert_at_end(self, block: BasicBlock) -> None: ...
    def and_(self, lhs: Value, rhs: Value) -> Value: ...
    def or_(self, lhs: Value, rhs: Value) -> Value: ...
    def xor(self, lhs: Value, rhs: Value) -> Value: ...
    def add(self, lhs: Value, rhs: Value) -> Value: ...
    def sub(self, lhs: Value, rhs: Value) -> Value: ...
    def mul(self, lhs: Value, rhs: Value) -> Value: ...
    def shl(self, lhs: Value, rhs: Value) -> Value: ...
    def lshr(self, lhs: Value, rhs: Value) -> Value: ...
    def icmp(self, pred: IntPredicate, lhs: Value, rhs: Value) -> Value: ...
    def call(
        self,
        callee: Value,
        args: Sequence[Union[Value, bool, int, float]],
    ) -> Optional[Value]: ...
    def if_(
        self,
        cond: Value,
        true: Callable[[], None] = ...,
        false: Callable[[], None] = ...,
    ) -> None: ...
    def ret(self, value: Optional[Value]) -> Instruction: ...

class BasicQisBuilder:
    def __init__(self, builder: Builder) -> None: ...
    def cx(self, control: Value, target: Value) -> None: ...
    def cz(self, control: Value, target: Value) -> None: ...
    def h(self, qubit: Value) -> None: ...
    def mz(self, qubit: Value, result: Value) -> None: ...
    def reset(self, qubit: Value) -> None: ...
    def rx(self, theta: Union[Value, float], qubit: Value) -> None: ...
    def ry(self, theta: Union[Value, float], qubit: Value) -> None: ...
    def rz(self, theta: Union[Value, float], qubit: Value) -> None: ...
    def s(self, qubit: Value) -> None: ...
    def s_adj(self, qubit: Value) -> None: ...
    def t(self, qubit: Value) -> None: ...
    def t_adj(self, qubit: Value) -> None: ...
    def x(self, qubit: Value) -> None: ...
    def y(self, qubit: Value) -> None: ...
    def z(self, qubit: Value) -> None: ...
    def if_result(
        self,
        cond: Value,
        one: Callable[[], None] = ...,
        zero: Callable[[], None] = ...,
    ) -> None: ...

class SimpleModule:
    def __init__(
        self,
        name: str,
        num_qubits: int,
        num_results: int,
        context: Optional[Context] = None,
    ) -> None: ...
    @property
    def context(self) -> Context: ...
    @property
    def qubits(self) -> List[Value]: ...
    @property
    def results(self) -> List[Value]: ...
    @property
    def builder(self) -> Builder: ...
    def ir(self) -> str: ...
    def bitcode(self) -> bytes: ...
    def add_external_function(self, name: str, ty: FunctionType) -> Function: ...
    def add_global_string(self, value: bytes) -> Value: ...

class Type:
    @staticmethod
    def void(context: Context) -> Type: ...
    @staticmethod
    def double(context: Context) -> Type: ...
    @property
    def is_void(self) -> bool: ...
    @property
    def is_double(self) -> bool: ...

class IntType(Type):
    def __init__(self, context: Context, width: int) -> None: ...
    @property
    def width(self) -> int: ...

class FunctionType(Type):
    def __init__(self, ret: Type, params: Sequence[Type]) -> None: ...
    @property
    def ret(self) -> Type: ...
    @property
    def params(self) -> List[Type]: ...

class StructType(Type):
    @property
    def name(self) -> Optional[str]: ...
    @property
    def fields(self) -> List[Type]: ...

class ArrayType(Type):
    @property
    def element(self) -> Type: ...
    @property
    def count(self) -> int: ...

class PointerType(Type):
    def __init__(self, pointee: Type) -> None: ...
    @property
    def pointee(self) -> Type: ...
    @property
    def address_space(self) -> int: ...

def qubit_type(context: Context) -> Type: ...
def is_qubit_type(ty: Type) -> bool: ...
def result_type(context: Context) -> Type: ...
def is_result_type(ty: Type) -> bool: ...

class Value:
    @property
    def type(self) -> Type: ...
    @property
    def name(self) -> str: ...

class BasicBlock(Value):
    def __init__(
        self,
        context: Context,
        name: str,
        parent: Optional[Function] = None,
        before: Optional[BasicBlock] = None,
    ) -> None: ...
    @property
    def instructions(self) -> List[Instruction]: ...
    @property
    def terminator(self) -> Optional[Instruction]: ...

class Constant(Value):
    @property
    def is_null(self) -> bool: ...

class IntConstant(Constant):
    @property
    def type(self) -> IntType: ...
    @property
    def value(self) -> int: ...

class FloatConstant(Constant):
    @property
    def value(self) -> float: ...

class Function(Constant):
    def __init__(
        self, ty: FunctionType, linkage: Linkage, name: str, module: Module
    ) -> None: ...
    @property
    def type(self) -> FunctionType: ...
    @property
    def params(self) -> List[Value]: ...
    @property
    def basic_blocks(self) -> List[BasicBlock]: ...
    def attribute(self, name: str) -> Optional[Attribute]: ...

class ConstantExpr(Constant):
    @staticmethod
    def getelementptr(
        value: Value, indices: Sequence[Value], inbounds: bool
    ) -> ConstantExpr: ...

class Linkage(Enum):
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

class Attribute:
    @property
    def value(self) -> str: ...

def const(ty: Type, value: Union[int, float]) -> Value: ...
def constant_bytes(value: Value) -> Optional[bytes]: ...
def entry_point(
    module: Module, name: str, required_num_qubits: int, required_num_results: int
) -> Function: ...
def is_entry_point(function: Function) -> bool: ...
def is_interop_friendly(function: Function) -> bool: ...
def qubit_id(value: Value) -> Optional[int]: ...
def qubit(context: Context, id: int) -> Value: ...
def required_num_qubits(function: Function) -> Optional[int]: ...
def required_num_results(function: Function) -> Optional[int]: ...
def result_id(value: Value) -> Optional[int]: ...
def result(context: Context, id: int) -> Value: ...

class Opcode(Enum):
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

class IntPredicate(Enum):
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

class FloatPredicate(Enum):
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

class Instruction(Value):
    @property
    def opcode(self) -> Opcode: ...
    @property
    def operands(self) -> List[Value]: ...
    @property
    def successors(self) -> List[BasicBlock]: ...

class Switch(Instruction):
    @property
    def cond(self) -> Value: ...
    @property
    def default(self) -> BasicBlock: ...
    @property
    def cases(self) -> List[Tuple[IntConstant, BasicBlock]]: ...

class ICmp(Instruction):
    @property
    def predicate(self) -> IntPredicate: ...

class FCmp(Instruction):
    @property
    def predicate(self) -> FloatPredicate: ...

class Call(Instruction):
    @property
    def callee(self) -> Value: ...
    @property
    def args(self) -> List[Value]: ...

class Phi(Instruction):
    @property
    def incoming(self) -> List[Tuple[Value, BasicBlock]]: ...

class Module:
    def __init__(self, context: Context, name: str) -> None: ...
    @staticmethod
    def from_ir(ir: str, name: str = "") -> Module: ...
    @staticmethod
    def from_bitcode(bitcode: bytes, name: str = "") -> Module: ...
    @property
    def source_filename(self) -> str: ...
    @source_filename.setter
    def source_filename(self, value: str) -> None: ...
    @property
    def functions(self) -> List[Function]: ...
    @property
    def bitcode(self) -> bytes: ...
    @property
    def context(self) -> Context: ...
    def __str__(self) -> str: ...

def verify_module(module: Module) -> Optional[str]: ...

class PyNonadaptiveJit:
    def eval(
        self,
        file_path: str,
        gateset: GateSet,
        entry_point: Optional[str],
        result_stream: Optional[List[bool]],
    ) -> None: ...

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator._builder import IntPredicate
from typing import Callable, List, Optional, Sequence, Tuple, Union

class Context: ...
class Type: ...

class VoidType(Type):
    def __init__(self, context: Context) -> None: ...

class IntegerType(Type):
    def __init__(self, context: Context, width: int) -> None: ...

class DoubleType(Type):
    def __init__(self, context: Context) -> None: ...

class FunctionType(Type):
    def __init__(self, return_: Type, params: List[Type]) -> None: ...

class StructType(Type):
    @staticmethod
    def opaque(context: Context, name: str) -> StructType: ...

class ArrayType(Type):
    def __init__(self, element: Type, count: int) -> None: ...

class PointerType(Type):
    def __init__(self, pointee: Type, address_space: int) -> None: ...

class Function: ...
class Value: ...

def const(ty: Type, value: Union[int, float]) -> Value: ...

class Builder:
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
        function: Function,
        args: Sequence[Union[Value, bool, int, float]],
    ) -> Optional[Value]: ...
    def if_(
        self,
        cond: Value,
        true: Callable[[], None] = ...,
        false: Callable[[], None] = ...,
    ) -> None: ...

class SimpleModule:
    def __init__(self, name: str, num_qubits: int, num_results: int) -> None: ...
    @property
    def context(self) -> Context: ...
    @property
    def qubits(self) -> Tuple[Value, ...]: ...
    @property
    def results(self) -> Tuple[Value, ...]: ...
    @property
    def builder(self) -> Builder: ...
    def ir(self) -> str: ...
    def bitcode(self) -> bytes: ...
    def add_external_function(self, name: str, ty: Type) -> Function: ...

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
        result: Value,
        one: Callable[[], None] = ...,
        zero: Callable[[], None] = ...,
    ) -> None: ...

def ir_to_bitcode(
    ir: str, module_name: Optional[str] = ..., source_file_name: Optional[str] = ...
) -> bytes: ...
def bitcode_to_ir(
    bitcode: bytes,
    module_name: Optional[str] = ...,
    source_file_name: Optional[str] = ...,
) -> str: ...

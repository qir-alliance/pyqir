# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""
Provides the ability to generate QIR using a Python API.

This package is intended to be used by code automating translation processes
enabling the conversion in some format to QIR via Python; i.e., this is a low
level API intended to be used as a bridge to existing Python frameworks enabling
the generation of QIR rather than directly consumed by an end-user. It is not
intended to be used as a framework for algorithm and application development.
"""

from pyqir.generator._native import (
    ArrayType as ArrayType,
    Attribute as Attribute,
    BasicBlock as BasicBlock,
    BasicQisBuilder as BasicQisBuilder,
    Builder as Builder,
    Call as Call,
    Constant as Constant,
    FCmp as FCmp,
    FloatConstant as FloatConstant,
    FloatPredicate as FloatPredicate,
    Function as Function,
    FunctionType as FunctionType,
    ICmp as ICmp,
    Instruction as Instruction,
    IntConstant as IntConstant,
    IntPredicate as IntPredicate,
    IntType as IntType,
    Module as Module,
    Opcode as Opcode,
    Phi as Phi,
    PointerType as PointerType,
    SimpleModule as SimpleModule,
    StructType as StructType,
    Switch as Switch,
    Type as Type,
    TypeFactory as TypeFactory,
    Value as Value,
    bitcode_to_ir as bitcode_to_ir,
    const as const,
    global_byte_string_value as global_byte_string_value,
    global_byte_string_value_name as global_byte_string_value_name,
    ir_to_bitcode as ir_to_bitcode,
    is_entry_point as is_entry_point,
    is_interop_friendly as is_interop_friendly,
    is_qubit as is_qubit,
    is_result as is_result,
    qubit_id as qubit_id,
    required_num_qubits as required_num_qubits,
    required_num_results as required_num_results,
    result_id as result_id,
)

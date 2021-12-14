# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

from typing import List
from dataclasses import dataclass
from pyqir_generator.instruction import *
import pyqir_generator.pyqir_generator as native


@dataclass
class Register:
    name: str
    size: int


@dataclass
class Module:
    name: str
    bits: List[Register]
    qubits: List[Register]
    instructions: List[Instruction]

    def ir(self) -> str:
        return self._native_module().ir()

    def bitcode_base64(self) -> str:
        return self._native_module().bitcode_base64()

    def write(self, path: str) -> None:
        self._native_module().write(path)

    def _native_module(self) -> native.Module:
        # TODO: This is not very efficient to do every time.
        return native.Module(
            name=self.name,
            bits=list(map(_native_register, self.bits)),
            qubits=list(map(_native_register, self.qubits)),
            instructions=list(map(_native_instruction, self.instructions))
        )


def enable_logging() -> None:
    native.enable_logging()


def _native_instruction(instruction: Instruction) -> native.Instruction:
    match instruction:
        case Cx(control, target): return native.cx(control, target)
        case Cz(control, target): return native.cz(control, target)
        case H(qubit): return native.h(qubit)
        case M(qubit, target): return native.m(qubit, target)
        case Reset(qubit): return native.reset(qubit)
        case Rx(theta, qubit): return native.rx(theta, qubit)
        case Ry(theta, qubit): return native.ry(theta, qubit)
        case Rz(theta, qubit): return native.rz(theta, qubit)
        case S(qubit): return native.s(qubit)
        case SAdj(qubit): return native.s_adj(qubit)
        case T(qubit): return native.t(qubit)
        case TAdj(qubit): return native.t_adj(qubit)
        case X(qubit): return native.x(qubit)
        case Y(qubit): return native.y(qubit)
        case Z(qubit): return native.z(qubit)
        case DumpMachine(): return native.dump_machine
        case If(condition, if_true, if_false):
            if_true = list(map(_native_instruction, if_true))
            if_false = list(map(_native_instruction, if_false))
            return native.if_(condition, if_true, if_false)
        case _: raise ValueError("Unsupported instruction.")


def _native_register(register: Register) -> native.Register:
    return native.Register(register.name, register.size)

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""
Smoke tests to check that basic Python API functionality works and generates an
IR string without errors. These tests are not meant to make detailed assertions
about the generated IR.
"""

from pathlib import Path

import pytest

import pyqir
from pyqir import (
    BasicQisBuilder,
    Context,
    Function,
    FunctionType,
    Instruction,
    IntType,
    Linkage,
    Module,
    SimpleModule,
    Type,
)


def test_bell() -> None:
    module = SimpleModule("Bell circuit", num_qubits=2, num_results=2)
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])
    qis.mz(module.qubits[0], module.results[0])
    qis.mz(module.qubits[1], module.results[1])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'Bell circuit'")


def test_bell_no_measure() -> None:
    module = SimpleModule("Bell circuit", num_qubits=2, num_results=0)
    qis = BasicQisBuilder(module.builder)
    qis.h(module.qubits[0])
    qis.cx(module.qubits[0], module.qubits[1])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'Bell circuit'")


def test_bernstein_vazirani() -> None:
    module = SimpleModule("Bernstein-Vazirani", num_qubits=6, num_results=5)
    qis = BasicQisBuilder(module.builder)
    inputs = module.qubits[:5]
    target = module.qubits[5]
    outputs = module.results

    qis.x(target)

    qis.h(inputs[0])
    qis.h(inputs[1])
    qis.h(inputs[2])
    qis.h(inputs[3])
    qis.h(inputs[4])

    qis.h(target)

    qis.cx(inputs[1], target)
    qis.cx(inputs[3], target)
    qis.cx(inputs[4], target)

    qis.h(inputs[0])
    qis.h(inputs[1])
    qis.h(inputs[2])
    qis.h(inputs[3])
    qis.h(inputs[4])

    qis.mz(inputs[0], outputs[0])
    qis.mz(inputs[1], outputs[1])
    qis.mz(inputs[2], outputs[2])
    qis.mz(inputs[3], outputs[3])
    qis.mz(inputs[4], outputs[4])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'Bernstein-Vazirani'")


def test_all_gates() -> None:
    module = SimpleModule("All Gates", num_qubits=5, num_results=5)
    qis = BasicQisBuilder(module.builder)
    q = module.qubits[:4]
    control = module.qubits[4]
    c = module.results

    qis.cx(q[0], control)
    qis.cz(q[1], control)
    qis.h(q[0])
    qis.reset(q[0])
    qis.rx(15.0, q[1])
    qis.ry(16.0, q[2])
    qis.rz(17.0, q[3])
    qis.s(q[0])
    qis.s_adj(q[1])
    qis.t(q[2])
    qis.t_adj(q[3])
    qis.x(q[0])
    qis.y(q[1])
    qis.z(q[2])

    qis.mz(q[0], c[0])
    qis.mz(q[1], c[1])
    qis.mz(q[2], c[2])
    qis.mz(q[3], c[3])

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'All Gates'")


def test_if() -> None:
    module = SimpleModule("If", num_qubits=1, num_results=1)
    qis = BasicQisBuilder(module.builder)
    f = module.add_external_function("f", FunctionType(IntType(module.context, 1), []))

    b = module.builder.call(f, [])
    assert b is not None
    module.builder.if_(
        cond=b,
        true=lambda: qis.x(module.qubits[0]),
        false=lambda: qis.h(module.qubits[0]),
    )

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'If'")


def test_if_result() -> None:
    module = SimpleModule("If Result", num_qubits=1, num_results=1)
    qis = BasicQisBuilder(module.builder)

    qis.mz(module.qubits[0], module.results[0])
    qis.if_result(
        cond=module.results[0],
        one=lambda: qis.x(module.qubits[0]),
        zero=lambda: qis.h(module.qubits[0]),
    )

    ir = module.ir()
    assert ir.startswith("; ModuleID = 'If Result'")


def test_multiple_contexts() -> None:
    m1 = SimpleModule("m1", 0, 0)
    m2 = SimpleModule("m2", 0, 0)
    with pytest.raises(ValueError, match=r"^Owners are incompatible\.$"):
        m1.add_external_function(
            "f",
            FunctionType(pyqir.result_type(m1.context), [pyqir.qubit_type(m2.context)]),
        )


def test_ir_idempotence() -> None:
    m = SimpleModule("ir_idempotence", num_qubits=1, num_results=0)
    qis = BasicQisBuilder(m.builder)
    qis.x(m.qubits[0])
    ir1 = m.ir()
    assert ir1.startswith("; ModuleID = 'ir_idempotence")
    ir2 = m.ir()
    assert ir1 == ir2


def test_bitcode_idempotence() -> None:
    m = SimpleModule("bitcode_idempotence", num_qubits=1, num_results=0)
    qis = BasicQisBuilder(m.builder)
    qis.x(m.qubits[0])
    bc1 = m.bitcode()
    bc2 = m.bitcode()
    assert bc1 == bc2


def test_ir_gate_ir() -> None:
    m = SimpleModule("ir_gate_ir", num_qubits=1, num_results=0)
    qis = BasicQisBuilder(m.builder)
    qis.x(m.qubits[0])
    ir1 = m.ir()
    assert "call void @__quantum__qis__x__body(%Qubit* null)" in ir1
    qis.h(m.qubits[0])
    ir2 = m.ir()
    assert "call void @__quantum__qis__h__body(%Qubit* null)" in ir2


def test_shared_context() -> None:
    context = Context()

    m1 = SimpleModule("test", 1, 1, context)
    qis1 = BasicQisBuilder(m1.builder)
    qis1.mz(m1.qubits[0], m1.results[0])

    m2 = SimpleModule("test", 1, 1, context)
    qis2 = BasicQisBuilder(m2.builder)
    qis2.mz(m2.qubits[0], m2.results[0])

    assert m1.ir() == m2.ir()


def test_function_lifetime() -> None:
    def make_func() -> Function:
        c = Context()
        m = Module(c, "test")
        return Function(FunctionType(Type.void(c), []), Linkage.EXTERNAL, "f", m)

    assert str(make_func()) == "declare void @f()\n"


def test_instruction_lifetime() -> None:
    def make_inst() -> Instruction:
        m = SimpleModule("test", 1, 1)
        i8 = IntType(m.context, 8)
        f = m.add_external_function("f", FunctionType(i8, []))
        x = m.builder.call(f, [])
        assert x is not None
        v = m.builder.add(x, pyqir.const(i8, 2))
        assert isinstance(v, Instruction)
        return v

    assert str(make_inst()) == "  %1 = add i8 %0, 2"


def test_parsed_function_lifetime() -> None:
    def get_entry() -> Function:
        m = Module.from_bitcode(Path("tests/hello.bc").read_bytes())
        return next(filter(pyqir.is_entry_point, m.functions))

    assert get_entry().name == "program__main"

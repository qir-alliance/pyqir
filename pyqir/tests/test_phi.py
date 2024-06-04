# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os

import pytest
from pathlib import Path

import pyqir


def test_ghz_loop() -> None:
    num_qubits = 10

    context = pyqir.Context()
    builder = pyqir.Builder(context)
    mod = pyqir.qir_module(context, "ghz")
    entry_point = pyqir.entry_point(mod, "kernel", num_qubits, num_qubits)

    header = pyqir.BasicBlock(context, "header", entry_point)
    cond = pyqir.BasicBlock(context, "cond", entry_point)
    body = pyqir.BasicBlock(context, "body", entry_point)
    footer = pyqir.BasicBlock(context, "footer", entry_point)

    builder.insert_at_end(header)
    nullptr = pyqir.Constant.null(pyqir.PointerType(pyqir.IntType(context, 8)))
    pyqir.rt.initialize(builder, nullptr)
    pyqir.qis.h(builder, pyqir.qubit(context, 0))
    builder.br(cond)

    builder.insert_at_end(cond)
    i64 = pyqir.IntType(mod.context, 64)
    phi = builder.phi(i64)
    zero_const = pyqir.const(i64, 0)
    phi.add_incoming(zero_const, header)
    num_qubits_const = pyqir.const(i64, num_qubits)
    ub = builder.sub(num_qubits_const, pyqir.const(i64, 1))
    icmp = builder.icmp(pyqir.IntPredicate.ULT, phi, ub)
    builder.condbr(icmp, body, footer)

    builder.insert_at_end(body)
    one_const = pyqir.const(i64, 1)
    incr = builder.add(phi, one_const)
    pyqir.qis.cx(builder, builder.dyn_qubit(phi), builder.dyn_qubit(incr))
    builder.br(cond)
    phi.add_incoming(incr, body)

    builder.insert_at_end(footer)
    for i in range(num_qubits):
        pyqir.qis.mz(builder, pyqir.qubit(context, i), pyqir.result(context, i))

    pyqir.rt.tuple_record_output(builder, num_qubits_const, nullptr)
    for i in range(num_qubits):
        pyqir.rt.result_record_output(builder, pyqir.result(context, i), nullptr)

    builder.ret(None)
    mod.verify()

    ir = str(mod)
    file = os.path.join(os.path.dirname(__file__), "resources/test_ghz_loop.ll")
    expected = Path(file).read_text()
    assert ir == expected

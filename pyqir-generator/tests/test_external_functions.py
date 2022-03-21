# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, types
import unittest


class ExternalFunctionsTest(unittest.TestCase):
    def test_call_no_params(self) -> None:
        mod = SimpleModule("test", 0, 0)
        f = mod.add_external_function(
            "test_function", types.Function([], types.VOID)
        )
        mod.builder.call(f, [])
        self.assertIn("call void @test_function()", mod.ir())

    def test_call_single_qubit(self) -> None:
        mod = SimpleModule("test", 1, 0)
        f = mod.add_external_function(
            "test_function", types.Function([types.QUBIT], types.VOID)
        )
        mod.builder.call(f, [mod.qubits[0]])
        self.assertIn("call void @test_function(%Qubit* null)", mod.ir())

    def test_call_two_qubits(self) -> None:
        mod = SimpleModule("test", 2, 0)
        f = mod.add_external_function(
            "test_function",
            types.Function([types.QUBIT, types.QUBIT], types.VOID)
        )
        mod.builder.call(f, [mod.qubits[0], mod.qubits[1]])
        self.assertIn(
            "call void @test_function(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))",
            mod.ir(),
        )

    def test_call_float(self) -> None:
        mod = SimpleModule("test", 0, 0)
        f = mod.add_external_function(
            "test_function", types.Function([types.DOUBLE], types.VOID)
        )
        mod.builder.call(f, [23.25])
        self.assertIn(
            "call void @test_function(double 2.325000e+01)", mod.ir()
        )

    def test_call_int(self) -> None:
        mod = SimpleModule("test", 0, 0)
        f = mod.add_external_function(
            "test_function", types.Function([types.INT], types.VOID)
        )
        mod.builder.call(f, [42])
        self.assertIn("call void @test_function(i64 42)", mod.ir())

    def test_call_bool_true(self) -> None:
        mod = SimpleModule("test", 0, 0)
        f = mod.add_external_function(
            "test_function", types.Function([types.BOOL], types.VOID)
        )
        mod.builder.call(f, [True])
        self.assertIn("call void @test_function(i1 true)", mod.ir())

    def test_call_bool_false(self) -> None:
        mod = SimpleModule("test", 0, 0)
        f = mod.add_external_function(
            "test_function", types.Function([types.BOOL], types.VOID)
        )
        mod.builder.call(f, [False])
        self.assertIn("call void @test_function(i1 false)", mod.ir())

    def test_call_single_result(self) -> None:
        mod = SimpleModule("test", 1, 1)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])

        f = mod.add_external_function(
            "test_function", types.Function([types.RESULT], types.VOID)
        )
        mod.builder.call(f, [mod.results[0]])
        self.assertIn("call void @test_function(%Result* %result0)", mod.ir())

    def test_call_two_results(self) -> None:
        mod = SimpleModule("test", 1, 2)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])
        qis.m(mod.qubits[0], mod.results[1])

        f = mod.add_external_function(
            "test_function",
            types.Function([types.RESULT, types.RESULT], types.VOID)
        )
        mod.builder.call(f, [mod.results[0], mod.results[1]])

        self.assertIn(
            "call void @test_function(%Result* %result0, %Result* %result1)",
            mod.ir(),
        )

    def test_call_numbers(self) -> None:
        b = True
        bool_rep = f"i1 true"
        i = 42
        int_rep = f"i64 42"
        d = 42.42
        double_rep = "double 4.242000e+01"

        mod = SimpleModule("test", 0, 0)
        f = mod.add_external_function("test_function", types.Function(
            [types.BOOL, types.INT, types.DOUBLE], types.VOID
        ))
        mod.builder.call(f, [b, i, d])

        self.assertIn(
            f"call void @test_function({bool_rep}, {int_rep}, {double_rep})",
            mod.ir(),
        )

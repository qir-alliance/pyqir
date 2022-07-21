# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, Value, types
from typing import Any, Callable, List, Tuple
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

    def test_call_single_static_result(self) -> None:
        mod = SimpleModule("test", 1, 1)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])

        f = mod.add_external_function(
            "test_function", types.Function([types.RESULT], types.VOID)
        )
        mod.builder.call(f, [mod.results[0]])
        self.assertIn("call void @test_function(%Result* null)", mod.ir())

    def test_call_single_dynamic_result(self) -> None:
        mod = SimpleModule("test", 1, 1)
        mod.use_static_result_alloc(False)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])

        f = mod.add_external_function(
            "test_function", types.Function([types.RESULT], types.VOID)
        )
        mod.builder.call(f, [mod.results[0]])
        self.assertIn("call void @test_function(%Result* %result0)", mod.ir())

    def test_call_two_static_results(self) -> None:
        mod = SimpleModule("test", 1, 2)
        qis = BasicQisBuilder(mod.builder)
        qis.m(mod.qubits[0], mod.results[0])
        qis.m(mod.qubits[0], mod.results[1])

        f = mod.add_external_function(
            "test_function",
            types.Function([types.RESULT, types.RESULT], types.VOID)
        )
        mod.builder.call(f, [mod.results[1], mod.results[0]])

        self.assertIn(
            "call void @test_function(%Result* inttoptr (i64 1 to %Result*), %Result* null)",
            mod.ir(),
        )

    def test_call_two_dynamic_results(self) -> None:
        mod = SimpleModule("test", 1, 2)
        mod.use_static_result_alloc(False)
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

    def test_wrong_type(self) -> None:
        cases: List[Tuple[List[types.Value], Callable[[SimpleModule], List[Any]]]] = [
            ([types.BOOL], lambda _: ["true"]),
            ([types.BOOL], lambda mod: [mod.results[0]]),
            ([types.INT], lambda _: [1.23]),
            ([types.INT], lambda _: ["123"]),
            ([types.DOUBLE], lambda _: ["1.23"]),
            ([types.QUBIT], lambda mod: [mod.results[0]]),
            ([types.RESULT], lambda mod: [mod.qubits[0]]),
        ]

        for param_types, get_args in cases:
            mod = SimpleModule("test", 1, 1)
            args = get_args(mod)

            with self.subTest(repr(args)):
                f = mod.add_external_function(
                    "test_function", types.Function(param_types, types.VOID)
                )

                with self.assertRaises(TypeError):
                    mod.builder.call(f, args)

    def test_overflow(self) -> None:
        cases = [
            [123],
            [2 ** 64],
        ]

        for args in cases:
            with self.subTest(repr(args)):
                mod = SimpleModule("test", 0, 0)
                f = mod.add_external_function(
                    "test_function", types.Function([types.BOOL], types.VOID)
                )

                with self.assertRaises(OverflowError):
                    mod.builder.call(f, args)

    def test_wrong_number_of_args(self) -> None:
        cases: List[List[Value]] = [
            [],
            [1.23],
            [1.23, True, False],
        ]

        for args in cases:
            with self.subTest(repr(args)):
                mod = SimpleModule("test", 0, 0)
                param_types: List[types.Value] = [types.DOUBLE, types.BOOL]
                f = mod.add_external_function(
                    "test_function", types.Function(param_types, types.VOID)
                )

                message = f"Expected {len(param_types)} arguments, got {len(args)}."
                with self.assertRaisesRegex(ValueError, message):
                    mod.builder.call(f, args)

    def test_variable(self) -> None:
        mod = SimpleModule("test", 0, 0)
        foo = mod.add_external_function("foo", types.Function([], types.INT))
        bar = mod.add_external_function(
            "bar", types.Function([types.INT], types.VOID))

        x = mod.builder.call(foo, [])
        mod.builder.call(bar, [x])

        ir = mod.ir()
        self.assertIn("%0 = call i64 @foo()", ir)
        self.assertIn("call void @bar(i64 %0)", ir)

    def test_variable_wrong_external_type(self) -> None:
        mod = SimpleModule("test", 0, 0)
        foo = mod.add_external_function("foo", types.Function([], types.INT))
        bar = mod.add_external_function(
            "bar", types.Function([types.QUBIT], types.VOID))

        x = mod.builder.call(foo, [])
        mod.builder.call(bar, [x])

        # TODO: Should this be a TypeError?
        with self.assertRaisesRegex(OSError, "Call parameter type does not match function signature!"):
            mod.ir()

    def test_variable_wrong_angle_type(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        foo = mod.add_external_function("foo", types.Function([], types.INT))

        x = mod.builder.call(foo, [])
        qis.rz(x, mod.qubits[0])

        # TODO: Should this be a TypeError?
        with self.assertRaisesRegex(OSError, "Call parameter type does not match function signature!"):
            mod.ir()

    def test_two_variables(self) -> None:
        mod = SimpleModule("test", 0, 0)
        foo = mod.add_external_function("foo", types.Function([], types.INT))
        bar = mod.add_external_function(
            "bar", types.Function([types.INT, types.INT], types.VOID))

        x = mod.builder.call(foo, [])
        y = mod.builder.call(foo, [])
        mod.builder.call(bar, [x, y])

        ir = mod.ir()
        self.assertIn("%0 = call i64 @foo()", ir)
        self.assertIn("%1 = call i64 @foo()", ir)
        self.assertIn("call void @bar(i64 %0, i64 %1)", ir)

    def test_computed_rotation(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        foo = mod.add_external_function(
            "foo", types.Function([], types.DOUBLE))

        theta = mod.builder.call(foo, [])
        qis.rx(theta, mod.qubits[0])
        qis.ry(theta, mod.qubits[0])
        qis.rz(theta, mod.qubits[0])

        ir = mod.ir()
        self.assertIn("%0 = call double @foo()", ir)
        self.assertIn(
            "call void @__quantum__qis__rx__body(double %0, %Qubit* null)", ir)
        self.assertIn(
            "call void @__quantum__qis__ry__body(double %0, %Qubit* null)", ir)
        self.assertIn(
            "call void @__quantum__qis__rz__body(double %0, %Qubit* null)", ir)

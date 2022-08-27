# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import BasicQisBuilder, SimpleModule, Type, Value, const, types
import re
from typing import Any, Callable, Union
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

    def test_call_double(self) -> None:
        values: list[Union[Value, float]] = [const(types.DOUBLE, 23.25), 23.25]
        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                f = mod.add_external_function(
                    "test_function", types.Function([types.DOUBLE], types.VOID)
                )
                mod.builder.call(f, [value])
                self.assertIn(
                    "call void @test_function(double 2.325000e+01)", mod.ir()
                )

    def test_call_int(self) -> None:
        values: list[Union[Value, int]] = [const(types.Int(64), 42), 42]
        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                f = mod.add_external_function(
                    "test_function",
                    types.Function([types.Int(64)], types.VOID)
                )
                mod.builder.call(f, [value])
                self.assertIn("call void @test_function(i64 42)", mod.ir())

    def test_call_bool_true(self) -> None:
        values: list[Union[Value, bool]] = [const(types.BOOL, True), True]
        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                f = mod.add_external_function(
                    "test_function", types.Function([types.BOOL], types.VOID)
                )
                mod.builder.call(f, [value])
                self.assertIn("call void @test_function(i1 true)", mod.ir())

    def test_call_bool_false(self) -> None:
        values: list[Union[Value, bool]] = [const(types.BOOL, False), False]
        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                f = mod.add_external_function(
                    "test_function", types.Function([types.BOOL], types.VOID)
                )
                mod.builder.call(f, [value])
                self.assertIn("call void @test_function(i1 false)", mod.ir())

    def test_call_single_result(self) -> None:
        mod = SimpleModule("test", 1, 1)
        qis = BasicQisBuilder(mod.builder)
        qis.mz(mod.qubits[0], mod.results[0])

        f = mod.add_external_function(
            "test_function", types.Function([types.RESULT], types.VOID)
        )
        mod.builder.call(f, [mod.results[0]])
        self.assertIn("call void @test_function(%Result* null)", mod.ir())

    def test_call_two_results(self) -> None:
        mod = SimpleModule("test", 1, 2)
        qis = BasicQisBuilder(mod.builder)
        qis.mz(mod.qubits[0], mod.results[0])
        qis.mz(mod.qubits[0], mod.results[1])

        f = mod.add_external_function(
            "test_function",
            types.Function([types.RESULT, types.RESULT], types.VOID)
        )
        mod.builder.call(f, [mod.results[1], mod.results[0]])

        self.assertIn(
            "call void @test_function(%Result* inttoptr (i64 1 to %Result*), %Result* null)",
            mod.ir(),
        )

    def test_call_numbers(self) -> None:
        b = const(types.BOOL, True)
        bool_rep = f"i1 true"
        i = const(types.Int(64), 42)
        int_rep = f"i64 42"
        d = const(types.DOUBLE, 42.42)
        double_rep = "double 4.242000e+01"

        mod = SimpleModule("test", 0, 0)
        f = mod.add_external_function("test_function", types.Function(
            [types.BOOL, types.Int(64), types.DOUBLE], types.VOID
        ))
        mod.builder.call(f, [b, i, d])

        self.assertIn(
            f"call void @test_function({bool_rep}, {int_rep}, {double_rep})",
            mod.ir(),
        )

    def test_wrong_type_conversion(self) -> None:
        cases: list[tuple[list[Type], Callable[[SimpleModule], list[Any]]]] = [
            ([types.BOOL], lambda _: ["true"]),
            ([types.Int(64)], lambda _: [1.23]),
            ([types.Int(64)], lambda _: ["123"]),
            ([types.DOUBLE], lambda _: ["1.23"]),
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

    def test_overflow_bool_value(self) -> None:
        with self.assertRaises(OverflowError):
            const(types.BOOL, 123)

    def test_overflow_int_value(self) -> None:
        with self.assertRaises(OverflowError):
            const(types.Int(64), 2 ** 64)

    def test_overflow_conversion(self) -> None:
        for value in [123, 2 ** 64]:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                f = mod.add_external_function(
                    "test_function", types.Function([types.BOOL], types.VOID)
                )

                with self.assertRaises(OverflowError):
                    mod.builder.call(f, [value])

    def test_wrong_number_of_args(self) -> None:
        cases: list[list[Value]] = [
            [],
            [const(types.DOUBLE, 1.23)],
            [
                const(types.DOUBLE, 1.23),
                const(types.BOOL, True),
                const(types.BOOL, False),
            ],
        ]

        for args in cases:
            with self.subTest(repr(args)):
                mod = SimpleModule("test", 0, 0)
                param_types: list[Type] = [types.DOUBLE, types.BOOL]
                f = mod.add_external_function(
                    "test_function", types.Function(param_types, types.VOID)
                )

                message = f"Expected {len(param_types)} arguments, got {len(args)}."
                with self.assertRaisesRegex(ValueError, "^" + re.escape(message) + "$"):
                    mod.builder.call(f, args)

    def test_variable(self) -> None:
        mod = SimpleModule("test", 0, 0)
        foo = mod.add_external_function(
            "foo", types.Function([], types.Int(64)))
        bar = mod.add_external_function(
            "bar", types.Function([types.Int(64)], types.VOID))

        x = mod.builder.call(foo, [])
        assert x is not None
        mod.builder.call(bar, [x])

        ir = mod.ir()
        self.assertIn("%0 = call i64 @foo()", ir)
        self.assertIn("call void @bar(i64 %0)", ir)

    def test_variable_wrong_external_type(self) -> None:
        mod = SimpleModule("test", 0, 0)
        foo = mod.add_external_function(
            "foo", types.Function([], types.Int(64)))
        bar = mod.add_external_function(
            "bar", types.Function([types.QUBIT], types.VOID))

        x = mod.builder.call(foo, [])
        assert x is not None
        mod.builder.call(bar, [x])

        with self.assertRaisesRegex(OSError, "^Call parameter type does not match function signature!"):
            mod.ir()

    def test_variable_wrong_angle_type(self) -> None:
        mod = SimpleModule("test", 1, 0)
        qis = BasicQisBuilder(mod.builder)
        foo = mod.add_external_function(
            "foo", types.Function([], types.Int(64)))

        x = mod.builder.call(foo, [])
        assert x is not None
        qis.rz(x, mod.qubits[0])

        with self.assertRaisesRegex(OSError, "^Call parameter type does not match function signature!"):
            mod.ir()

    def test_two_variables(self) -> None:
        mod = SimpleModule("test", 0, 0)
        foo = mod.add_external_function(
            "foo", types.Function([], types.Int(64)))
        bar = mod.add_external_function(
            "bar", types.Function([types.Int(64), types.Int(64)], types.VOID))

        x = mod.builder.call(foo, [])
        assert x is not None
        y = mod.builder.call(foo, [])
        assert y is not None
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
        assert theta is not None
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

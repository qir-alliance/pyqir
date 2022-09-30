# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir.generator import (
    BasicQisBuilder,
    SimpleModule,
    Type,
    TypeFactory,
    Value,
    const,
)
import re
from typing import Any, Callable, List, Tuple, Union
import unittest


class ExternalFunctionsTest(unittest.TestCase):
    def test_call_no_params(self) -> None:
        mod = SimpleModule("test", 0, 0)
        types = mod.types
        f = mod.add_external_function("test_function", types.function(types.void, []))
        mod.builder.call(f, [])
        self.assertIn("call void @test_function()", mod.ir())

    def test_call_single_qubit(self) -> None:
        mod = SimpleModule("test", 1, 0)
        types = mod.types
        f = mod.add_external_function(
            "test_function", types.function(types.void, [types.qubit])
        )
        mod.builder.call(f, [mod.qubits[0]])
        self.assertIn("call void @test_function(%Qubit* null)", mod.ir())

    def test_call_two_qubits(self) -> None:
        mod = SimpleModule("test", 2, 0)
        types = mod.types
        f = mod.add_external_function(
            "test_function",
            types.function(types.void, [types.qubit, types.qubit]),
        )
        mod.builder.call(f, [mod.qubits[0], mod.qubits[1]])
        self.assertIn(
            "call void @test_function(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))",
            mod.ir(),
        )

    def test_call_double(self) -> None:
        values: List[Callable[[TypeFactory], Union[Value, float]]] = [
            lambda types: const(types.double, 23.25),
            lambda _: 23.25,
        ]

        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                types = mod.types
                f = mod.add_external_function(
                    "test_function",
                    types.function(types.void, [types.double]),
                )
                mod.builder.call(f, [value(types)])
                self.assertIn("call void @test_function(double 2.325000e+01)", mod.ir())

    def test_call_int(self) -> None:
        values: List[Callable[[TypeFactory], Union[Value, int]]] = [
            lambda types: const(types.int(64), 42),
            lambda _: 42,
        ]

        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                types = mod.types
                f = mod.add_external_function(
                    "test_function",
                    types.function(types.void, [types.int(64)]),
                )
                mod.builder.call(f, [value(types)])
                self.assertIn("call void @test_function(i64 42)", mod.ir())

    def test_call_bool_true(self) -> None:
        values: List[Callable[[TypeFactory], Union[Value, bool]]] = [
            lambda types: const(types.bool, True),
            lambda _: True,
        ]

        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                types = mod.types
                f = mod.add_external_function(
                    "test_function",
                    types.function(types.void, [types.bool]),
                )
                mod.builder.call(f, [value(types)])
                self.assertIn("call void @test_function(i1 true)", mod.ir())

    def test_call_bool_false(self) -> None:
        values: List[Callable[[TypeFactory], Union[Value, bool]]] = [
            lambda types: const(types.bool, False),
            lambda _: False,
        ]

        for value in values:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                types = mod.types
                f = mod.add_external_function(
                    "test_function",
                    types.function(types.void, [types.bool]),
                )
                mod.builder.call(f, [value(types)])
                self.assertIn("call void @test_function(i1 false)", mod.ir())

    def test_call_single_result(self) -> None:
        mod = SimpleModule("test", 1, 1)
        qis = BasicQisBuilder(mod.builder)
        qis.mz(mod.qubits[0], mod.results[0])

        types = mod.types
        f = mod.add_external_function(
            "test_function", types.function(types.void, [types.result])
        )
        mod.builder.call(f, [mod.results[0]])
        self.assertIn("call void @test_function(%Result* null)", mod.ir())

    def test_call_two_results(self) -> None:
        mod = SimpleModule("test", 1, 2)
        qis = BasicQisBuilder(mod.builder)
        qis.mz(mod.qubits[0], mod.results[0])
        qis.mz(mod.qubits[0], mod.results[1])

        types = mod.types
        f = mod.add_external_function(
            "test_function",
            types.function(types.void, [types.result, types.result]),
        )
        mod.builder.call(f, [mod.results[1], mod.results[0]])

        self.assertIn(
            "call void @test_function(%Result* inttoptr (i64 1 to %Result*), %Result* null)",
            mod.ir(),
        )

    def test_call_numbers(self) -> None:
        mod = SimpleModule("test", 0, 0)
        types = mod.types

        b = const(types.bool, True)
        bool_rep = f"i1 true"
        i = const(types.int(64), 42)
        int_rep = f"i64 42"
        d = const(types.double, 42.42)
        double_rep = "double 4.242000e+01"

        f = mod.add_external_function(
            "test_function",
            types.function(
                types.void,
                [types.bool, types.int(64), types.double],
            ),
        )
        mod.builder.call(f, [b, i, d])

        self.assertIn(
            f"call void @test_function({bool_rep}, {int_rep}, {double_rep})",
            mod.ir(),
        )

    def test_wrong_type_conversion(self) -> None:
        cases: List[Tuple[Callable[[TypeFactory], List[Type]], List[Any]]] = [
            (lambda types: [types.bool], ["true"]),
            (lambda types: [types.int(64)], [1.23]),
            (lambda types: [types.int(64)], ["123"]),
            (lambda types: [types.double], ["1.23"]),
        ]

        for param_types, args in cases:
            mod = SimpleModule("test", 1, 1)
            types = mod.types

            with self.subTest(repr(args)):
                f = mod.add_external_function(
                    "test_function",
                    types.function(types.void, param_types(types)),
                )

                with self.assertRaises(TypeError):
                    mod.builder.call(f, args)

    def test_overflow_bool_value(self) -> None:
        mod = SimpleModule("test", 0, 0)
        with self.assertRaises(OverflowError):
            const(mod.types.bool, 123)

    def test_overflow_int_value(self) -> None:
        mod = SimpleModule("test", 0, 0)
        with self.assertRaises(OverflowError):
            const(mod.types.int(64), 2**64)

    def test_overflow_conversion(self) -> None:
        for value in [123, 2**64]:
            with self.subTest(repr(value)):
                mod = SimpleModule("test", 0, 0)
                types = mod.types
                f = mod.add_external_function(
                    "test_function",
                    types.function(types.void, [types.bool]),
                )

                with self.assertRaises(OverflowError):
                    mod.builder.call(f, [value])

    def test_wrong_number_of_args(self) -> None:
        cases: List[Callable[[TypeFactory], List[Value]]] = [
            lambda _: [],
            lambda types: [const(types.double, 1.23)],
            lambda types: [
                const(types.double, 1.23),
                const(types.bool, True),
                const(types.bool, False),
            ],
        ]

        for get_args in cases:
            with self.subTest(repr(get_args)):
                mod = SimpleModule("test", 0, 0)
                types = mod.types
                args = get_args(types)
                param_types: List[Type] = [types.double, types.bool]
                f = mod.add_external_function(
                    "test_function",
                    types.function(types.void, param_types),
                )

                message = f"Expected {len(param_types)} arguments, got {len(args)}."
                with self.assertRaisesRegex(ValueError, "^" + re.escape(message) + "$"):
                    mod.builder.call(f, args)

    def test_variable(self) -> None:
        mod = SimpleModule("test", 0, 0)
        types = mod.types
        foo = mod.add_external_function("foo", types.function(types.int(64), []))
        bar = mod.add_external_function(
            "bar",
            types.function(types.void, [types.int(64)]),
        )

        x = mod.builder.call(foo, [])
        assert x is not None
        mod.builder.call(bar, [x])

        ir = mod.ir()
        self.assertIn("%0 = call i64 @foo()", ir)
        self.assertIn("call void @bar(i64 %0)", ir)

    def test_variable_wrong_external_type(self) -> None:
        mod = SimpleModule("test", 0, 0)
        types = mod.types
        foo = mod.add_external_function("foo", types.function(types.int(64), []))
        bar = mod.add_external_function(
            "bar", types.function(types.void, [types.qubit])
        )

        x = mod.builder.call(foo, [])
        assert x is not None
        mod.builder.call(bar, [x])

        with self.assertRaisesRegex(
            OSError, "^Call parameter type does not match function signature!"
        ):
            mod.ir()

    def test_variable_wrong_angle_type(self) -> None:
        mod = SimpleModule("test", 1, 0)
        types = mod.types
        qis = BasicQisBuilder(mod.builder)
        foo = mod.add_external_function("foo", types.function(types.int(64), []))

        x = mod.builder.call(foo, [])
        assert x is not None
        with self.assertRaisesRegex(BaseException, "^Found IntValue"):
            qis.rz(x, mod.qubits[0])

    def test_two_variables(self) -> None:
        mod = SimpleModule("test", 0, 0)
        types = mod.types
        foo = mod.add_external_function("foo", types.function(types.int(64), []))
        bar = mod.add_external_function(
            "bar",
            types.function(
                types.void,
                [types.int(64), types.int(64)],
            ),
        )

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
        types = mod.types
        qis = BasicQisBuilder(mod.builder)
        foo = mod.add_external_function("foo", types.function(types.double, []))

        theta = mod.builder.call(foo, [])
        assert theta is not None
        qis.rx(theta, mod.qubits[0])
        qis.ry(theta, mod.qubits[0])
        qis.rz(theta, mod.qubits[0])

        ir = mod.ir()
        self.assertIn("%0 = call double @foo()", ir)
        self.assertIn(
            "call void @__quantum__qis__rx__body(double %0, %Qubit* null)", ir
        )
        self.assertIn(
            "call void @__quantum__qis__ry__body(double %0, %Qubit* null)", ir
        )
        self.assertIn(
            "call void @__quantum__qis__rz__body(double %0, %Qubit* null)", ir
        )

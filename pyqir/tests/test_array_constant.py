# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    ArrayConstant,
    ArrayType,
    Context,
    FloatConstant,
    GlobalVariable,
    IntConstant,
    IntType,
    Module,
    PointerType,
)


def test_global_constant_data_array_i32() -> None:
    """Introspect a global constant array of i32 values."""
    ir = """
    @my_ints = constant [3 x i32] [i32 10, i32 20, i32 30]
    """
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert isinstance(gv, GlobalVariable)
    assert gv.name == "my_ints"
    assert gv.is_constant is True

    arr = gv.initializer
    assert arr is not None
    assert isinstance(arr, ArrayConstant)
    assert arr.count == 3

    elements = arr.elements
    assert len(elements) == 3
    assert all(isinstance(e, IntConstant) for e in elements)
    assert [e.value for e in elements] == [10, 20, 30]

    arr_ty = arr.type
    assert isinstance(arr_ty, ArrayType)
    assert arr_ty.count == 3
    elem_ty = arr_ty.element
    assert isinstance(elem_ty, IntType)
    assert elem_ty.width == 32


def test_global_constant_data_array_double() -> None:
    """Introspect a global constant array of double values."""
    ir = """
    @my_doubles = constant [2 x double] [double 1.5, double 2.75]
    """
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert isinstance(gv, GlobalVariable)
    assert gv.is_constant is True

    arr = gv.initializer
    assert arr is not None
    assert isinstance(arr, ArrayConstant)
    assert arr.count == 2

    elements = arr.elements
    assert len(elements) == 2
    assert all(isinstance(e, FloatConstant) for e in elements)
    assert elements[0].value == 1.5
    assert elements[1].value == 2.75


def test_global_constant_array_of_arrays() -> None:
    """Introspect a global constant containing nested arrays."""
    ir = """
    @matrix = constant [2 x [2 x i32]] [
      [2 x i32] [i32 1, i32 2],
      [2 x i32] [i32 3, i32 4]
    ]
    """
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert isinstance(gv, GlobalVariable)

    outer = gv.initializer
    assert outer is not None
    assert isinstance(outer, ArrayConstant)
    assert outer.count == 2

    inner_elements = outer.elements
    assert len(inner_elements) == 2
    assert all(isinstance(e, ArrayConstant) for e in inner_elements)

    first_row = inner_elements[0]
    assert isinstance(first_row, ArrayConstant)
    assert first_row.count == 2
    assert [e.value for e in first_row.elements] == [1, 2]

    second_row = inner_elements[1]
    assert isinstance(second_row, ArrayConstant)
    assert [e.value for e in second_row.elements] == [3, 4]


def test_global_variable_without_initializer() -> None:
    """A global declaration without an initializer returns None."""
    ir = """
    @extern_arr = external global [4 x i32]
    """
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert isinstance(gv, GlobalVariable)
    assert gv.initializer is None
    assert gv.is_constant is False


def test_global_mutable_array() -> None:
    """A mutable global (not constant) still exposes its initializer."""
    ir = """
    @mutable_arr = global [3 x i32] [i32 7, i32 8, i32 9]
    """
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert isinstance(gv, GlobalVariable)
    assert gv.is_constant is False

    arr = gv.initializer
    assert arr is not None
    assert isinstance(arr, ArrayConstant)
    assert arr.count == 3
    assert all(isinstance(e, IntConstant) for e in arr.elements)
    assert [e.value for e in arr.elements] == [7, 8, 9]


def test_global_array_of_pointers_to_arrays() -> None:
    """Introspect a 2D array expressed as an array of pointers to row arrays."""
    ir = """
    @row0 = constant [3 x i32] [i32 10, i32 20, i32 30]
    @row1 = constant [3 x i32] [i32 40, i32 50, i32 60]
    @matrix = constant [2 x ptr] [ptr @row0, ptr @row1]
    """
    mod = Module.from_ir(Context(), ir, "test")
    matrix_gv = next(gv for gv in mod.global_variables if gv.name == "matrix")
    assert isinstance(matrix_gv, GlobalVariable)
    assert matrix_gv.is_constant is True

    arr = matrix_gv.initializer
    assert arr is not None
    assert isinstance(arr, ArrayConstant)
    assert arr.count == 2

    arr_ty = arr.type
    assert isinstance(arr_ty, ArrayType)
    assert arr_ty.count == 2
    assert isinstance(arr_ty.element, PointerType)

    # Each element is a GlobalVariable pointing to a row array.
    elements = arr.elements
    assert len(elements) == 2
    assert all(isinstance(e, GlobalVariable) for e in elements)
    assert elements[0].name == "row0"
    assert elements[1].name == "row1"

    # Follow the pointers to introspect the underlying row data.
    row0_init = elements[0].initializer
    assert row0_init is not None
    assert isinstance(row0_init, ArrayConstant)
    assert [e.value for e in row0_init.elements] == [10, 20, 30]

    row1_init = elements[1].initializer
    assert row1_init is not None
    assert isinstance(row1_init, ArrayConstant)
    assert [e.value for e in row1_init.elements] == [40, 50, 60]

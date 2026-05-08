# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from pyqir import (
    ArrayConstant,
    ArrayType,
    Constant,
    Context,
    FloatConstant,
    GlobalVariable,
    IntConstant,
    IntType,
    Module,
    PointerType,
)


def test_empty_module_has_no_globals() -> None:
    mod = Module.from_ir(Context(), "", "test")
    assert mod.global_variables == []


def test_single_global_variable() -> None:
    ir = "@x = global i32 42"
    mod = Module.from_ir(Context(), ir, "test")
    assert len(mod.global_variables) == 1
    gv = mod.global_variables[0]
    assert isinstance(gv, GlobalVariable)
    assert gv.name == "x"


def test_multiple_global_variables() -> None:
    ir = """
    @a = global i32 1
    @b = global i32 2
    @c = global i32 3
    """
    mod = Module.from_ir(Context(), ir, "test")
    assert len(mod.global_variables) == 3
    names = [gv.name for gv in mod.global_variables]
    assert names == ["a", "b", "c"]


def test_global_variables_are_global_variable_instances() -> None:
    ir = """
    @x = global i32 0
    @y = constant double 1.0
    """
    mod = Module.from_ir(Context(), ir, "test")
    for gv in mod.global_variables:
        assert isinstance(gv, GlobalVariable)
        assert isinstance(gv, Constant)


def test_global_variable_with_int_initializer() -> None:
    ir = "@val = constant i32 99"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    init = gv.initializer
    assert init is not None
    assert isinstance(init, IntConstant)
    assert init.value == 99


def test_global_variable_with_float_initializer() -> None:
    ir = "@pi = constant double 3.14159"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    init = gv.initializer
    assert init is not None
    assert isinstance(init, FloatConstant)
    assert abs(init.value - 3.14159) < 1e-10


def test_global_variable_without_initializer() -> None:
    ir = "@ext = external global i32"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert gv.initializer is None


def test_is_constant_true() -> None:
    ir = "@c = constant i32 5"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert gv.is_constant is True


def test_is_constant_false() -> None:
    ir = "@m = global i32 5"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert gv.is_constant is False


def test_global_variable_name() -> None:
    ir = "@my_special_name = global i32 0"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    assert gv.name == "my_special_name"


def test_global_variable_type_is_pointer() -> None:
    ir = "@val = global i32 10"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    # Global variables are pointers to their content.
    assert isinstance(gv.type, PointerType)


def test_global_variable_initializer_type() -> None:
    ir = "@val = global i64 123"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    init = gv.initializer
    assert init is not None
    assert isinstance(init.type, IntType)
    assert init.type.width == 64


def test_globals_coexist_with_functions() -> None:
    ir = """
    @g = global i32 1

    define void @f() {
      ret void
    }
    """
    mod = Module.from_ir(Context(), ir, "test")
    assert len(mod.global_variables) == 1
    assert mod.global_variables[0].name == "g"
    assert len(mod.functions) == 1
    assert mod.functions[0].name == "f"


def test_globals_do_not_include_functions() -> None:
    ir = """
    @g1 = global i32 0
    @g2 = global i32 0

    define void @func() {
      ret void
    }

    declare void @decl()
    """
    mod = Module.from_ir(Context(), ir, "test")
    gv_names = [gv.name for gv in mod.global_variables]
    assert gv_names == ["g1", "g2"]


def test_global_array_type() -> None:
    ir = "@arr = constant [4 x i32] [i32 1, i32 2, i32 3, i32 4]"
    mod = Module.from_ir(Context(), ir, "test")
    gv = mod.global_variables[0]
    init = gv.initializer
    assert init is not None
    arr_ty = init.type
    assert isinstance(arr_ty, ArrayType)
    assert arr_ty.count == 4
    assert isinstance(arr_ty.element, IntType)
    assert arr_ty.element.width == 32


def test_multiple_globals_mixed_types() -> None:
    ir = """
    @an_int = constant i32 42
    @a_float = constant double 2.718
    @an_array = constant [2 x i32] [i32 5, i32 6]
    @external = external global i64
    """
    mod = Module.from_ir(Context(), ir, "test")
    assert len(mod.global_variables) == 4

    gv_int = mod.global_variables[0]
    assert gv_int.name == "an_int"
    assert gv_int.is_constant is True
    assert isinstance(gv_int.initializer, IntConstant)
    assert gv_int.initializer.value == 42

    gv_float = mod.global_variables[1]
    assert gv_float.name == "a_float"
    assert isinstance(gv_float.initializer, FloatConstant)
    assert abs(gv_float.initializer.value - 2.718) < 1e-10

    gv_array = mod.global_variables[2]
    assert gv_array.name == "an_array"
    assert isinstance(gv_array.initializer, ArrayConstant)

    gv_ext = mod.global_variables[3]
    assert gv_ext.name == "external"
    assert gv_ext.initializer is None
    assert gv_ext.is_constant is False


def test_global_with_internal_linkage() -> None:
    ir = '@.str = private constant [6 x i8] c"hello\\00"'
    mod = Module.from_ir(Context(), ir, "test")
    assert len(mod.global_variables) == 1
    gv = mod.global_variables[0]
    assert gv.name == ".str"
    assert gv.is_constant is True
    assert gv.initializer is not None

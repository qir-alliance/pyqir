use super::calls;
use inkwell::values::{BasicMetadataValueEnum, IntValue, PointerValue};
use qirlib::codegen::CodeGenerator;

pub(crate) fn get_one<'a>(generator: &CodeGenerator<'a>) -> PointerValue<'a> {
    calls::emit_call_with_return(
        &generator.builder,
        generator.runtime_library.result_get_one,
        &[],
        "one",
    )
    .into_pointer_value()
}

pub(crate) fn equals<'a>(
    generator: &CodeGenerator<'a>,
    x: PointerValue<'a>,
    y: PointerValue<'a>,
) -> IntValue<'a> {
    let x = BasicMetadataValueEnum::PointerValue(x);
    let y = BasicMetadataValueEnum::PointerValue(y);

    calls::emit_call_with_return(
        &generator.builder,
        generator.runtime_library.result_equal,
        &[x, y],
        "",
    )
    .into_int_value()
}

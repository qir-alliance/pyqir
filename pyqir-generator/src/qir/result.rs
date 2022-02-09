use inkwell::values::{BasicMetadataValueEnum, IntValue, PointerValue};
use qirlib::codegen::CodeGenerator;

pub(crate) fn get_one<'a>(generator: &CodeGenerator<'a>) -> PointerValue<'a> {
    generator
        .emit_call_with_return(generator.rt_result_get_one(), &[], "one")
        .into_pointer_value()
}

pub(crate) fn equal<'a>(
    generator: &CodeGenerator<'a>,
    result1: PointerValue<'a>,
    result2: PointerValue<'a>,
) -> IntValue<'a> {
    let result1 = BasicMetadataValueEnum::PointerValue(result1);
    let result2 = BasicMetadataValueEnum::PointerValue(result2);
    generator
        .emit_call_with_return(generator.rt_result_equal(), &[result1, result2], "equal")
        .into_int_value()
}

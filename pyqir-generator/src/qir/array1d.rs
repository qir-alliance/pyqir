// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::values::{BasicValue, BasicValueEnum};
use inkwell::AddressSpace;

use crate::qir::basic_values;

use qirlib::codegen::CodeGenerator;

use super::calls;

pub(crate) fn emit_array_1d<'ctx>(
    generator: &CodeGenerator<'ctx>,
    name: &str,
    size: u64,
) -> (BasicValueEnum<'ctx>, Vec<(u64, BasicValueEnum<'ctx>)>) {
    let sub_result_name = name.to_string();
    let sub_result = emit_array_allocate1d(generator, 8, size, sub_result_name.as_str());
    let mut items = vec![];
    for index in 0..size {
        let cast = get_bitcast_result_pointer_array_element(
            generator,
            index,
            &sub_result,
            sub_result_name.as_str(),
        );
        items.push((index, cast));
        let zero = generator
            .builder
            .build_call(
                generator.runtime_library.result_get_zero,
                &[],
                format!("zero_{}", index).as_str(),
            )
            .try_as_basic_value()
            .left()
            .unwrap();
        let one = basic_values::u64_to_i32(generator, 1);
        generator.builder.build_call(
            generator.runtime_library.result_update_reference_count,
            &[zero.into(), one],
            name,
        );
        generator
            .builder
            .build_store(cast.into_pointer_value(), zero);
    }

    (sub_result, items)
}

fn get_bitcast_array_pointer_element<'ctx>(
    generator: &CodeGenerator<'ctx>,
    index: u64,
    sub_result: &BasicValueEnum<'ctx>,
    sub_result_name: &str,
) -> BasicValueEnum<'ctx> {
    let element_raw_ptr_name = format!("{}_{}_raw", sub_result_name, index);
    let sub_result_element_ptr = emit_array_get_element_ptr_1d(
        generator,
        index,
        sub_result.as_basic_value_enum(),
        element_raw_ptr_name.as_str(),
    );

    let element_result_ptr_name = format!("{}_result_{}", sub_result_name, index);
    let target_type = generator.types.array.ptr_type(AddressSpace::Generic);
    let cast = generator.builder.build_bitcast(
        sub_result_element_ptr,
        target_type,
        element_result_ptr_name.as_str(),
    );
    cast
}

fn get_bitcast_qubit_pointer_element<'ctx>(
    generator: &CodeGenerator<'ctx>,
    i: u64,
    sub_result: &BasicValueEnum<'ctx>,
    sub_result_name: &str,
) -> BasicValueEnum<'ctx> {
    let element_raw_ptr_name = format!("{}_{}_raw", sub_result_name, i);
    let sub_result_element_ptr = emit_array_get_element_ptr_1d(
        generator,
        i,
        sub_result.as_basic_value_enum(),
        element_raw_ptr_name.as_str(),
    );

    let element_result_ptr_name = format!("{}_result_{}", sub_result_name, i);
    let target_type = generator.types.qubit;
    let cast = generator.builder.build_bitcast(
        sub_result_element_ptr,
        target_type.ptr_type(AddressSpace::Generic),
        element_result_ptr_name.as_str(),
    );
    cast
}

pub fn get_bitcast_result_pointer_array_element<'ctx>(
    generator: &CodeGenerator<'ctx>,
    index: u64,
    sub_result: &BasicValueEnum<'ctx>,
    sub_result_name: &str,
) -> BasicValueEnum<'ctx> {
    let element_raw_ptr_name = format!("{}_{}_raw", sub_result_name, index);
    let sub_result_element_ptr = emit_array_get_element_ptr_1d(
        generator,
        index,
        sub_result.as_basic_value_enum(),
        element_raw_ptr_name.as_str(),
    );

    let element_result_ptr_name = format!("{}_result_{}", sub_result_name, index);
    let target_type = generator.types.result.ptr_type(AddressSpace::Generic);
    let cast = generator.builder.build_bitcast(
        sub_result_element_ptr,
        target_type,
        element_result_ptr_name.as_str(),
    );
    cast
}

pub(crate) fn emit_empty_result_array_allocate1d<'ctx>(
    generator: &CodeGenerator<'ctx>,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    emit_array_allocate1d(generator, 8, 0, result_name)
}

pub(crate) fn emit_array_allocate1d<'ctx>(
    generator: &CodeGenerator<'ctx>,
    bits: u64,
    length: u64,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    let args = &[
        basic_values::u64_to_i32(generator, bits),
        basic_values::u64_to_i64(generator, length),
    ];
    calls::emit_call_with_return(
        &generator.builder,
        generator.runtime_library.array_create_1d,
        args,
        result_name,
    )
}

pub(crate) fn emit_array_get_element_ptr_1d<'ctx>(
    generator: &CodeGenerator<'ctx>,
    index: u64,
    target: BasicValueEnum<'ctx>,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    let args = &[target.into(), basic_values::u64_to_i64(generator, index)];
    let value = generator
        .builder
        .build_call(
            generator.runtime_library.array_get_element_ptr_1d,
            args,
            result_name,
        )
        .try_as_basic_value();
    value.left().unwrap()
}

pub(crate) fn set_elements<'ctx>(
    generator: &CodeGenerator<'ctx>,
    results: &BasicValueEnum<'ctx>,
    sub_results: &[BasicValueEnum<'ctx>],
    name: &str,
) {
    for (index, _) in sub_results.iter().enumerate() {
        let result_indexed_name = format!("{}_result_tmp", name);
        let result_indexed = get_bitcast_array_pointer_element(
            generator,
            index as u64,
            results,
            result_indexed_name.as_str(),
        );

        let _ = generator
            .builder
            .build_store(result_indexed.into_pointer_value(), sub_results[index]);
    }
}

pub(crate) fn create_ctl_wrapper<'ctx>(
    generator: &CodeGenerator<'ctx>,
    control_qubit: &BasicValueEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    let name = String::from("__controlQubits__");
    let control_qubits = emit_array_allocate1d(generator, 8, 1, &name[..]);
    wrap_value_in_array(
        generator,
        &control_qubits,
        control_qubit,
        format!("{}{}", name, 0).as_str(),
    );
    control_qubits
}

pub(crate) fn wrap_value_in_array<'ctx>(
    generator: &CodeGenerator<'ctx>,
    results: &BasicValueEnum<'ctx>,
    sub_results: &BasicValueEnum<'ctx>,
    name: &str,
) {
    let result_indexed_name = format!("{}_result_tmp", name);
    let result_indexed =
        get_bitcast_qubit_pointer_element(generator, 0, results, result_indexed_name.as_str());

    let _ = generator.builder.build_store(
        result_indexed.into_pointer_value(),
        sub_results.as_basic_value_enum(),
    );
}

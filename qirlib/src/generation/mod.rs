// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{context::Context, memory_buffer::MemoryBuffer};

use crate::module;

pub mod emit;
pub mod qir;

/// # Errors
///
/// Will return `Err` if a module cannot be created from the supplied IR
pub fn ir_to_bitcode(
    value: &str,
    module_name: &Option<String>,
    source_file_name: &Option<String>,
) -> Result<Vec<u8>, String> {
    let context = Context::create();
    let bytes = value.as_bytes();
    let buffer_name = match module_name {
        Some(name) => name.as_str(),
        None => "",
    };
    let memory_buffer = MemoryBuffer::create_from_memory_range_copy(bytes, buffer_name);
    let module = context
        .create_module_from_ir(memory_buffer)
        .map_err(|err| err.to_string())?;

    if let Some(source_name) = source_file_name {
        module.set_source_file_name(source_name.as_str());
    }

    let bitcode = module.write_bitcode_to_memory().as_slice().to_owned();
    Ok(bitcode)
}

/// # Errors
///
/// Will return `Err` if a module cannot be created from the supplied bitcode
pub fn bitcode_to_ir(
    value: &[u8],
    module_name: &Option<String>,
    source_file_name: &Option<String>,
) -> Result<String, String> {
    let context = Context::create();
    let buffer_name = match module_name.as_ref() {
        Some(name) => name.as_str(),
        None => "",
    };
    let module = module::load_memory(value, buffer_name, &context)?;

    if let Some(source_name) = source_file_name.as_ref() {
        module.set_source_file_name(source_name.as_str());
    }

    let ir = module.print_to_string().to_string();

    Ok(ir)
}

#[cfg(test)]
mod module_conversion_tests {
    use super::{qir::init_module_builder, *};
    use crate::codegen::{
        qis,
        types::{qubit_id, result_id},
        BuilderRef,
    };

    fn example_ir() -> String {
        let context = Context::create();
        let module = context.create_module("test");
        let builder = context.create_builder();
        let builder = BuilderRef::new(&builder, &module);
        init_module_builder(&module, &builder);
        qis::call_mz(
            builder,
            qubit_id(builder, 0).into(),
            result_id(builder, 0).into(),
        );
        builder.build_return(None);
        module.print_to_string().to_string()
    }

    #[test]
    fn ir_round_trip_is_identical() -> Result<(), String> {
        let actual_ir = example_ir();
        let bitcode = ir_to_bitcode(actual_ir.as_str(), &None, &None)?;
        let converted_ir = bitcode_to_ir(
            bitcode.as_slice(),
            &Some("test".to_owned()),
            &Some("test".to_owned()),
        )?;
        assert_eq!(actual_ir, converted_ir);
        Ok(())
    }
}

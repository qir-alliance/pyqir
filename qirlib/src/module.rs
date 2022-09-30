// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    attributes::AttributeLoc, context::Context, memory_buffer::MemoryBuffer, module::Module,
    values::FunctionValue,
};
use std::{ffi::OsStr, path::Path};

/// # Errors
///
/// - Path has an unsupported extension.
/// - Module fails to load.
pub fn load_file(path: impl AsRef<Path>, context: &Context) -> Result<Module, String> {
    let path = path.as_ref();
    let extension = path.extension().and_then(OsStr::to_str);

    match extension {
        Some("ll") => MemoryBuffer::create_from_file(path)
            .and_then(|buffer| context.create_module_from_ir(buffer))
            .map_err(|e| e.to_string()),
        Some("bc") => Module::parse_bitcode_from_path(path, context).map_err(|e| e.to_string()),
        _ => Err(format!("Unsupported file extension '{:?}'.", extension)),
    }
}

/// # Errors
///
/// - Module fails to load.
pub fn load_memory<'a>(
    bytes: &[u8],
    name: &str,
    context: &'a Context,
) -> Result<Module<'a>, String> {
    let buffer = MemoryBuffer::create_from_memory_range_copy(bytes, name);
    Module::parse_bitcode_from_buffer(&buffer, context).map_err(|e| e.to_string())
}

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
    let module = load_memory(value, buffer_name, &context)?;

    if let Some(source_name) = source_file_name.as_ref() {
        module.set_source_file_name(source_name.as_str());
    }

    let ir = module.print_to_string().to_string();

    Ok(ir)
}

pub(crate) fn create_entry_point<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let context = module.get_context();
    let fn_type = context.void_type().fn_type(&[], false);
    let fn_value = module.add_function("main", fn_type, None);

    let entry_point_attribute = context.create_string_attribute("EntryPoint", "");
    fn_value.add_attribute(AttributeLoc::Function, entry_point_attribute);
    fn_value
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::Path};

    use crate::{
        build::{self, BuilderRef},
        module::{self, create_entry_point},
        qis,
        types::{qubit_id, result_id},
    };
    use inkwell::context::Context;
    use tempfile::tempdir;

    #[test]
    fn entry_point_function_has_correct_signature_and_default_attribute() {
        let context = Context::create();
        let module = context.create_module("test");
        let builder = context.create_builder();

        let entry_point = create_entry_point(&module);
        let entry = context.append_basic_block(entry_point, "entry");
        builder.position_at_end(entry);
        builder.build_return(None);
        let ir_string = module.print_to_string().to_string();
        let expected = "; ModuleID = 'test'\nsource_filename = \"test\"\n\ndefine void @main() #0 {\nentry:\n  ret void\n}\n\nattributes #0 = { \"EntryPoint\" }\n";
        assert_eq!(expected, ir_string);
    }

    fn example_ir() -> String {
        let context = Context::create();
        let module = context.create_module("test");
        let builder = context.create_builder();
        let builder = BuilderRef::new(&builder, &module);
        build::init(builder);
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
        let bitcode = module::ir_to_bitcode(actual_ir.as_str(), &None, &None)?;
        let converted_ir = module::bitcode_to_ir(
            bitcode.as_slice(),
            &Some("test".to_owned()),
            &Some("test".to_owned()),
        )?;
        assert_eq!(actual_ir, converted_ir);
        Ok(())
    }

    #[test]
    fn emitted_bitcode_files_are_identical_to_base64_encoded() {
        let dir = tempdir().expect("");
        let tmp_path = dir.into_path();
        let name = "test";
        let file_path = tmp_path.join(format!("{}.bc", name));
        let file_path_string = file_path.display().to_string();

        let context = Context::create();
        let module = context.create_module(name);
        module.write_bitcode_to_path(Path::new(&file_path_string));

        let mut emitted_bitcode_file =
            File::open(file_path_string.as_str()).expect("Could not open emitted bitcode file");
        let mut emitted_bitcode_bytes = vec![];
        emitted_bitcode_file
            .read_to_end(&mut emitted_bitcode_bytes)
            .expect("Could not read emitted bitcode file");

        let decoded_bitcode_bytes = module.write_bitcode_to_memory();

        assert_eq!(
            emitted_bitcode_bytes.as_slice(),
            decoded_bitcode_bytes.as_slice()
        );
    }
}

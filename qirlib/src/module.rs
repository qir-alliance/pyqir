// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{
    attributes::AttributeLoc,
    builder::Builder,
    context::Context,
    memory_buffer::MemoryBuffer,
    module::Module,
    passes::{PassManager, PassManagerBuilder},
    values::FunctionValue,
    OptimizationLevel,
};

/// # Errors
///
/// Will return `Err` if a module cannot be created from the supplied IR
pub fn ir_to_bitcode(
    value: &str,
    module_name: Option<&str>,
    source_file_name: Option<&str>,
) -> Result<Vec<u8>, String> {
    let context = Context::create();
    let buffer = MemoryBuffer::create_from_memory_range_copy(
        value.as_bytes(),
        module_name.unwrap_or_default(),
    );
    let module = context
        .create_module_from_ir(buffer)
        .map_err(|e| e.to_string())?;
    if let Some(name) = source_file_name {
        module.set_source_file_name(name);
    }
    Ok(module.write_bitcode_to_memory().as_slice().to_owned())
}

/// # Errors
///
/// Will return `Err` if a module cannot be created from the supplied bitcode
pub fn bitcode_to_ir(
    value: &[u8],
    module_name: Option<&str>,
    source_file_name: Option<&str>,
) -> Result<String, String> {
    let context = Context::create();
    let module = load_memory(value, module_name.unwrap_or_default(), &context)?;
    if let Some(name) = source_file_name {
        module.set_source_file_name(name);
    }
    Ok(module.print_to_string().to_string())
}

pub fn simple_init(
    module: &Module,
    builder: &Builder,
    required_num_qubits: u64,
    required_num_results: u64,
) {
    let context = module.get_context();
    let entry_point = create_entry_point(module);
    add_num_attribute(entry_point, "requiredQubits", required_num_qubits);
    add_num_attribute(entry_point, "requiredResults", required_num_results);
    let entry = context.append_basic_block(entry_point, "entry");
    builder.position_at_end(entry);
}

#[allow(clippy::missing_errors_doc)]
pub fn simple_finalize(module: &Module) -> Result<(), String> {
    run_basic_passes(module);
    module.verify().map_err(|e| e.to_string())
}

/// # Errors
///
/// - Module fails to load.
pub(crate) fn load_memory<'a>(
    bytes: &[u8],
    name: &str,
    context: &'a Context,
) -> Result<Module<'a>, String> {
    let buffer = MemoryBuffer::create_from_memory_range_copy(bytes, name);
    Module::parse_bitcode_from_buffer(&buffer, context).map_err(|e| e.to_string())
}

// This method returns true if any of the passes modified the function or module and false otherwise.
pub(crate) fn run_basic_passes(module: &Module) -> bool {
    let pass_manager_builder = PassManagerBuilder::create();
    pass_manager_builder.set_optimization_level(OptimizationLevel::None);
    let fpm = PassManager::create(());
    fpm.add_global_dce_pass();
    fpm.add_strip_dead_prototypes_pass();
    pass_manager_builder.populate_module_pass_manager(&fpm);
    fpm.run_on(module)
}

fn create_entry_point<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let context = module.get_context();
    let fn_type = context.void_type().fn_type(&[], false);
    let fn_value = module.add_function("main", fn_type, None);

    let entry_point_attribute = context.create_string_attribute("EntryPoint", "");
    fn_value.add_attribute(AttributeLoc::Function, entry_point_attribute);
    fn_value
}

fn add_num_attribute(function: FunctionValue, key: &str, value: u64) {
    let context = function.get_type().get_context();
    let attribute = context.create_string_attribute(key, &value.to_string());
    function.add_attribute(AttributeLoc::Function, attribute);
}

#[cfg(test)]
mod tests {
    use crate::{
        builder::Builder,
        module::{self, create_entry_point},
        qis::BuilderBasicQisExt,
    };
    use inkwell::context::Context;
    use std::{fs::File, io::Read, path::Path};
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
        let builder = Builder::new(&module);
        super::simple_init(&module, &builder, 1, 1);
        builder.build_mz(builder.build_qubit(0), builder.build_result(0));
        builder.build_return(None);
        module.print_to_string().to_string()
    }

    #[test]
    fn ir_round_trip_is_identical() -> Result<(), String> {
        let actual_ir = example_ir();
        let bitcode = module::ir_to_bitcode(actual_ir.as_str(), None, None)?;
        let converted_ir = module::bitcode_to_ir(bitcode.as_slice(), Some("test"), Some("test"))?;
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

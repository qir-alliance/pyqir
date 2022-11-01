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

pub fn simple_init(
    module: &Module,
    builder: &Builder,
    required_num_qubits: u64,
    required_num_results: u64,
    name: Option<&str>,
) {
    let context = module.get_context();
    let entry_point = create_entry_point(module, name);
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

pub fn link<'ctx>(
    context: &'ctx Context,
    modules: Vec<(&[u8], String)>,
    name: Option<&str>,
) -> Result<(Module<'ctx>, Vec<String>), String> {
    let module: Module<'ctx> = context.create_module(name.unwrap_or("default"));
    let mut names = vec![];
    for (bytes, n) in modules {
        let buffer = MemoryBuffer::create_from_memory_range_copy(bytes, n.as_str());
        let m = Module::parse_bitcode_from_buffer(&buffer, context).map_err(|e| e.to_string())?;
        let mm = choose_entry_points(module_functions(&m), None)?;
        for mz in mm {
            names.push(mz.get_name().to_str().unwrap().to_owned());
        }
        module.link_in_module(m).map_err(|e| e.to_string())?;
    }
    Ok((module, names))
}

fn choose_entry_points<'ctx>(
    functions: impl Iterator<Item = FunctionValue<'ctx>>,
    name: Option<&str>,
) -> Result<Vec<FunctionValue<'ctx>>, String> {
    let entry_points = functions
        .filter(|f| is_entry_point(*f) && name.iter().all(|n| f.get_name().to_str() == Ok(n)))
        .collect::<Vec<FunctionValue<'ctx>>>();
    Ok(entry_points)
}

fn is_entry_point(function: FunctionValue) -> bool {
    function
        .get_string_attribute(AttributeLoc::Function, "EntryPoint")
        .is_some()
}

pub(crate) fn module_functions<'ctx>(
    module: &Module<'ctx>,
) -> impl Iterator<Item = FunctionValue<'ctx>> {
    struct FunctionValueIter<'ctx>(Option<FunctionValue<'ctx>>);

    impl<'ctx> Iterator for FunctionValueIter<'ctx> {
        type Item = FunctionValue<'ctx>;

        fn next(&mut self) -> Option<Self::Item> {
            let function = self.0;
            self.0 = function.and_then(inkwell::values::FunctionValue::get_next_function);
            function
        }
    }

    FunctionValueIter(module.get_first_function())
}

fn create_entry_point<'ctx>(module: &Module<'ctx>, name: Option<&str>) -> FunctionValue<'ctx> {
    let context = module.get_context();
    let fn_type = context.void_type().fn_type(&[], false);
    let fn_value = module.add_function(name.unwrap_or("main"), fn_type, None);

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
    use crate::{module::create_entry_point, tests::assert_reference_ir};
    use inkwell::context::Context;
    use std::{fs::File, io::Read, path::Path};
    use tempfile::tempdir;

    #[test]
    fn entry_point_function_has_correct_signature_and_default_attribute() {
        let context = Context::create();
        let module = context.create_module("test");
        let builder = context.create_builder();

        let entry_point = create_entry_point(&module, None);
        let entry = context.append_basic_block(entry_point, "entry");
        builder.position_at_end(entry);
        builder.build_return(None);
        let ir_string = module.print_to_string().to_string();
        let expected = "; ModuleID = 'test'\nsource_filename = \"test\"\n\ndefine void @main() #0 {\nentry:\n  ret void\n}\n\nattributes #0 = { \"EntryPoint\" }\n";
        assert_eq!(expected, ir_string);
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

    #[test]
    fn zero_required_qubits_results() -> Result<(), String> {
        assert_reference_ir("module/zero_required_qubits_results", 0, 0, |_| ())
    }

    #[test]
    fn one_required_qubit() -> Result<(), String> {
        assert_reference_ir("module/one_required_qubit", 1, 0, |_| ())
    }

    #[test]
    fn one_required_result() -> Result<(), String> {
        assert_reference_ir("module/one_required_result", 0, 1, |_| ())
    }

    #[test]
    fn many_required_qubits_results() -> Result<(), String> {
        assert_reference_ir("module/many_required_qubits_results", 5, 7, |_| ())
    }
}

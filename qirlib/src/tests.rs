// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{module, Builder};
use inkwell::context::Context;
use normalize_line_endings::normalized;
use std::{env, fs, path::PathBuf};

/// Compares generated IR against reference files in the "resources/tests" folder. If changes
/// to code generation break the tests:
///
/// 1. Run the tests with the `PYQIR_TEST_SAVE_REFERENCES` environment variable set to
///    regenerate the reference files.
/// 2. Review the changes and make sure they look reasonable.
/// 3. Unset the environment variable and run the tests again to confirm that they pass.
pub(crate) fn assert_reference_ir(
    id: &str,
    required_num_qubits: u64,
    required_num_results: u64,
    build: impl for<'ctx> Fn(&Builder<'ctx, '_>),
) -> Result<(), String> {
    const PYQIR_TEST_SAVE_REFERENCES: &str = "PYQIR_TEST_SAVE_REFERENCES";
    let (prefix, name) = split_id(id);
    let actual_ir = build_ir(name, required_num_qubits, required_num_results, build)?;

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources");
    path.push("tests");
    prefix.iter().for_each(|p| path.push(p));
    path.push(name);
    path.set_extension("ll");

    if env::var(PYQIR_TEST_SAVE_REFERENCES).is_ok() {
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&path, actual_ir).map_err(|e| e.to_string())?;

        Err(format!(
            "Saved reference IR. Run again without the {} environment variable.",
            PYQIR_TEST_SAVE_REFERENCES
        ))
    } else {
        let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let expected_ir: String = normalized(contents.chars()).collect();
        assert_eq!(expected_ir, actual_ir);
        Ok(())
    }
}

fn build_ir(
    name: &str,
    required_num_qubits: u64,
    required_num_results: u64,
    build: impl for<'ctx> Fn(&Builder<'ctx, '_>),
) -> Result<String, String> {
    let context = Context::create();
    let module = context.create_module(name);
    let builder = Builder::new(&module);
    module::simple_init(&module, &builder, required_num_qubits, required_num_results);
    build(&builder);
    builder.build_return(None);
    module::simple_finalize(&module)?;
    Ok(module.print_to_string().to_string())
}

fn split_id(id: &str) -> (Vec<&str>, &str) {
    let mut parts: Vec<_> = id.split('/').collect();
    let name = parts.pop().unwrap();
    (parts, name)
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{context::Context, memory_buffer::MemoryBuffer, module::Module};
use std::{ffi::OsStr, path::Path};

/// # Errors
///
/// - Module fails to load.
pub fn load_template(context: &Context) -> Result<Module, String> {
    load_memory(include_bytes!("module.bc"), context)
}

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
pub fn load_memory<'ctx>(bytes: &[u8], context: &'ctx Context) -> Result<Module<'ctx>, String> {
    let buffer = MemoryBuffer::create_from_memory_range_copy(bytes, "QIR module");
    Module::parse_bitcode_from_buffer(&buffer, context).map_err(|e| e.to_string())
}

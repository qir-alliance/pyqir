// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{memory_buffer::MemoryBuffer, module::Module};

use std::path::Path;

use crate::context::ContextType;

pub(crate) fn load_module<'ctx>(
    context: &'ctx inkwell::context::Context,
    context_type: ContextType<'ctx>,
) -> Result<Module<'ctx>, String> {
    match context_type {
        ContextType::Template => {
            let template = include_bytes!("module.bc");
            load_module_from_bytes(template, "template", context)
        }
        ContextType::File(path) => {
            let ext = path.extension().and_then(std::ffi::OsStr::to_str);
            match ext {
                Some("ll") => load_module_from_ir_file(path, context),
                Some("bc") => load_module_from_bitcode_file(path, context),
                _ => panic!("Unsupported module extension {:?}", ext),
            }
        }
        ContextType::Memory(bytes) => load_module_from_bytes(bytes, "memory", context),
    }
}

fn load_module_from_bytes<'ctx>(
    bytes: &[u8],
    name: &str,
    context: &'ctx inkwell::context::Context,
) -> Result<Module<'ctx>, String> {
    let buffer = MemoryBuffer::create_from_memory_range_copy(bytes, name);
    Module::parse_bitcode_from_buffer(&buffer, context).map_err(|e| e.to_string())
}

pub(crate) fn load_module_from_bitcode_file<'ctx, P: AsRef<Path>>(
    path: P,
    context: &'ctx inkwell::context::Context,
) -> Result<Module<'ctx>, String> {
    Module::parse_bitcode_from_path(path, context).map_err(|e| e.to_string())
}

pub(crate) fn load_module_from_ir_file<'ctx, P: AsRef<Path>>(
    path: P,
    context: &'ctx inkwell::context::Context,
) -> Result<Module<'ctx>, String> {
    let buffer = MemoryBuffer::create_from_file(path.as_ref()).map_err(|e| e.to_string())?;
    context
        .create_module_from_ir(buffer)
        .map_err(|e| e.to_string())
}

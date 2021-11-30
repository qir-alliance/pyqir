// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use inkwell::{memory_buffer::MemoryBuffer, module::Module};

use std::path::Path;

use crate::context::ContextType;

pub(crate) fn load_module<'ctx>(
    context: &'ctx inkwell::context::Context,
    context_type: ContextType<'ctx>,
) -> Result<Module<'ctx>, String> {
    let module = match context_type {
        ContextType::Template(name) => load_module_from_bitcode_template(&context, &name[..])?,
        ContextType::File(file_name) => {
            let file_path = Path::new(&file_name[..]);
            let ext = file_path.extension().and_then(std::ffi::OsStr::to_str);
            let module = match ext {
                Some("ll") => load_module_from_ir_file(file_path, context)?,
                Some("bc") => load_module_from_bitcode_file(file_path, context)?,
                _ => panic!("Unsupported module extension {:?}", ext),
            };
            module
        }
    };
    Ok(module)
}

pub(crate) fn load_module_from_bitcode_template<'ctx>(
    context: &'ctx inkwell::context::Context,
    name: &'ctx str,
) -> Result<Module<'ctx>, String> {
    let module_contents = include_bytes!("module.bc");
    let buffer = MemoryBuffer::create_from_memory_range_copy(module_contents, name);
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
    let memory_buffer = load_memory_buffer_from_ir_file(path)?;
    context
        .create_module_from_ir(memory_buffer)
        .map_err(|e| e.to_string())
}

pub(crate) fn load_memory_buffer_from_ir_file<P: AsRef<Path>>(
    path: P,
) -> Result<MemoryBuffer, String> {
    MemoryBuffer::create_from_file(path.as_ref()).map_err(|e| e.to_string())
}

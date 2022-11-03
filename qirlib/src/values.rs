// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types;
use inkwell::{module::Module, values::PointerValue};

pub fn qubit<'ctx>(module: &Module<'ctx>, id: u64) -> PointerValue<'ctx> {
    module
        .get_context()
        .i64_type()
        .const_int(id, false)
        .const_to_pointer(types::qubit(module))
}

pub fn result<'ctx>(module: &Module<'ctx>, id: u64) -> PointerValue<'ctx> {
    module
        .get_context()
        .i64_type()
        .const_int(id, false)
        .const_to_pointer(types::result(module))
}

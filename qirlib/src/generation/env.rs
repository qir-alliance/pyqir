use crate::generation::interop::Variable;
use inkwell::values::BasicValueEnum;
use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};

pub(crate) struct Environment<'ctx> {
    variables: HashMap<Variable, BasicValueEnum<'ctx>>,
}

impl<'ctx> Environment<'ctx> {
    pub(crate) fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub(crate) fn variable(&self, var: Variable) -> Option<BasicValueEnum<'ctx>> {
        self.variables.get(&var).copied()
    }

    pub(crate) fn set_variable(
        &mut self,
        var: Variable,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), VariableReassignedError> {
        match self.variables.entry(var) {
            Occupied(_) => Err(VariableReassignedError),
            Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct VariableReassignedError;

use crate::generation::interop::Variable;
use inkwell::values::BasicValueEnum;
use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};

pub(crate) struct Environment<'ctx> {
    qubits: HashMap<String, BasicValueEnum<'ctx>>,
    results: HashMap<String, BasicValueEnum<'ctx>>,
    variables: HashMap<Variable, BasicValueEnum<'ctx>>,
}

impl<'ctx> Environment<'ctx> {
    pub(crate) fn new(
        qubits: HashMap<String, BasicValueEnum<'ctx>>,
        results: HashMap<String, BasicValueEnum<'ctx>>,
        variables: HashMap<Variable, BasicValueEnum<'ctx>>,
    ) -> Self {
        Self {
            qubits,
            results,
            variables,
        }
    }

    pub(crate) fn qubit(&self, name: &str) -> Option<BasicValueEnum<'ctx>> {
        self.qubits.get(name).copied()
    }

    pub(crate) fn result(&self, name: &str) -> Option<BasicValueEnum<'ctx>> {
        self.results.get(name).copied()
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

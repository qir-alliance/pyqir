use crate::generation::interop::Variable;
use inkwell::values::{BasicValueEnum, PointerValue};
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    convert::Into,
};

pub(crate) struct Environment<'ctx> {
    qubits: HashMap<String, BasicValueEnum<'ctx>>,
    results: HashMap<String, Option<PointerValue<'ctx>>>,
    variables: HashMap<Variable, BasicValueEnum<'ctx>>,
}

impl<'ctx> Environment<'ctx> {
    pub(crate) fn new(
        qubits: HashMap<String, BasicValueEnum<'ctx>>,
        results: HashMap<String, Option<PointerValue<'ctx>>>,
        variables: HashMap<Variable, BasicValueEnum<'ctx>>,
    ) -> Self {
        Self {
            qubits,
            results,
            variables,
        }
    }

    pub(crate) fn qubit(&self, name: &str) -> Option<BasicValueEnum<'ctx>> {
        self.qubits.get(name).cloned()
    }

    pub(crate) fn iter_qubits(&self) -> impl Iterator<Item = (&str, BasicValueEnum<'ctx>)> {
        self.qubits
            .iter()
            .map(|(name, value)| (name.as_str(), *value))
    }

    pub(crate) fn result(&self, name: &str) -> Option<Option<PointerValue<'ctx>>> {
        self.results.get(name).cloned()
    }

    pub(crate) fn set_result(
        &mut self,
        name: String,
        value: PointerValue<'ctx>,
    ) -> Result<(), String> {
        match self.results.entry(name) {
            Occupied(mut entry) => {
                *entry.get_mut() = Some(value);
                Ok(())
            }
            Vacant(_) => Err("Result not found. Results can only be updated, not created.".into()),
        }
    }

    pub(crate) fn variable(&self, var: &Variable) -> Option<BasicValueEnum<'ctx>> {
        self.variables.get(var).cloned()
    }

    pub(crate) fn set_variable(
        &mut self,
        var: Variable,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), String> {
        match self.variables.entry(var) {
            Occupied(_) => Err("Variable already exists. Variables cannot be reassigned.".into()),
            Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }
}

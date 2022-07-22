use crate::generation::interop::Variable;
use inkwell::values::{BasicValueEnum, PointerValue};
use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
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
        self.qubits.get(name).copied()
    }

    pub(crate) fn iter_qubits(&self) -> impl Iterator<Item = (&str, BasicValueEnum<'ctx>)> {
        self.qubits
            .iter()
            .map(|(name, value)| (name.as_str(), *value))
    }

    pub(crate) fn result(&self, name: &str) -> ResultState<'ctx> {
        match self.results.get(name) {
            None => ResultState::NotFound,
            Some(None) => ResultState::Uninitialized,
            Some(&Some(r)) => ResultState::Initialized(r),
        }
    }

    pub(crate) fn set_result(
        &mut self,
        name: String,
        value: PointerValue<'ctx>,
    ) -> Result<(), ResultNotFoundError> {
        match self.results.entry(name) {
            Occupied(mut entry) => {
                *entry.get_mut() = Some(value);
                Ok(())
            }
            Vacant(_) => Err(ResultNotFoundError),
        }
    }

    pub(crate) fn variable(&self, var: &Variable) -> Option<BasicValueEnum<'ctx>> {
        self.variables.get(var).copied()
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

#[derive(Debug)]
pub(crate) struct ResultNotFoundError;

pub(crate) enum ResultState<'ctx> {
    NotFound,
    Uninitialized,
    Initialized(PointerValue<'ctx>),
}

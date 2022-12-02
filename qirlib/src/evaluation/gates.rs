// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use lazy_static::lazy_static;
#[allow(clippy::upper_case_acronyms)]
type QUBIT = u64;
#[allow(clippy::upper_case_acronyms)]
type RESULT = u64;
use mut_static::MutStatic;

use crate::evaluation::interop::{ClassicalRegister, Instruction, QuantumRegister, SemanticModel};

lazy_static! {
    pub static ref CURRENT_GATES: MutStatic<BaseProfile> = MutStatic::from(BaseProfile::new());
}

#[derive(Default)]
pub struct BaseProfile {
    model: SemanticModel,
    max_id: QUBIT,
    declared_cubits: bool,
}

pub struct GateScope {}

impl GateScope {
    #[must_use]
    pub fn new() -> GateScope {
        let mut gs = CURRENT_GATES
            .write()
            .expect("Could not acquire lock on gate set.");
        gs.clear_gateset();
        GateScope {}
    }
}

impl Default for GateScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GateScope {
    fn drop(&mut self) {
        let mut gs = CURRENT_GATES
            .write()
            .expect("Could not acquire lock on gate set.");
        gs.clear_gateset();
    }
}

impl BaseProfile {
    #[must_use]
    pub fn new() -> Self {
        BaseProfile {
            model: SemanticModel::new(String::from("QIR")),
            max_id: 0,
            declared_cubits: false,
        }
    }

    pub fn reset(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("reset {}", qubit);
        self.model.add_inst(Instruction::new(
            "reset",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn clear_gateset(&mut self) {
        self.model = SemanticModel::new(String::from("QIR"));
        self.max_id = 0;
        self.declared_cubits = false;
    }

    fn record_max_qubit_id(&mut self, qubit: QUBIT) {
        self.declared_cubits = true;
        if qubit > self.max_id {
            self.max_id = qubit;
        }
    }

    #[must_use]
    pub fn get_model(&self) -> SemanticModel {
        self.model.clone()
    }

    pub fn infer_allocations(&mut self) {
        if !self.declared_cubits {
            return;
        }
        for index in 0..=self.max_id {
            let qr = QuantumRegister::new(String::from("qubit"), index);
            self.model.add_reg(&qr.as_register());
        }
        let cr = ClassicalRegister::new(String::from("output"), self.max_id + 1);
        self.model.add_reg(&cr.as_register());
    }

    pub fn barrier(&mut self) {
        log::debug!("barrier");
        self.model.add_inst(Instruction::new("barrier", vec![]));
    }

    pub fn cx(&mut self, control: QUBIT, target: QUBIT) {
        self.record_max_qubit_id(control);
        self.record_max_qubit_id(target);

        log::debug!("cx {}:{}", control, target);
        self.model.add_inst(Instruction::new(
            "cx",
            [control.to_string().as_str(), target.to_string().as_str()].to_vec(),
        ));
    }

    pub fn cz(&mut self, control: QUBIT, target: QUBIT) {
        self.record_max_qubit_id(control);
        self.record_max_qubit_id(target);

        log::debug!("cz {}:{}", control, target);
        self.model.add_inst(Instruction::new(
            "cz",
            [control.to_string().as_str(), target.to_string().as_str()].to_vec(),
        ));
    }

    pub fn h(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("h {}", qubit);
        self.model.add_inst(Instruction::new(
            "h",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn m(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("m {}", qubit);
        self.model.add_inst(Instruction::new(
            "m",
            vec![BaseProfile::get_qubit_string(qubit).as_str()],
        ));
    }

    pub fn mz(&mut self, qubit: QUBIT, result: RESULT) {
        self.record_max_qubit_id(qubit);

        log::debug!("m {} into {}", qubit, result);
        self.model.add_inst(Instruction::new(
            "m",
            [
                BaseProfile::get_qubit_string(qubit).as_str(),
                BaseProfile::get_result_string(Some(result)).as_str(),
            ]
            .to_vec(),
        ));
    }

    pub fn rx(&mut self, theta: f64, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("rx {}({})", qubit, theta);

        self.model.add_inst(Instruction::new(
            "rx",
            [
                theta.to_string().as_str(),
                BaseProfile::get_qubit_string(qubit).as_str(),
            ]
            .to_vec(),
        ));
    }

    pub fn ry(&mut self, theta: f64, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("ry {}({})", qubit, theta);

        self.model.add_inst(Instruction::new(
            "ry",
            [
                theta.to_string().as_str(),
                BaseProfile::get_qubit_string(qubit).as_str(),
            ]
            .to_vec(),
        ));
    }

    pub fn rz(&mut self, theta: f64, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("rz {}({})", qubit, theta);

        self.model.add_inst(Instruction::new(
            "rz",
            [
                theta.to_string().as_str(),
                BaseProfile::get_qubit_string(qubit).as_str(),
            ]
            .to_vec(),
        ));
    }

    pub fn s(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("s {}", qubit);

        self.model.add_inst(Instruction::new(
            "s",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn s_adj(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("s_adj {}", qubit);
        self.model.add_inst(Instruction::new(
            "s_adj",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn swap(&mut self, qubit1: QUBIT, qubit2: QUBIT) {
        self.record_max_qubit_id(qubit1);
        self.record_max_qubit_id(qubit2);

        log::debug!("swap ({}, {})", qubit1, qubit2);

        self.model.add_inst(Instruction::new(
            "swap",
            [qubit1.to_string().as_str(), qubit2.to_string().as_str()].to_vec(),
        ));
    }

    pub fn t(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("t {}", qubit);
        self.model.add_inst(Instruction::new(
            "t",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn t_adj(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("t_adj {}", qubit);
        self.model.add_inst(Instruction::new(
            "t_adj",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn x(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("x {}", qubit);
        self.model.add_inst(Instruction::new(
            "x",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn y(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("y {}", qubit);
        self.model.add_inst(Instruction::new(
            "y",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    pub fn z(&mut self, qubit: QUBIT) {
        self.record_max_qubit_id(qubit);

        log::debug!("z {}", qubit);
        self.model.add_inst(Instruction::new(
            "z",
            [BaseProfile::get_qubit_string(qubit).as_str()].to_vec(),
        ));
    }

    fn get_qubit_string(qubit: QUBIT) -> String {
        format!("{qubit}")
    }

    fn get_result_string(result: Option<RESULT>) -> String {
        match result {
            Some(value) => format!("{value}"),
            None => String::new(),
        }
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuantumRegister {
    pub name: String,
    pub index: u64,
}

impl QuantumRegister {
    #[must_use]
    pub fn new(name: String, index: u64) -> Self {
        QuantumRegister { name, index }
    }

    #[must_use]
    pub fn as_register(&self) -> Register {
        Register::Quantum(self.clone())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClassicalRegister {
    pub name: String,
    pub size: u64,
}

impl ClassicalRegister {
    #[must_use]
    pub fn new(name: String, size: u64) -> Self {
        ClassicalRegister { name, size }
    }

    #[must_use]
    pub fn as_register(&self) -> Register {
        Register::Classical(self.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Register {
    Quantum(QuantumRegister),
    Classical(ClassicalRegister),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controlled {
    pub control: String,
    pub target: String,
}

impl Controlled {
    #[must_use]
    pub fn new(control: String, target: String) -> Self {
        Controlled { control, target }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Measured {
    pub qubit: String,
    pub target: String,
}

impl Measured {
    #[must_use]
    pub fn new(qubit: String, target: String) -> Self {
        Measured { qubit, target }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rotated {
    pub theta: f64,
    pub qubit: String,
}

impl Rotated {
    #[must_use]
    pub fn new(theta: f64, qubit: String) -> Self {
        Rotated { theta, qubit }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Single {
    pub qubit: String,
}

impl Single {
    #[must_use]
    pub fn new(qubit: String) -> Self {
        Single { qubit }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub name: String,
    pub args: Vec<String>,
}

impl Instruction {
    #[must_use]
    pub fn new(name: &str, args: Vec<&str>) -> Self {
        Instruction {
            name: name.to_string(),
            args: args
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }
}

#[derive(Clone, Default)]
pub struct SemanticModel {
    pub name: String,
    pub registers: Vec<ClassicalRegister>,
    pub qubits: Vec<QuantumRegister>,
    pub instructions: Vec<Instruction>,
}

impl SemanticModel {
    #[must_use]
    pub fn new(name: String) -> Self {
        SemanticModel {
            name,
            registers: vec![],
            qubits: vec![],
            instructions: vec![],
        }
    }

    pub fn add_reg(&mut self, reg: &Register) {
        match &reg {
            Register::Classical(classical) => self.registers.push(classical.clone()),
            Register::Quantum(quantum) => self.qubits.push(quantum.clone()),
        }
    }

    pub fn add_inst(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

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

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    pub condition: String,
    pub then_insts: Vec<Instruction>,
    pub else_insts: Vec<Instruction>,
}

// https://github.com/microsoft/qsharp-language/blob/ageller/profile/Specifications/QIR/Base-Profile.md
#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Cx(Controlled),
    Cz(Controlled),
    H(Single),
    S(Single),
    SAdj(Single),
    T(Single),
    TAdj(Single),
    X(Single),
    Y(Single),
    Z(Single),
    Rx(Rotated),
    Ry(Rotated),
    Rz(Rotated),
    Reset(Single),
    M(Measured),
    Call(Call),
    If(If),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Value>,
}

#[derive(Clone, Copy)]
pub enum ValueType {
    Integer { width: u32 },
    Double,
    Qubit,
    Result,
}

#[derive(Clone)]
pub enum ReturnType {
    Void,
    Value(ValueType),
}

#[derive(Clone)]
pub struct FunctionType {
    pub param_types: Vec<ValueType>,
    pub return_type: ReturnType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer { width: u32, value: u64 },
    Double(f64),
    Qubit(String),
    Result(String),
}

#[derive(Clone)]
pub struct SemanticModel {
    pub name: String,
    pub registers: Vec<ClassicalRegister>,
    pub qubits: Vec<QuantumRegister>,
    pub instructions: Vec<Instruction>,
    pub static_alloc: bool,
    pub external_functions: HashMap<String, FunctionType>,
}

impl SemanticModel {
    #[must_use]
    pub fn new(name: String) -> Self {
        SemanticModel {
            name,
            registers: vec![],
            qubits: vec![],
            instructions: vec![],
            static_alloc: false,
            external_functions: HashMap::new(),
        }
    }

    pub fn add_reg(&mut self, reg: &Register) {
        match reg {
            Register::Classical(classical) => self.registers.push(classical.clone()),
            Register::Quantum(quantum) => self.qubits.push(quantum.clone()),
        }
    }

    pub fn add_inst(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }
}

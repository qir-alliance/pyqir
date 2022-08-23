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

#[derive(Clone, Debug, PartialEq)]
pub struct Controlled {
    pub control: Value,
    pub target: Value,
}

impl Controlled {
    #[must_use]
    pub fn new(control: Value, target: Value) -> Self {
        Controlled { control, target }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Measured {
    pub qubit: Value,
    pub target: String,
}

impl Measured {
    #[must_use]
    pub fn new(qubit: Value, target: String) -> Self {
        Measured { qubit, target }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rotated {
    pub theta: Value,
    pub qubit: Value,
}

impl Rotated {
    #[must_use]
    pub fn new(theta: Value, qubit: Value) -> Self {
        Rotated { theta, qubit }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Single {
    pub qubit: Value,
}

impl Single {
    #[must_use]
    pub fn new(qubit: Value) -> Self {
        Single { qubit }
    }
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
    BinaryOp(BinaryOp),
    Call(Call),
    If(If),
    IfResult(IfResult),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOp {
    pub kind: BinaryKind,
    pub lhs: Value,
    pub rhs: Value,
    pub result: Variable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryKind {
    And,
    Or,
    Xor,
    Add,
    Sub,
    Mul,
    Shl,
    LShr,
    ICmp(IntPredicate),
}

/// An integer comparison predicate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntPredicate {
    /// Equal.
    EQ,
    /// Not equal.
    NE,
    /// Unsigned greater than.
    UGT,
    /// Unsigned greater than or equal.
    UGE,
    /// Unsigned less than.
    ULT,
    /// Unsigned less than or equal.
    ULE,
    /// Signed greater than.
    SGT,
    /// Signed greater than or equal.
    SGE,
    /// Signed less than.
    SLT,
    /// Signed less than or equal.
    SLE,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Value>,
    pub result: Option<Variable>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    pub cond: Value,
    pub if_true: Vec<Instruction>,
    pub if_false: Vec<Instruction>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfResult {
    pub cond: String,
    pub if_one: Vec<Instruction>,
    pub if_zero: Vec<Instruction>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Void,
    Int {
        width: u32,
    },
    Double,
    Qubit,
    Result,
    Function {
        params: Vec<Type>,
        result: Box<Type>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(Int),
    Double(f64),
    Qubit(String),
    Result(String),
    Variable(Variable),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Variable {
    id: i64,
}

impl Variable {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn next(&self) -> Self {
        Self { id: self.id + 1 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Int {
    width: u32,
    value: u64,
}

impl Int {
    /// Creates a new constant integer, returning `None` if the number of bits required to represent
    /// `value` is greater than `width`.
    #[must_use]
    pub fn new(width: u32, value: u64) -> Option<Self> {
        let value_width = u64::BITS - u64::leading_zeros(value);
        if value_width > width {
            None
        } else {
            Some(Self { width, value })
        }
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn value(&self) -> u64 {
        self.value
    }
}

#[derive(Clone)]
pub struct SemanticModel {
    pub name: String,
    pub registers: Vec<ClassicalRegister>,
    pub qubits: Vec<QuantumRegister>,
    pub instructions: Vec<Instruction>,
    pub use_static_qubit_alloc: bool,
    pub use_static_result_alloc: bool,
    pub external_functions: Vec<(String, Type)>,
}

impl SemanticModel {
    #[must_use]
    pub fn new(name: String) -> Self {
        SemanticModel {
            name,
            registers: vec![],
            qubits: vec![],
            instructions: vec![],
            use_static_qubit_alloc: false,
            use_static_result_alloc: true,
            external_functions: vec![],
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

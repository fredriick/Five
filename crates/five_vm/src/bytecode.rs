//! Bytecode definitions.

/// Operation codes for the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    /// Push a constant onto the stack
    Constant,
    /// Push nil
    Nil,
    /// Push true
    True,
    /// Push false
    False,
    /// Pop a value from the stack
    Pop,

    /// Get a local variable
    GetLocal,
    /// Set a local variable
    SetLocal,
    /// Get a global variable
    GetGlobal,
    /// Set a global variable
    SetGlobal,
    /// Define a global variable
    DefineGlobal,

    /// Add two values
    Add,
    /// Subtract two values
    Subtract,
    /// Multiply two values
    Multiply,
    /// Divide two values
    Divide,
    /// Modulo two values
    Modulo,
    /// Negate a value
    Negate,
    /// Logical not
    Not,

    /// Equality comparison
    Equal,
    /// Not equal comparison
    NotEqual,
    /// Less than comparison
    Less,
    /// Less than or equal comparison
    LessEqual,
    /// Greater than comparison
    Greater,
    /// Greater than or equal comparison
    GreaterEqual,

    /// Unconditional jump
    Jump,
    /// Jump if top of stack is false
    JumpIfFalse,
    /// Jump if top of stack is true
    JumpIfTrue,

    /// Call a function
    Call,
    /// Return from a function
    Return,

    /// Create an array
    Array,
    /// Index into an array
    Index,
}

/// A bytecode instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operand: Option<u32>,
}

impl Instruction {
    /// Create a new instruction.
    pub fn new(opcode: OpCode, operand: Option<u32>) -> Self {
        Self { opcode, operand }
    }

    /// Create a simple instruction with no operand.
    pub fn simple(opcode: OpCode) -> Self {
        Self::new(opcode, None)
    }
}

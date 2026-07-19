//! Bytecode chunks.

use crate::bytecode::Instruction;
use crate::VMValue;

/// A chunk of bytecode.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chunk {
    /// The bytecode instructions.
    pub code: Vec<Instruction>,
    /// Constant pool.
    pub constants: Vec<VMValue>,
    /// Line numbers for debugging.
    pub lines: Vec<usize>,
}

impl Chunk {
    /// Create a new empty chunk.
    pub fn new() -> Self {
        Self::default()
    }

    /// Write an instruction to the chunk.
    pub fn write(&mut self, instruction: Instruction, line: usize) {
        self.code.push(instruction);
        self.lines.push(line);
    }

    /// Add a constant and return its index.
    pub fn add_constant(&mut self, value: VMValue) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// Get the current code offset.
    pub fn current_offset(&self) -> usize {
        self.code.len()
    }

    /// Patch a jump instruction with the actual offset.
    pub fn patch_jump(&mut self, offset: usize) {
        let jump_target = self.code.len();
        if let Some(instr) = self.code.get_mut(offset) {
            instr.operand = Some(jump_target as u32);
        }
    }
}

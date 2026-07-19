//! Five Compiler - Compiles AST to bytecode.

mod codegen;

pub use codegen::Compiler;

use five_ast::Program;
use five_core::FiveResult;
use five_vm::Chunk;

/// Compile a program to bytecode.
pub fn compile(program: &Program) -> FiveResult<Chunk> {
    let mut compiler = Compiler::new();
    compiler.compile_program(program)
}

//! Five AST - Abstract Syntax Tree definitions for the Five programming language.
//!
//! The AST is designed to be AI-native, meaning it can be easily serialized
//! and manipulated as first-class data.

pub mod expr;
pub mod stmt;
pub mod types;
pub mod visitor;

pub use expr::*;
pub use stmt::*;
pub use types::*;
pub use visitor::*;

use five_core::Span;
use serde::{Deserialize, Serialize};

/// A complete Five program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        let span = if statements.is_empty() {
            Span::dummy()
        } else {
            let first = statements.first().unwrap().span;
            let last = statements.last().unwrap().span;
            first.merge(last)
        };
        Self { statements, span }
    }
}

/// A literal value in the source code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Nil,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // Logical
    And,
    Or,
    // Pipe
    Pipe,
}

impl BinaryOp {
    /// Get the precedence of this operator (higher = binds tighter).
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Eq | Self::Ne => 3,
            Self::Lt | Self::Gt | Self::Le | Self::Ge => 4,
            Self::Add | Self::Sub => 5,
            Self::Mul | Self::Div | Self::Mod => 6,
            Self::Pipe => 0, // Lowest precedence, left-to-right
        }
    }

    /// Check if this operator is left-associative.
    pub fn is_left_assoc(&self) -> bool {
        true // All our operators are left-associative
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,      // &
    MutRef,   // &mut
    Deref,    // *
}

/// Effect annotations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    IO,
    State,
    Async,
    Pure,
    Custom(String),
}

/// Ownership mode for bindings and parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Ownership {
    #[default]
    Owned,
    Borrowed,
    MutBorrowed,
}

/// A pattern for matching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternKind {
    /// Wildcard pattern: _
    Wildcard,
    /// Variable binding: x
    Identifier(String),
    /// Literal pattern: 42, "hello"
    Literal(Literal),
    /// Constructor pattern: Some(x), None
    Constructor {
        name: String,
        args: Vec<Pattern>,
    },
    /// Tuple pattern: (a, b, c)
    Tuple(Vec<Pattern>),
}

/// A match arm (pattern => expression).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<Type>,
    pub ownership: Ownership,
    pub span: Span,
}

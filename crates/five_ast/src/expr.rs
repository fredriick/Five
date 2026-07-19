//! Expression AST nodes.

use crate::{BinaryOp, Effect, Literal, MatchArm, Param, Stmt, UnaryOp};
use five_core::Span;
use serde::{Deserialize, Serialize};

/// An expression in Five.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Create a literal expression.
    pub fn literal(lit: Literal, span: Span) -> Self {
        Self::new(ExprKind::Literal(lit), span)
    }

    /// Create an identifier expression.
    pub fn ident(name: impl Into<String>, span: Span) -> Self {
        Self::new(ExprKind::Identifier(name.into()), span)
    }

    /// Create a binary expression.
    pub fn binary(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span,
        )
    }

    /// Create a unary expression.
    pub fn unary(op: UnaryOp, expr: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::Unary {
                op,
                expr: Box::new(expr),
            },
            span,
        )
    }
}

/// The kind of an expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    /// A literal value: 42, "hello", true
    Literal(Literal),

    /// An identifier: foo, bar
    Identifier(String),

    /// A binary operation: a + b, x == y
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    /// A unary operation: -x, !flag
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    /// A function call: foo(a, b)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    /// A method call: obj.method(args)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    /// Field access: obj.field
    Field {
        object: Box<Expr>,
        field: String,
    },

    /// Index access: arr[idx]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// A lambda expression: (x, y) => x + y
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
        effects: Vec<Effect>,
    },

    /// An if expression: if cond { then } else { else }
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },

    /// A match expression
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// A block expression: { stmts; expr }
    Block {
        statements: Vec<Stmt>,
        /// The final expression (return value of the block)
        expr: Option<Box<Expr>>,
    },

    /// Array literal: [1, 2, 3]
    Array(Vec<Expr>),

    /// Tuple literal: (a, b, c)
    Tuple(Vec<Expr>),

    /// Await expression: await future
    Await(Box<Expr>),

    /// Return expression (for early returns)
    Return(Option<Box<Expr>>),

    /// Break expression
    Break(Option<Box<Expr>>),

    /// Continue expression
    Continue,

    /// Range expression: 1..10 or 1..=10
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },

    /// Struct instantiation: Point { x: 1, y: 2 }
    Struct {
        name: String,
        fields: Vec<(String, Expr)>,
    },

    /// Assignment: x = value
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    /// Compound assignment: x += value, x -= value, etc.
    CompoundAssign {
        target: Box<Expr>,
        op: BinaryOp,
        value: Box<Expr>,
    },
}

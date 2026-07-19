//! Statement AST nodes.

use crate::{Effect, Expr, Ownership, Param, Type};
use five_core::Span;
use serde::{Deserialize, Serialize};

/// A statement in Five.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The kind of a statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    /// Variable binding: let x = expr
    Let {
        name: String,
        type_ann: Option<Type>,
        value: Expr,
        mutable: bool,
        ownership: Ownership,
    },

    /// Function declaration
    Fn {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        effects: Vec<Effect>,
        body: Expr,
        is_async: bool,
    },

    /// Type alias: type Name = Type
    Type {
        name: String,
        params: Vec<String>,
        definition: Type,
    },

    /// Algebraic data type: data Option<T> { Some(T), None }
    Data {
        name: String,
        params: Vec<String>,
        variants: Vec<Variant>,
    },

    /// Struct definition
    Struct {
        name: String,
        params: Vec<String>,
        fields: Vec<StructField>,
    },

    /// Expression statement
    Expr(Expr),

    /// While loop
    While {
        condition: Expr,
        body: Expr,
    },

    /// For loop: for x in iter { body }
    For {
        binding: String,
        iter: Expr,
        body: Expr,
    },

    /// Import statement: import module::item
    Import {
        path: Vec<String>,
        alias: Option<String>,
    },

    /// Export statement: export fn foo
    Export(Box<Stmt>),
}

/// A variant in an algebraic data type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub fields: VariantFields,
    pub span: Span,
}

/// Fields of a variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariantFields {
    /// No fields: None
    Unit,
    /// Positional fields: Some(T)
    Tuple(Vec<Type>),
    /// Named fields: Person { name: String, age: Int }
    Named(Vec<StructField>),
}

/// A field in a struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

//! Type system AST nodes.

use crate::Effect;
use five_core::Span;
use serde::{Deserialize, Serialize};

/// A type in Five.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

impl Type {
    pub fn new(kind: TypeKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Create a named type.
    pub fn named(name: impl Into<String>, span: Span) -> Self {
        Self::new(TypeKind::Named(name.into()), span)
    }

    /// Create the Any type.
    pub fn any(span: Span) -> Self {
        Self::new(TypeKind::Any, span)
    }

    /// Create the Infer type (for type inference).
    pub fn infer(span: Span) -> Self {
        Self::new(TypeKind::Infer, span)
    }

    /// Create the Unit type.
    pub fn unit(span: Span) -> Self {
        Self::new(TypeKind::Unit, span)
    }
}

/// The kind of a type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    /// A named type: Int, String, MyType
    Named(String),

    /// A generic type: Option<T>, Result<T, E>
    Generic {
        name: String,
        params: Vec<Type>,
    },

    /// A function type: (Int, Int) -> Int with IO
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        effects: Vec<Effect>,
    },

    /// A union type: A | B | C
    Union(Vec<Type>),

    /// A reference type: &T or &mut T
    Reference {
        inner: Box<Type>,
        mutable: bool,
        lifetime: Option<Lifetime>,
    },

    /// A tuple type: (A, B, C)
    Tuple(Vec<Type>),

    /// An array type: [T]
    Array(Box<Type>),

    /// The any type - opt out of type checking
    Any,

    /// Infer this type
    Infer,

    /// The unit type: ()
    Unit,

    /// Never type (for functions that don't return)
    Never,
}

/// A lifetime annotation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lifetime {
    pub name: String,
    pub span: Span,
}

impl Lifetime {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    /// The static lifetime.
    pub fn static_lifetime(span: Span) -> Self {
        Self::new("static", span)
    }
}

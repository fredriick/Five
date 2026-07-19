//! Five Types - Type system and type inference for the Five programming language.

mod infer;
mod unify;

pub use infer::TypeChecker;
pub use unify::Unifier;

use five_ast::{Effect, Type, TypeKind};
use five_core::Span;
use std::collections::HashMap;

/// A type variable (for inference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

/// Internal type representation for inference.
#[derive(Debug, Clone, PartialEq)]
pub enum InferType {
    /// A concrete named type
    Concrete(String),
    /// A type variable (to be inferred)
    Var(TypeVar),
    /// A function type
    Function {
        params: Vec<InferType>,
        return_type: Box<InferType>,
        effects: Vec<Effect>,
    },
    /// A generic type application
    Generic {
        name: String,
        params: Vec<InferType>,
    },
    /// A tuple type
    Tuple(Vec<InferType>),
    /// An array type
    Array(Box<InferType>),
    /// A reference type
    Reference {
        inner: Box<InferType>,
        mutable: bool,
    },
    /// The any type (opt-out of type checking)
    Any,
    /// The unit type
    Unit,
    /// The never type
    Never,
    /// An error type (for error recovery)
    Error,
}

impl InferType {
    /// Convert an AST type to an inference type.
    pub fn from_ast(ty: &Type) -> Self {
        match &ty.kind {
            TypeKind::Named(name) => Self::Concrete(name.clone()),
            TypeKind::Generic { name, params } => Self::Generic {
                name: name.clone(),
                params: params.iter().map(Self::from_ast).collect(),
            },
            TypeKind::Function {
                params,
                return_type,
                effects,
            } => Self::Function {
                params: params.iter().map(Self::from_ast).collect(),
                return_type: Box::new(Self::from_ast(return_type)),
                effects: effects.clone(),
            },
            TypeKind::Union(_types) => {
                // TODO: proper union type support
                Self::Any
            }
            TypeKind::Reference { inner, mutable, .. } => Self::Reference {
                inner: Box::new(Self::from_ast(inner)),
                mutable: *mutable,
            },
            TypeKind::Tuple(types) => Self::Tuple(types.iter().map(Self::from_ast).collect()),
            TypeKind::Array(inner) => Self::Array(Box::new(Self::from_ast(inner))),
            TypeKind::Any => Self::Any,
            TypeKind::Infer => Self::Any, // Will be replaced by type var
            TypeKind::Unit => Self::Unit,
            TypeKind::Never => Self::Never,
        }
    }

    /// Convert back to an AST type.
    pub fn to_ast(&self, span: Span) -> Type {
        let kind = match self {
            Self::Concrete(name) => TypeKind::Named(name.clone()),
            Self::Var(_) => TypeKind::Infer,
            Self::Function {
                params,
                return_type,
                effects,
            } => TypeKind::Function {
                params: params.iter().map(|t| t.to_ast(span)).collect(),
                return_type: Box::new(return_type.to_ast(span)),
                effects: effects.clone(),
            },
            Self::Generic { name, params } => TypeKind::Generic {
                name: name.clone(),
                params: params.iter().map(|t| t.to_ast(span)).collect(),
            },
            Self::Tuple(types) => TypeKind::Tuple(types.iter().map(|t| t.to_ast(span)).collect()),
            Self::Array(inner) => TypeKind::Array(Box::new(inner.to_ast(span))),
            Self::Reference { inner, mutable } => TypeKind::Reference {
                inner: Box::new(inner.to_ast(span)),
                mutable: *mutable,
                lifetime: None,
            },
            Self::Any => TypeKind::Any,
            Self::Unit => TypeKind::Unit,
            Self::Never => TypeKind::Never,
            Self::Error => TypeKind::Any,
        };
        Type::new(kind, span)
    }

    /// Check if this type contains a type variable.
    pub fn contains_var(&self, var: TypeVar) -> bool {
        match self {
            Self::Var(v) => *v == var,
            Self::Function {
                params,
                return_type,
                ..
            } => {
                params.iter().any(|p| p.contains_var(var)) || return_type.contains_var(var)
            }
            Self::Generic { params, .. } => params.iter().any(|p| p.contains_var(var)),
            Self::Tuple(types) => types.iter().any(|t| t.contains_var(var)),
            Self::Array(inner) => inner.contains_var(var),
            Self::Reference { inner, .. } => inner.contains_var(var),
            _ => false,
        }
    }

    /// Substitute a type variable with a type.
    pub fn substitute(&self, var: TypeVar, replacement: &InferType) -> Self {
        match self {
            Self::Var(v) if *v == var => replacement.clone(),
            Self::Var(_) => self.clone(),
            Self::Function {
                params,
                return_type,
                effects,
            } => Self::Function {
                params: params.iter().map(|p| p.substitute(var, replacement)).collect(),
                return_type: Box::new(return_type.substitute(var, replacement)),
                effects: effects.clone(),
            },
            Self::Generic { name, params } => Self::Generic {
                name: name.clone(),
                params: params.iter().map(|p| p.substitute(var, replacement)).collect(),
            },
            Self::Tuple(types) => {
                Self::Tuple(types.iter().map(|t| t.substitute(var, replacement)).collect())
            }
            Self::Array(inner) => Self::Array(Box::new(inner.substitute(var, replacement))),
            Self::Reference { inner, mutable } => Self::Reference {
                inner: Box::new(inner.substitute(var, replacement)),
                mutable: *mutable,
            },
            _ => self.clone(),
        }
    }
}

/// Type environment for tracking bindings.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Variable -> Type mappings
    bindings: HashMap<String, InferType>,
    /// Parent environment
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: TypeEnv) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, ty: InferType) {
        self.bindings.insert(name, ty);
    }

    pub fn get(&self, name: &str) -> Option<InferType> {
        if let Some(ty) = self.bindings.get(name) {
            return Some(ty.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.get(name);
        }
        None
    }
}

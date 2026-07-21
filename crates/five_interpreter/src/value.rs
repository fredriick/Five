//! Runtime values for the Five interpreter.

use five_ast::{Expr, Param};
use five_core::{FiveResult, Span};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::Environment;

/// A runtime value in Five.
#[derive(Clone)]
pub enum Value {
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Character value
    Char(char),
    /// Boolean value
    Bool(bool),
    /// Nil value (absence of value)
    Nil,
    /// Array value
    Array(Vec<Value>),
    /// Tuple value
    Tuple(Vec<Value>),
    /// Map/Dictionary value
    Map(Vec<(Value, Value)>),
    /// Range value
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    /// User-defined function
    Function {
        name: String,
        params: Vec<Param>,
        body: Expr,
        env: Rc<RefCell<Environment>>,
    },
    /// Built-in function
    BuiltinFunction {
        name: String,
        func: fn(Vec<Value>, Span) -> FiveResult<Value>,
    },
    /// Struct value
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    /// Struct constructor
    StructConstructor {
        name: String,
        fields: Vec<String>,
    },
    /// Data variant value
    DataVariant {
        type_name: String,
        variant: String,
        values: Vec<Value>,
    },
    /// Data constructor (for variants with fields)
    DataConstructor {
        type_name: String,
        variant: String,
        arity: usize,
    },
    /// Reference (immutable)
    Ref(Box<Value>),
    /// Mutable reference
    MutRef(Box<Value>),
    /// Return value (for control flow)
    Return(Box<Value>),
    /// Break value (for control flow)
    Break(Option<Box<Value>>),
    /// Continue (for control flow)
    Continue,
}

impl Value {
    /// Get the type name of this value.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::String(_) => "String",
            Self::Char(_) => "Char",
            Self::Bool(_) => "Bool",
            Self::Nil => "Nil",
            Self::Array(_) => "Array",
            Self::Tuple(_) => "Tuple",
            Self::Map(_) => "Map",
            Self::Range { .. } => "Range",
            Self::Function { .. } => "Function",
            Self::BuiltinFunction { .. } => "BuiltinFunction",
            Self::Struct { .. } => "Struct",
            Self::StructConstructor { .. } => "StructConstructor",
            Self::DataVariant { .. } => "DataVariant",
            Self::DataConstructor { .. } => "DataConstructor",
            Self::Ref(_) => "Ref",
            Self::MutRef(_) => "MutRef",
            Self::Return(_) => "Return",
            Self::Break(_) => "Break",
            Self::Continue => "Continue",
        }
    }

    /// Check if this value is truthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Nil => false,
            Self::Int(n) => *n != 0,
            Self::Float(n) => *n != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::Array(arr) => !arr.is_empty(),
            Self::Map(m) => !m.is_empty(),
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Char(a), Self::Char(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Nil, Self::Nil) => true,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Tuple(a), Self::Tuple(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            (
                Self::DataVariant {
                    type_name: tn1,
                    variant: v1,
                    values: vals1,
                },
                Self::DataVariant {
                    type_name: tn2,
                    variant: v2,
                    values: vals2,
                },
            ) => tn1 == tn2 && v1 == v2 && vals1 == vals2,
            (Self::Ref(a), Self::Ref(b)) => a == b,
            (Self::MutRef(a), Self::MutRef(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Char(c) => write!(f, "'{}'", c),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => write!(f, "nil"),
            Self::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", v)?;
                }
                write!(f, "]")
            }
            Self::Tuple(vals) => {
                write!(f, "(")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", v)?;
                }
                write!(f, ")")
            }
            Self::Map(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}: {:?}", k, v)?;
                }
                write!(f, "}}")
            }
            Self::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    write!(f, "{}..={}", start, end)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
            Self::Function { name, params, .. } => {
                write!(f, "<fn {}({})>", name, params.len())
            }
            Self::BuiltinFunction { name, .. } => {
                write!(f, "<builtin {}>", name)
            }
            Self::Struct { name, fields } => {
                write!(f, "{} {{ ", name)?;
                for (i, (field_name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {:?}", field_name, value)?;
                }
                write!(f, " }}")
            }
            Self::StructConstructor { name, .. } => {
                write!(f, "<struct {}>", name)
            }
            Self::DataVariant {
                variant, values, ..
            } => {
                if values.is_empty() {
                    write!(f, "{}", variant)
                } else {
                    write!(f, "{}(", variant)?;
                    for (i, v) in values.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{:?}", v)?;
                    }
                    write!(f, ")")
                }
            }
            Self::DataConstructor { variant, .. } => {
                write!(f, "<constructor {}>", variant)
            }
            Self::Ref(inner) => write!(f, "&{:?}", inner),
            Self::MutRef(inner) => write!(f, "&mut {:?}", inner),
            Self::Return(inner) => write!(f, "return {:?}", inner),
            Self::Break(inner) => match inner {
                Some(v) => write!(f, "break {:?}", v),
                None => write!(f, "break"),
            },
            Self::Continue => write!(f, "continue"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Char(c) => write!(f, "{}", c),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => write!(f, "nil"),
            Self::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Self::Tuple(vals) => {
                write!(f, "(")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Self::Map(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Self::DataVariant {
                variant, values, ..
            } => {
                if values.is_empty() {
                    write!(f, "{}", variant)
                } else {
                    write!(f, "{}(", variant)?;
                    for (i, v) in values.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", v)?;
                    }
                    write!(f, ")")
                }
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

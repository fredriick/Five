//! Error types for the Five programming language.

use crate::span::Span;
use thiserror::Error;

/// Result type for Five operations.
pub type FiveResult<T> = Result<T, FiveError>;

/// The main error type for Five.
#[derive(Debug, Error)]
pub enum FiveError {
    #[error("Lexer error at {span:?}: {message}")]
    LexerError { message: String, span: Span },

    #[error("Parser error at {span:?}: {message}")]
    ParserError { message: String, span: Span },

    #[error("Type error at {span:?}: {message}")]
    TypeError { message: String, span: Span },

    #[error("Runtime error at {span:?}: {message}")]
    RuntimeError { message: String, span: Span },

    #[error("Effect error at {span:?}: {message}")]
    EffectError { message: String, span: Span },

    #[error("Ownership error at {span:?}: {message}")]
    OwnershipError { message: String, span: Span },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Custom(String),
}

impl FiveError {
    /// Create a lexer error.
    pub fn lexer(message: impl Into<String>, span: Span) -> Self {
        Self::LexerError {
            message: message.into(),
            span,
        }
    }

    /// Create a parser error.
    pub fn parser(message: impl Into<String>, span: Span) -> Self {
        Self::ParserError {
            message: message.into(),
            span,
        }
    }

    /// Create a type error.
    pub fn type_error(message: impl Into<String>, span: Span) -> Self {
        Self::TypeError {
            message: message.into(),
            span,
        }
    }

    /// Create a runtime error.
    pub fn runtime(message: impl Into<String>, span: Span) -> Self {
        Self::RuntimeError {
            message: message.into(),
            span,
        }
    }

    /// Create an effect error.
    pub fn effect(message: impl Into<String>, span: Span) -> Self {
        Self::EffectError {
            message: message.into(),
            span,
        }
    }

    /// Create an ownership error.
    pub fn ownership(message: impl Into<String>, span: Span) -> Self {
        Self::OwnershipError {
            message: message.into(),
            span,
        }
    }

    /// Get the span associated with this error, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::LexerError { span, .. }
            | Self::ParserError { span, .. }
            | Self::TypeError { span, .. }
            | Self::RuntimeError { span, .. }
            | Self::EffectError { span, .. }
            | Self::OwnershipError { span, .. } => Some(*span),
            Self::IoError(_) | Self::Custom(_) => None,
        }
    }

    /// Get the error message.
    pub fn message(&self) -> String {
        match self {
            Self::LexerError { message, .. }
            | Self::ParserError { message, .. }
            | Self::TypeError { message, .. }
            | Self::RuntimeError { message, .. }
            | Self::EffectError { message, .. }
            | Self::OwnershipError { message, .. } => message.clone(),
            Self::IoError(e) => e.to_string(),
            Self::Custom(s) => s.clone(),
        }
    }
}

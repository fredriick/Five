//! Five Core - Shared utilities and error types for the Five programming language.

pub mod error;
pub mod span;

pub use error::{FiveError, FiveResult};
pub use span::{Span, Spanned};

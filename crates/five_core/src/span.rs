//! Source location tracking for error reporting.

use serde::{Deserialize, Serialize};

/// A span representing a range in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Create a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a dummy span for generated code.
    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Merge two spans into one covering both.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Get the length of this span.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::dummy()
    }
}

/// A value with an associated source span.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new spanned value.
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    /// Map the inner value while preserving the span.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }

    /// Get a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.node
    }

    /// Get a mutable reference to the inner value.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.node
    }

    /// Unwrap the spanned value, discarding the span.
    pub fn into_inner(self) -> T {
        self.node
    }
}

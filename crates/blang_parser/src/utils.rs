//! Utility functions and extensions.

use blang_lexer::Token;
use blang_span::{BytePos, Span};

/// Extension trait for BytePos.
pub trait BytePosExt {
    /// Create a span from this position to another.
    fn to(self, end: BytePos) -> Span;
}

impl BytePosExt for BytePos {
    fn to(self, end: BytePos) -> Span {
        Span::new(self, end)
    }
}

/// Extension trait for Token.
pub trait TokenExt {
    /// Get the span of this token.
    fn span(&self) -> Span;

    /// Get the end position of this token.
    fn end(&self) -> BytePos;
}

impl TokenExt for Token {
    fn span(&self) -> Span {
        Span::with_len(BytePos::from(self.start), self.len as u32)
    }

    fn end(&self) -> BytePos {
        BytePos::from(self.start + self.len)
    }
}

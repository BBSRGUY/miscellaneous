//! Parse error types.

use blang_lexer::TokenKind;
use blang_span::Span;
use std::fmt;
use thiserror::Error;

/// A parse error with location information.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct ParseError {
    /// The kind of error.
    pub kind: ParseErrorKind,
    /// The span where the error occurred.
    pub span: Span,
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        ParseError { kind, span }
    }

    /// Create an unexpected token error.
    pub fn unexpected_token(found: TokenKind, expected: Vec<TokenKind>, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken { found, expected },
            span,
        }
    }

    /// Create an unexpected end of file error.
    pub fn unexpected_eof(expected: Vec<TokenKind>, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEof { expected },
            span,
        }
    }

    /// Create a missing token error.
    pub fn missing_token(expected: TokenKind, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::MissingToken { expected },
            span,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.span)
    }
}

/// The kind of parse error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseErrorKind {
    /// Unexpected token.
    #[error("unexpected token {found:?}, expected one of {}", format_token_list(.expected))]
    UnexpectedToken {
        found: TokenKind,
        expected: Vec<TokenKind>,
    },

    /// Unexpected end of file.
    #[error("unexpected end of file, expected one of {}", format_token_list(.expected))]
    UnexpectedEof { expected: Vec<TokenKind> },

    /// Missing expected token.
    #[error("missing {expected:?}")]
    MissingToken { expected: TokenKind },

    /// Invalid literal.
    #[error("invalid literal: {message}")]
    InvalidLiteral { message: String },

    /// Invalid integer literal.
    #[error("invalid integer literal")]
    InvalidInteger,

    /// Invalid float literal.
    #[error("invalid float literal")]
    InvalidFloat,

    /// Invalid character literal.
    #[error("invalid character literal")]
    InvalidChar,

    /// Invalid escape sequence.
    #[error("invalid escape sequence")]
    InvalidEscape,

    /// Unterminated string literal.
    #[error("unterminated string literal")]
    UnterminatedString,

    /// Expected an expression.
    #[error("expected expression")]
    ExpectedExpression,

    /// Expected a pattern.
    #[error("expected pattern")]
    ExpectedPattern,

    /// Expected a type.
    #[error("expected type")]
    ExpectedType,

    /// Expected an item.
    #[error("expected item")]
    ExpectedItem,

    /// Expected an identifier.
    #[error("expected identifier")]
    ExpectedIdent,

    /// Expected a path.
    #[error("expected path")]
    ExpectedPath,

    /// Invalid assignment target.
    #[error("invalid assignment target")]
    InvalidAssignmentTarget,

    /// Duplicate field in struct literal.
    #[error("duplicate field `{field}` in struct literal")]
    DuplicateField { field: String },

    /// Expected one of the listed tokens.
    #[error("expected {message}")]
    Expected { message: String },

    /// Generic parse error.
    #[error("{message}")]
    Other { message: String },
}

/// Format a list of tokens for error messages.
fn format_token_list(tokens: &[TokenKind]) -> String {
    if tokens.is_empty() {
        return "nothing".to_string();
    }

    if tokens.len() == 1 {
        return format!("{:?}", tokens[0]);
    }

    let mut result = String::new();
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 {
            if i == tokens.len() - 1 {
                result.push_str(" or ");
            } else {
                result.push_str(", ");
            }
        }
        result.push_str(&format!("{:?}", token));
    }
    result
}

/// A type alias for parse results.
pub type ParseResult<T> = Result<T, ParseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_token_list() {
        assert_eq!(format_token_list(&[]), "nothing");
        assert_eq!(format_token_list(&[TokenKind::Plus]), "Plus");
        assert_eq!(
            format_token_list(&[TokenKind::Plus, TokenKind::Minus]),
            "Plus or Minus"
        );
        assert_eq!(
            format_token_list(&[TokenKind::Plus, TokenKind::Minus, TokenKind::Star]),
            "Plus, Minus or Star"
        );
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::unexpected_token(
            TokenKind::Plus,
            vec![TokenKind::Minus, TokenKind::Star],
            Span::from_offsets(0, 1),
        );

        let msg = err.to_string();
        assert!(msg.contains("unexpected token"));
        assert!(msg.contains("Plus"));
    }

    #[test]
    fn test_unexpected_eof() {
        let err = ParseError::unexpected_eof(vec![TokenKind::CloseBrace], Span::from_offsets(10, 10));

        match err.kind {
            ParseErrorKind::UnexpectedEof { ref expected } => {
                assert_eq!(expected.len(), 1);
                assert_eq!(expected[0], TokenKind::CloseBrace);
            }
            _ => panic!("Expected UnexpectedEof"),
        }
    }
}

//! Core parser implementation.

use crate::error::{ParseError, ParseErrorKind, ParseResult};
use crate::token_stream::TokenStream;
use crate::utils::BytePosExt;
use blang_ast::{GenericArgs, Ident, Path, PathSegment};
use blang_lexer::TokenKind;
use blang_span::Span;

/// The main parser struct.
pub struct Parser<'a> {
    /// The token stream.
    pub(crate) stream: TokenStream<'a>,
    /// The source code (for extracting text).
    pub(crate) source: &'a str,
}

impl<'a> Parser<'a> {
    /// Create a new parser from source code.
    pub fn new(source: &'a str) -> Self {
        Parser {
            stream: TokenStream::new(source),
            source,
        }
    }

    /// Get the source text for a span.
    pub(crate) fn source_text(&self, span: Span) -> &'a str {
        let start = span.start.as_usize();
        let end = span.end.as_usize();
        &self.source[start..end]
    }

    /// Parse an identifier.
    pub(crate) fn parse_ident(&mut self) -> ParseResult<Ident> {
        if let TokenKind::Ident = self.stream.peek_kind() {
            let token = self.stream.next();
            let name = self.source_text(token.span()).to_string();
            Ok(Ident::new(name, token.span()))
        } else {
            Err(ParseError::new(
                ParseErrorKind::ExpectedIdent,
                self.stream.current_span(),
            ))
        }
    }

    /// Parse a path (e.g., `foo`, `std::vec::Vec`).
    pub(crate) fn parse_path(&mut self) -> ParseResult<Path> {
        let start = self.stream.current_span().start;
        let mut segments = Vec::new();

        loop {
            let ident = self.parse_ident()?;
            segments.push(PathSegment {
                ident,
                generic_args: None,
            });

            if !self.stream.eat(TokenKind::ColonColon) {
                break;
            }
        }

        let end = self.stream.last_span().end;
        Ok(Path::new(segments, start.to(end)))
    }

    /// Parse a path with optional generic arguments.
    pub(crate) fn parse_path_with_generics(&mut self) -> ParseResult<Path> {
        let start = self.stream.current_span().start;
        let mut segments = Vec::new();

        loop {
            let ident = self.parse_ident()?;

            // Check for generic arguments
            let generic_args = if self.stream.at(TokenKind::Lt) {
                Some(self.parse_generic_args()?)
            } else {
                None
            };

            segments.push(PathSegment {
                ident,
                generic_args,
            });

            if !self.stream.eat(TokenKind::ColonColon) {
                break;
            }
        }

        let end = self.stream.last_span().end;
        Ok(Path::new(segments, start.to(end)))
    }

    /// Parse generic arguments (e.g., `<T, U>`).
    fn parse_generic_args(&mut self) -> ParseResult<GenericArgs> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::Lt).map_err(|_| {
            ParseError::missing_token(TokenKind::Lt, self.stream.current_span())
        })?;

        let mut args = Vec::new();
        args.push(self.parse_ty()?);

        while self.stream.eat(TokenKind::Comma) {
            if self.stream.at(TokenKind::Gt) {
                break;
            }
            args.push(self.parse_ty()?);
        }

        self.stream.expect(TokenKind::Gt).map_err(|_| {
            ParseError::missing_token(TokenKind::Gt, self.stream.current_span())
        })?;

        let end = self.stream.last_span().end;
        Ok(GenericArgs {
            args,
            span: start.to(end),
        })
    }

    /// Skip optional semicolon.
    pub(crate) fn skip_semi(&mut self) {
        self.stream.eat(TokenKind::Semi);
    }

    /// Expect a semicolon.
    pub(crate) fn expect_semi(&mut self) -> ParseResult<()> {
        self.stream.expect(TokenKind::Semi).map_err(|_| {
            ParseError::missing_token(TokenKind::Semi, self.stream.current_span())
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ident() {
        let mut parser = Parser::new("foo");
        let ident = parser.parse_ident().unwrap();
        assert_eq!(ident.name, "foo");
    }

    #[test]
    fn test_parse_simple_path() {
        let mut parser = Parser::new("foo");
        let path = parser.parse_path().unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.name, "foo");
    }

    #[test]
    fn test_parse_qualified_path() {
        let mut parser = Parser::new("std::vec::Vec");
        let path = parser.parse_path().unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[0].ident.name, "std");
        assert_eq!(path.segments[1].ident.name, "vec");
        assert_eq!(path.segments[2].ident.name, "Vec");
    }

    #[test]
    fn test_parse_path_with_generics() {
        let mut parser = Parser::new("Vec<i32>");
        let path = parser.parse_path_with_generics().unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.name, "Vec");
        assert!(path.segments[0].generic_args.is_some());
    }

    #[test]
    fn test_expected_ident_error() {
        let mut parser = Parser::new("123");
        let result = parser.parse_ident();
        assert!(result.is_err());
    }
}

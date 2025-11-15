//! Pattern parsing.

use crate::error::{ParseError, ParseErrorKind, ParseResult};
use crate::parser::Parser;
use blang_ast::{
    EnumPat, EnumVariantPat, FieldPat, IdentPat, Pat, PatKind, ReferencePat, StructPat, TuplePat,
    TupleStructPat,
};
use blang_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse a pattern.
    pub(crate) fn parse_pat(&mut self) -> ParseResult<Pat> {
        let start = self.stream.current_span().start;

        let kind = match self.stream.peek_kind() {
            // Wildcard pattern: _
            TokenKind::Underscore => {
                self.stream.next();
                PatKind::Wildcard
            }

            // Rest pattern: ..
            TokenKind::DotDot => {
                self.stream.next();
                PatKind::Rest
            }

            // Reference pattern: &pat or &mut pat
            TokenKind::And => {
                self.stream.next();
                let mutable = self.stream.eat(TokenKind::Mut);
                let pat = self.parse_pat()?;
                PatKind::Reference(Box::new(ReferencePat { mutable, pat }))
            }

            // Literal or path pattern
            TokenKind::Integer { .. }
            | TokenKind::Float { .. }
            | TokenKind::Char { .. }
            | TokenKind::String { .. }
            | TokenKind::True
            | TokenKind::False => {
                let lit = self.parse_literal()?;
                PatKind::Literal(lit)
            }

            // Identifier, struct, tuple struct, or enum pattern
            TokenKind::Ident => {
                self.parse_ident_or_path_pattern()?
            }

            // Tuple pattern: (p1, p2, ...)
            TokenKind::OpenParen => {
                self.stream.next();

                let mut elems = Vec::new();
                if !self.stream.at(TokenKind::CloseParen) {
                    elems.push(self.parse_pat()?);

                    while self.stream.eat(TokenKind::Comma) {
                        if self.stream.at(TokenKind::CloseParen) {
                            break;
                        }
                        if self.stream.at(TokenKind::DotDot) {
                            elems.push(self.parse_pat()?);
                            break;
                        }
                        elems.push(self.parse_pat()?);
                    }
                }

                self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                })?;

                PatKind::Tuple(TuplePat { elems })
            }

            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::ExpectedPattern,
                    self.stream.current_span(),
                ));
            }
        };

        let end = self.stream.last_span().end;
        let mut pat = Pat::new(kind, start.to(end));

        // Check for OR pattern: p1 | p2
        if self.stream.at(TokenKind::Pipe) {
            let mut patterns = vec![pat];
            while self.stream.eat(TokenKind::Pipe) {
                patterns.push(self.parse_pat()?);
            }
            let end = self.stream.last_span().end;
            pat = Pat::new(PatKind::Or(patterns), start.to(end));
        }

        Ok(pat)
    }

    /// Parse an identifier or path-based pattern.
    fn parse_ident_or_path_pattern(&mut self) -> ParseResult<PatKind> {
        let checkpoint = self.stream.checkpoint();
        let path = self.parse_path()?;

        // Check if this is a struct or tuple struct pattern
        if self.stream.at(TokenKind::OpenBrace) {
            // Struct pattern: Point { x, y }
            self.stream.next();

            let mut fields = Vec::new();
            let mut rest = false;

            while !self.stream.at(TokenKind::CloseBrace) {
                if self.stream.eat(TokenKind::DotDot) {
                    rest = true;
                    break;
                }

                let ident = self.parse_ident()?;
                let pat = if self.stream.eat(TokenKind::Colon) {
                    Some(self.parse_pat()?)
                } else {
                    None
                };

                fields.push(FieldPat { ident, pat });

                if !self.stream.eat(TokenKind::Comma) {
                    break;
                }
            }

            self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
            })?;

            Ok(PatKind::Struct(StructPat { path, fields, rest }))
        } else if self.stream.at(TokenKind::OpenParen) {
            // Tuple struct or enum pattern: Some(x)
            self.stream.next();

            let mut elems = Vec::new();
            if !self.stream.at(TokenKind::CloseParen) {
                elems.push(self.parse_pat()?);

                while self.stream.eat(TokenKind::Comma) {
                    if self.stream.at(TokenKind::CloseParen) {
                        break;
                    }
                    elems.push(self.parse_pat()?);
                }
            }

            self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
            })?;

            Ok(PatKind::TupleStruct(TupleStructPat { path, elems }))
        } else if path.segments.len() > 1 || self.stream.at(TokenKind::ColonColon) {
            // Multi-segment path - enum or const pattern
            Ok(PatKind::Enum(EnumPat {
                path,
                variant: EnumVariantPat::Unit,
            }))
        } else {
            // Simple identifier pattern
            // Restore and parse as identifier with possible subpattern
            self.stream.restore(checkpoint);
            let mutable = self.stream.eat(TokenKind::Mut);
            let ident = self.parse_ident()?;

            let subpattern = if self.stream.eat(TokenKind::At) {
                Some(Box::new(self.parse_pat()?))
            } else {
                None
            };

            Ok(PatKind::Ident(IdentPat {
                ident,
                mutable,
                subpattern,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_parse_wildcard_pattern() {
        let mut parser = Parser::new("_");
        let pat = parser.parse_pat().unwrap();
        assert!(matches!(pat.kind, PatKind::Wildcard));
    }

    #[test]
    fn test_parse_ident_pattern() {
        let mut parser = Parser::new("x");
        let pat = parser.parse_pat().unwrap();
        match pat.kind {
            PatKind::Ident(ref i) => {
                assert_eq!(i.ident.name, "x");
                assert!(!i.mutable);
            }
            _ => panic!("Expected identifier pattern"),
        }
    }

    #[test]
    fn test_parse_mut_ident_pattern() {
        let mut parser = Parser::new("mut y");
        let pat = parser.parse_pat().unwrap();
        match pat.kind {
            PatKind::Ident(ref i) => {
                assert_eq!(i.ident.name, "y");
                assert!(i.mutable);
            }
            _ => panic!("Expected identifier pattern"),
        }
    }

    #[test]
    fn test_parse_tuple_pattern() {
        let mut parser = Parser::new("(x, y, z)");
        let pat = parser.parse_pat().unwrap();
        match pat.kind {
            PatKind::Tuple(ref t) => {
                assert_eq!(t.elems.len(), 3);
            }
            _ => panic!("Expected tuple pattern"),
        }
    }

    #[test]
    fn test_parse_or_pattern() {
        let mut parser = Parser::new("x | y");
        let pat = parser.parse_pat().unwrap();
        match pat.kind {
            PatKind::Or(ref pats) => {
                assert_eq!(pats.len(), 2);
            }
            _ => panic!("Expected or pattern"),
        }
    }

    #[test]
    fn test_parse_reference_pattern() {
        let mut parser = Parser::new("&x");
        let pat = parser.parse_pat().unwrap();
        match pat.kind {
            PatKind::Reference(ref r) => {
                assert!(!r.mutable);
            }
            _ => panic!("Expected reference pattern"),
        }
    }

    #[test]
    fn test_parse_struct_pattern() {
        let mut parser = Parser::new("Point { x, y }");
        let pat = parser.parse_pat().unwrap();
        match pat.kind {
            PatKind::Struct(ref s) => {
                assert_eq!(s.fields.len(), 2);
                assert!(!s.rest);
            }
            _ => panic!("Expected struct pattern"),
        }
    }

    #[test]
    fn test_parse_tuple_struct_pattern() {
        let mut parser = Parser::new("Some(x)");
        let pat = parser.parse_pat().unwrap();
        match pat.kind {
            PatKind::TupleStruct(ref t) => {
                assert_eq!(t.path.segments[0].ident.name, "Some");
                assert_eq!(t.elems.len(), 1);
            }
            _ => panic!("Expected tuple struct pattern"),
        }
    }
}

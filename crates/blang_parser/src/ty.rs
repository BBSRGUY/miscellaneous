//! Type expression parsing.

use crate::error::{ParseError, ParseErrorKind, ParseResult};
use crate::parser::Parser;
use crate::utils::{BytePosExt, TokenExt};
use blang_ast::{ArrayLen, ArrayTy, FunctionTy, PointerTy, PrimitiveTy, ReferenceTy, Ty, TyKind};
use blang_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse a type expression.
    pub(crate) fn parse_ty(&mut self) -> ParseResult<Ty> {
        let start = self.stream.current_span().start;

        let kind = match self.stream.peek_kind() {

            // Reference types: &T or &mut T
            TokenKind::And => {
                self.stream.next();
                let mutable = self.stream.eat(TokenKind::Mut);
                let ty = self.parse_ty()?;
                TyKind::Reference(Box::new(ReferenceTy { mutable, ty }))
            }

            // Pointer types: *const T or *mut T
            TokenKind::Star => {
                self.stream.next();
                let mutable = if self.stream.eat(TokenKind::Mut) {
                    true
                } else if self.stream.eat(TokenKind::Const) {
                    false
                } else {
                    return Err(ParseError::new(
                        ParseErrorKind::Expected {
                            message: "'const' or 'mut' after '*'".to_string(),
                        },
                        self.stream.current_span(),
                    ));
                };
                let ty = self.parse_ty()?;
                TyKind::Pointer(Box::new(PointerTy { mutable, ty }))
            }

            // Array or slice: [T; N] or [T]
            TokenKind::OpenBracket => {
                self.stream.next();
                let elem_ty = self.parse_ty()?;

                if self.stream.eat(TokenKind::Semi) {
                    // Array type: [T; N]
                    let len = self.parse_array_len()?;
                    self.stream.expect(TokenKind::CloseBracket).map_err(|_| {
                        ParseError::missing_token(TokenKind::CloseBracket, self.stream.current_span())
                    })?;
                    TyKind::Array(Box::new(ArrayTy { elem_ty, len }))
                } else {
                    // Slice type: [T]
                    self.stream.expect(TokenKind::CloseBracket).map_err(|_| {
                        ParseError::missing_token(TokenKind::CloseBracket, self.stream.current_span())
                    })?;
                    TyKind::Slice(Box::new(elem_ty))
                }
            }

            // Tuple or unit type: (T, U) or ()
            TokenKind::OpenParen => {
                self.stream.next();

                if self.stream.eat(TokenKind::CloseParen) {
                    // Unit type: ()
                    TyKind::Primitive(PrimitiveTy::Unit)
                } else {
                    let mut types = vec![self.parse_ty()?];

                    while self.stream.eat(TokenKind::Comma) {
                        if self.stream.at(TokenKind::CloseParen) {
                            break;
                        }
                        types.push(self.parse_ty()?);
                    }

                    self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                        ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                    })?;

                    TyKind::Tuple(types)
                }
            }

            // Function type: fn(T, U) -> R
            TokenKind::Fn => {
                self.stream.next();
                self.stream.expect(TokenKind::OpenParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::OpenParen, self.stream.current_span())
                })?;

                let mut params = Vec::new();
                if !self.stream.at(TokenKind::CloseParen) {
                    params.push(self.parse_ty()?);

                    while self.stream.eat(TokenKind::Comma) {
                        if self.stream.at(TokenKind::CloseParen) {
                            break;
                        }
                        params.push(self.parse_ty()?);
                    }
                }

                self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                })?;

                let return_ty = if self.stream.eat(TokenKind::Arrow) {
                    Some(self.parse_ty()?)
                } else {
                    None
                };

                TyKind::Function(Box::new(FunctionTy { params, return_ty }))
            }

            // Never type: !
            TokenKind::Bang => {
                self.stream.next();
                TyKind::Never
            }

            // Path type (identifier or qualified path) or primitive type
            TokenKind::Ident => {
                // Check if it's a primitive type
                let token_start = self.stream.current_span().start;
                let token_len = self.stream.peek().len;
                let name = &self.source[token_start.0 as usize..(token_start.0 + token_len as u32) as usize];

                if let Some(prim) = PrimitiveTy::from_str(name) {
                    self.stream.next();
                    TyKind::Primitive(prim)
                } else {
                    let path = self.parse_path_with_generics()?;
                    TyKind::Path(path)
                }
            }

            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::ExpectedType,
                    self.stream.current_span(),
                ));
            }
        };

        let end = self.stream.last_span().end;
        Ok(Ty::new(kind, start.to(end)))
    }

    /// Parse an array length (literal or const path).
    fn parse_array_len(&mut self) -> ParseResult<ArrayLen> {
        if let TokenKind::Integer { .. } = self.stream.peek_kind() {
            let token = self.stream.next();
            let src = self.source_text(token.span());
            // Parse as u64 (simplified - just extract digits)
            let digits: String = src.chars().filter(|c| c.is_ascii_digit()).collect();
            let value = digits.parse::<u64>().map_err(|_| {
                ParseError::new(ParseErrorKind::InvalidInteger, token.span())
            })?;
            Ok(ArrayLen::Literal(value))
        } else if self.stream.at(TokenKind::Ident) {
            let path = self.parse_path()?;
            Ok(ArrayLen::Const(path))
        } else {
            Err(ParseError::new(
                ParseErrorKind::Expected {
                    message: "array length (integer or const)".to_string(),
                },
                self.stream.current_span(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_parse_primitive_types() {
        let mut parser = Parser::new("i32");
        let ty = parser.parse_ty().unwrap();
        assert!(matches!(ty.kind, TyKind::Primitive(PrimitiveTy::I32)));

        let mut parser = Parser::new("bool");
        let ty = parser.parse_ty().unwrap();
        assert!(matches!(ty.kind, TyKind::Primitive(PrimitiveTy::Bool)));
    }

    #[test]
    fn test_parse_reference_type() {
        let mut parser = Parser::new("&i32");
        let ty = parser.parse_ty().unwrap();
        match ty.kind {
            TyKind::Reference(ref r) => {
                assert!(!r.mutable);
                assert!(matches!(r.ty.kind, TyKind::Primitive(PrimitiveTy::I32)));
            }
            _ => panic!("Expected reference type"),
        }

        let mut parser = Parser::new("&mut bool");
        let ty = parser.parse_ty().unwrap();
        match ty.kind {
            TyKind::Reference(ref r) => {
                assert!(r.mutable);
            }
            _ => panic!("Expected reference type"),
        }
    }

    #[test]
    fn test_parse_array_type() {
        let mut parser = Parser::new("[i32; 10]");
        let ty = parser.parse_ty().unwrap();
        match ty.kind {
            TyKind::Array(ref a) => {
                assert!(matches!(a.elem_ty.kind, TyKind::Primitive(PrimitiveTy::I32)));
                assert_eq!(a.len, ArrayLen::Literal(10));
            }
            _ => panic!("Expected array type"),
        }
    }

    #[test]
    fn test_parse_slice_type() {
        let mut parser = Parser::new("[u8]");
        let ty = parser.parse_ty().unwrap();
        match ty.kind {
            TyKind::Slice(ref s) => {
                assert!(matches!(s.kind, TyKind::Primitive(PrimitiveTy::U8)));
            }
            _ => panic!("Expected slice type"),
        }
    }

    #[test]
    fn test_parse_tuple_type() {
        let mut parser = Parser::new("(i32, bool)");
        let ty = parser.parse_ty().unwrap();
        match ty.kind {
            TyKind::Tuple(ref types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected tuple type"),
        }
    }

    #[test]
    fn test_parse_unit_type() {
        let mut parser = Parser::new("()");
        let ty = parser.parse_ty().unwrap();
        assert!(matches!(ty.kind, TyKind::Primitive(PrimitiveTy::Unit)));
    }

    #[test]
    fn test_parse_function_type() {
        let mut parser = Parser::new("fn(i32, bool) -> u64");
        let ty = parser.parse_ty().unwrap();
        match ty.kind {
            TyKind::Function(ref f) => {
                assert_eq!(f.params.len(), 2);
                assert!(f.return_ty.is_some());
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_parse_never_type() {
        let mut parser = Parser::new("!");
        let ty = parser.parse_ty().unwrap();
        assert!(matches!(ty.kind, TyKind::Never));
    }
}

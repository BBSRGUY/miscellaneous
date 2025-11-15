//! Expression parsing with Pratt parser for precedence.

use crate::error::{ParseError, ParseErrorKind, ParseResult};
use crate::parser::Parser;
use crate::utils::{BytePosExt, TokenExt};
use blang_ast::*;
use blang_lexer::TokenKind;

/// Operator precedence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None = 0,
    Assignment = 1,     // = += -= etc.
    Range = 2,          // .. ..=
    LogicalOr = 3,      // ||
    LogicalAnd = 4,     // &&
    Comparison = 5,     // == != < <= > >=
    BitOr = 6,          // |
    BitXor = 7,         // ^
    BitAnd = 8,         // &
    Shift = 9,          // << >>
    Additive = 10,      // + -
    Multiplicative = 11, // * / %
    Cast = 12,          // as
    Unary = 13,         // - ! ~ * &
    Call = 14,          // () [] .
}

impl<'a> Parser<'a> {
    /// Parse an expression.
    pub(crate) fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_expr_with_precedence(Precedence::None)
    }

    /// Parse an expression with minimum precedence (Pratt parsing).
    fn parse_expr_with_precedence(&mut self, min_prec: Precedence) -> ParseResult<Expr> {
        let mut left = self.parse_prefix_expr()?;

        while !self.stream.at_eof() {
            let prec = self.infix_precedence();
            if prec < min_prec {
                break;
            }

            left = self.parse_infix_expr(left, prec)?;
        }

        Ok(left)
    }

    /// Get the precedence of the current infix operator.
    fn infix_precedence(&mut self) -> Precedence {
        match self.stream.peek_kind() {
            TokenKind::Eq
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq
            | TokenKind::AndEq
            | TokenKind::OrEq
            | TokenKind::CaretEq
            | TokenKind::ShlEq
            | TokenKind::ShrEq => Precedence::Assignment,

            TokenKind::DotDot | TokenKind::DotDotEq => Precedence::Range,
            TokenKind::OrOr => Precedence::LogicalOr,
            TokenKind::AndAnd => Precedence::LogicalAnd,
            TokenKind::EqEq | TokenKind::Ne | TokenKind::Lt | TokenKind::Le | TokenKind::Gt | TokenKind::Ge => {
                Precedence::Comparison
            }
            TokenKind::Or => Precedence::BitOr,
            TokenKind::Caret => Precedence::BitXor,
            TokenKind::And => Precedence::BitAnd,
            TokenKind::Shl | TokenKind::Shr => Precedence::Shift,
            TokenKind::Plus | TokenKind::Minus => Precedence::Additive,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Multiplicative,
            TokenKind::As => Precedence::Cast,
            TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }

    /// Parse a prefix expression (literal, identifier, unary op, etc.).
    fn parse_prefix_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        let kind = match self.stream.peek_kind() {
            // Literals
            TokenKind::Integer { .. }
            | TokenKind::Float { .. }
            | TokenKind::Char { .. }
            | TokenKind::String { .. }
            | TokenKind::TemplateString { .. }
            | TokenKind::True
            | TokenKind::False => {
                let lit = self.parse_literal()?;
                ExprKind::Literal(lit)
            }

            // Identifiers and paths
            TokenKind::Ident => {
                let path = self.parse_path_with_generics()?;

                // Check for struct literal
                if self.stream.at(TokenKind::OpenBrace) {
                    return self.parse_struct_expr(path);
                }

                ExprKind::Path(path)
            }

            // Parenthesized expression or tuple
            TokenKind::OpenParen => {
                return self.parse_paren_or_tuple();
            }

            // Array literal
            TokenKind::OpenBracket => {
                return self.parse_array_expr();
            }

            // Block expression
            TokenKind::OpenBrace => {
                let block = self.parse_block()?;
                ExprKind::Block(block)
            }

            // If expression
            TokenKind::If => {
                return self.parse_if_expr();
            }

            // Match expression
            TokenKind::Match => {
                return self.parse_match_expr();
            }

            // Loop expression
            TokenKind::Loop => {
                return self.parse_loop_expr();
            }

            // While expression
            TokenKind::While => {
                return self.parse_while_expr();
            }

            // For expression
            TokenKind::For => {
                return self.parse_for_expr();
            }

            // Unsafe block
            TokenKind::Unsafe => {
                self.stream.next();
                let block = self.parse_block()?;
                ExprKind::Unsafe(block)
            }

            // Return expression
            TokenKind::Return => {
                self.stream.next();
                let expr = if self.at_expr_end() {
                    None
                } else {
                    Some(Box::new(self.parse_expr()?))
                };
                ExprKind::Return(expr)
            }

            // Break expression
            TokenKind::Break => {
                self.stream.next();
                let expr = if self.at_expr_end() {
                    None
                } else {
                    Some(Box::new(self.parse_expr()?))
                };
                ExprKind::Break(expr)
            }

            // Continue expression
            TokenKind::Continue => {
                self.stream.next();
                ExprKind::Continue
            }

            // Unary operators: - ! ~
            TokenKind::Minus => {
                self.stream.next();
                let expr = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary(UnaryExpr {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }

            TokenKind::Bang => {
                self.stream.next();
                let expr = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary(UnaryExpr {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }

            TokenKind::Tilde => {
                self.stream.next();
                let expr = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary(UnaryExpr {
                    op: UnaryOp::BitNot,
                    expr: Box::new(expr),
                })
            }

            // Reference: &expr or &mut expr
            TokenKind::And => {
                self.stream.next();
                let mutable = self.stream.eat(TokenKind::Mut);
                let expr = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Reference(Box::new(ReferenceExpr { mutable, expr }))
            }

            // Dereference: *expr
            TokenKind::Star => {
                self.stream.next();
                let expr = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Dereference(Box::new(expr))
            }

            // Closure: |params| expr
            TokenKind::Or => {
                return self.parse_closure_expr();
            }

            // Signal, effect, memo expressions
            TokenKind::Signal => {
                self.stream.next();
                self.stream.expect(TokenKind::OpenParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::OpenParen, self.stream.current_span())
                })?;
                let expr = self.parse_expr()?;
                self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                })?;
                ExprKind::Signal(Box::new(expr))
            }

            TokenKind::Effect => {
                self.stream.next();
                self.stream.expect(TokenKind::OpenParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::OpenParen, self.stream.current_span())
                })?;
                let expr = self.parse_expr()?;
                self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                })?;
                ExprKind::Effect(Box::new(expr))
            }

            TokenKind::Memo => {
                self.stream.next();
                self.stream.expect(TokenKind::OpenParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::OpenParen, self.stream.current_span())
                })?;
                let expr = self.parse_expr()?;
                self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                })?;
                ExprKind::Memo(Box::new(expr))
            }

            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::ExpectedExpression,
                    self.stream.current_span(),
                ));
            }
        };

        let end = self.stream.last_span().end;
        Ok(Expr::new(kind, start.to(end)))
    }

    /// Parse an infix expression (binary operators, calls, field access, etc.).
    fn parse_infix_expr(&mut self, left: Expr, prec: Precedence) -> ParseResult<Expr> {
        let start = left.span.start;

        match self.stream.peek_kind() {
            // Binary operators
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqEq
            | TokenKind::Ne
            | TokenKind::Lt
            | TokenKind::Le
            | TokenKind::Gt
            | TokenKind::Ge
            | TokenKind::AndAnd
            | TokenKind::OrOr
            | TokenKind::And
            | TokenKind::Or
            | TokenKind::Caret
            | TokenKind::Shl
            | TokenKind::Shr
            | TokenKind::Eq
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq
            | TokenKind::AndEq
            | TokenKind::OrEq
            | TokenKind::CaretEq
            | TokenKind::ShlEq
            | TokenKind::ShrEq => {
                let kind = self.stream.peek_kind();
                self.stream.next();
                let op = self.token_to_binary_op(kind);
                let right = self.parse_expr_with_precedence(prec.next())?;
                let end = right.span.end;
                Ok(Expr::new(
                    ExprKind::Binary(BinaryExpr {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    }),
                    start.to(end),
                ))
            }

            // Type cast: as
            TokenKind::As => {
                self.stream.next();
                let ty = self.parse_ty()?;
                let end = ty.span.end;
                Ok(Expr::new(
                    ExprKind::Cast(Box::new(CastExpr { expr: left, ty })),
                    start.to(end),
                ))
            }

            // Range: .. or ..=
            TokenKind::DotDot | TokenKind::DotDotEq => {
                let inclusive = self.stream.peek_kind() == TokenKind::DotDotEq;
                self.stream.next();
                let end_expr = if self.at_expr_end() {
                    None
                } else {
                    Some(self.parse_expr_with_precedence(prec.next())?)
                };
                let end = end_expr.as_ref().map(|e| e.span.end).unwrap_or(self.stream.last_span().end);
                Ok(Expr::new(
                    ExprKind::Range(Box::new(RangeExpr {
                        start: Some(left),
                        end: end_expr,
                        inclusive,
                    })),
                    start.to(end),
                ))
            }

            // Function call: expr(args)
            TokenKind::OpenParen => {
                self.stream.next();
                let args = self.parse_call_args()?;
                self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                })?;
                let end = self.stream.last_span().end;
                Ok(Expr::new(
                    ExprKind::Call(Box::new(CallExpr { func: left, args })),
                    start.to(end),
                ))
            }

            // Index: expr[index]
            TokenKind::OpenBracket => {
                self.stream.next();
                let index = self.parse_expr()?;
                self.stream.expect(TokenKind::CloseBracket).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseBracket, self.stream.current_span())
                })?;
                let end = self.stream.last_span().end;
                Ok(Expr::new(
                    ExprKind::Index(Box::new(IndexExpr { expr: left, index })),
                    start.to(end),
                ))
            }

            // Field access or method call: expr.field or expr.method(args)
            TokenKind::Dot => {
                self.stream.next();

                // Check for await
                if self.stream.at(TokenKind::Await) {
                    self.stream.next();
                    let end = self.stream.last_span().end;
                    return Ok(Expr::new(ExprKind::Await(Box::new(left)), start.to(end)));
                }

                let ident = self.parse_ident()?;

                // Check if this is a method call
                if self.stream.at(TokenKind::OpenParen) {
                    self.stream.next();
                    let args = self.parse_call_args()?;
                    self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                        ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                    })?;
                    let end = self.stream.last_span().end;
                    Ok(Expr::new(
                        ExprKind::MethodCall(Box::new(MethodCallExpr {
                            receiver: left,
                            method: ident,
                            generic_args: None,
                            args,
                        })),
                        start.to(end),
                    ))
                } else {
                    // Field access
                    let end = ident.span.end;
                    Ok(Expr::new(
                        ExprKind::Field(Box::new(FieldExpr {
                            expr: left,
                            field: Field::Named(ident),
                        })),
                        start.to(end),
                    ))
                }
            }

            _ => Ok(left),
        }
    }

    /// Convert a token kind to a binary operator.
    fn token_to_binary_op(&self, kind: TokenKind) -> BinaryOp {
        match kind {
            TokenKind::Plus => BinaryOp::Add,
            TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Star => BinaryOp::Mul,
            TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Rem,
            TokenKind::EqEq => BinaryOp::Eq,
            TokenKind::Ne => BinaryOp::Ne,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Ge => BinaryOp::Ge,
            TokenKind::AndAnd => BinaryOp::And,
            TokenKind::OrOr => BinaryOp::Or,
            TokenKind::And => BinaryOp::BitAnd,
            TokenKind::Or => BinaryOp::BitOr,
            TokenKind::Caret => BinaryOp::BitXor,
            TokenKind::Shl => BinaryOp::Shl,
            TokenKind::Shr => BinaryOp::Shr,
            TokenKind::Eq => BinaryOp::Assign,
            TokenKind::PlusEq => BinaryOp::AddAssign,
            TokenKind::MinusEq => BinaryOp::SubAssign,
            TokenKind::StarEq => BinaryOp::MulAssign,
            TokenKind::SlashEq => BinaryOp::DivAssign,
            TokenKind::PercentEq => BinaryOp::RemAssign,
            TokenKind::AndEq => BinaryOp::BitAndAssign,
            TokenKind::OrEq => BinaryOp::BitOrAssign,
            TokenKind::CaretEq => BinaryOp::BitXorAssign,
            TokenKind::ShlEq => BinaryOp::ShlAssign,
            TokenKind::ShrEq => BinaryOp::ShrAssign,
            _ => panic!("Invalid binary operator: {:?}", kind),
        }
    }

    /// Check if we're at the end of an expression.
    fn at_expr_end(&mut self) -> bool {
        matches!(
            self.stream.peek_kind(),
            TokenKind::Semi
                | TokenKind::Comma
                | TokenKind::CloseParen
                | TokenKind::CloseBracket
                | TokenKind::CloseBrace
                | TokenKind::Eof
        )
    }

    /// Parse call arguments.
    fn parse_call_args(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();

        if !self.stream.at(TokenKind::CloseParen) {
            args.push(self.parse_expr()?);

            while self.stream.eat(TokenKind::Comma) {
                if self.stream.at(TokenKind::CloseParen) {
                    break;
                }
                args.push(self.parse_expr()?);
            }
        }

        Ok(args)
    }

    /// Parse a literal expression.
    pub(crate) fn parse_literal(&mut self) -> ParseResult<Lit> {
        let token = self.stream.next();
        let src = self.source_text(token.span());

        match token.kind {
            TokenKind::Integer { base, .. } => {
                let (raw, suffix) = self.split_suffix(src);
                Ok(Lit::int(
                    raw.to_string(),
                    match base {
                        blang_lexer::Base::Decimal => IntBase::Decimal,
                        blang_lexer::Base::Hexadecimal => IntBase::Hexadecimal,
                        blang_lexer::Base::Octal => IntBase::Octal,
                        blang_lexer::Base::Binary => IntBase::Binary,
                    },
                    suffix.map(|s| s.to_string()),
                ))
            }

            TokenKind::Float { .. } => {
                let (raw, suffix) = self.split_suffix(src);
                Ok(Lit::float(raw.to_string(), suffix.map(|s| s.to_string())))
            }

            TokenKind::True => Ok(Lit::bool(true)),
            TokenKind::False => Ok(Lit::bool(false)),

            TokenKind::Char { .. } => {
                let ch = self.parse_char_value(src)?;
                Ok(Lit::char(ch, src.to_string()))
            }

            TokenKind::String { raw, .. } => {
                let value = if raw {
                    src[2..src.len() - 1].to_string()
                } else {
                    self.parse_string_value(src)?
                };
                Ok(Lit::string(value, src.to_string(), raw))
            }

            TokenKind::TemplateString { .. } => {
                let parts = self.parse_template_parts(src)?;
                Ok(Lit::template(parts))
            }

            _ => Err(ParseError::new(
                ParseErrorKind::InvalidLiteral {
                    message: format!("unexpected token {:?}", token.kind),
                },
                token.span(),
            )),
        }
    }

    fn split_suffix<'b>(&self, src: &'b str) -> (&'b str, Option<&'b str>) {
        for (i, ch) in src.chars().enumerate() {
            if ch.is_ascii_alphabetic() && i > 0 {
                return (&src[..i], Some(&src[i..]));
            }
        }
        (src, None)
    }

    fn parse_char_value(&self, src: &str) -> ParseResult<char> {
        let inner = &src[1..src.len() - 1];
        if inner.starts_with('\\') {
            // Handle escape sequence (simplified)
            match &inner[1..] {
                "n" => Ok('\n'),
                "r" => Ok('\r'),
                "t" => Ok('\t'),
                "\\" => Ok('\\'),
                "'" => Ok('\''),
                "\"" => Ok('"'),
                "0" => Ok('\0'),
                _ => Err(ParseError::new(
                    ParseErrorKind::InvalidEscape,
                    Span::DUMMY,
                )),
            }
        } else {
            inner.chars().next().ok_or_else(|| {
                ParseError::new(ParseErrorKind::InvalidChar, Span::DUMMY)
            })
        }
    }

    fn parse_string_value(&self, src: &str) -> ParseResult<String> {
        let inner = &src[1..src.len() - 1];
        Ok(inner.replace("\\n", "\n").replace("\\t", "\t").replace("\\\\", "\\"))
    }

    fn parse_template_parts(&self, _src: &str) -> ParseResult<Vec<TemplatePart>> {
        // Simplified - just return empty for now
        Ok(vec![])
    }
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Range,
            Precedence::Range => Precedence::LogicalOr,
            Precedence::LogicalOr => Precedence::LogicalAnd,
            Precedence::LogicalAnd => Precedence::Comparison,
            Precedence::Comparison => Precedence::BitOr,
            Precedence::BitOr => Precedence::BitXor,
            Precedence::BitXor => Precedence::BitAnd,
            Precedence::BitAnd => Precedence::Shift,
            Precedence::Shift => Precedence::Additive,
            Precedence::Additive => Precedence::Multiplicative,
            Precedence::Multiplicative => Precedence::Cast,
            Precedence::Cast => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Call,
        }
    }
}

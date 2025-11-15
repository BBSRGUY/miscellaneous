//! Expression parsing helpers.

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use blang_ast::*;
use blang_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse a block expression: { stmts }
    pub(crate) fn parse_block(&mut self) -> ParseResult<Block> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let mut stmts = Vec::new();
        let mut expr = None;

        while !self.stream.at(TokenKind::CloseBrace) && !self.stream.at_eof() {
            // Try to parse a statement
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);

            // Check if the last statement is an expression without semicolon
            if let Some(last) = stmts.last() {
                if let StmtKind::Expr(ref e, false) = last.kind {
                    // This is the block's value
                    expr = Some(Box::new(e.clone()));
                    stmts.pop();
                    break;
                }
            }
        }

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        let end = self.stream.last_span().end;
        Ok(Block {
            stmts,
            expr,
            span: start.to(end),
        })
    }

    /// Parse an if expression.
    pub(crate) fn parse_if_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::If).map_err(|_| {
            ParseError::missing_token(TokenKind::If, self.stream.current_span())
        })?;

        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_block = if self.stream.eat(TokenKind::Else) {
            if self.stream.at(TokenKind::If) {
                Some(Box::new(self.parse_if_expr()?))
            } else {
                Some(Box::new(Expr::block(self.parse_block()?, self.stream.last_span())))
            }
        } else {
            None
        };

        let end = self.stream.last_span().end;
        Ok(Expr::new(
            ExprKind::If(Box::new(IfExpr {
                cond,
                then_block,
                else_block,
            })),
            start.to(end),
        ))
    }

    /// Parse a match expression.
    pub(crate) fn parse_match_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::Match).map_err(|_| {
            ParseError::missing_token(TokenKind::Match, self.stream.current_span())
        })?;

        let expr = self.parse_expr()?;

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let mut arms = Vec::new();

        while !self.stream.at(TokenKind::CloseBrace) && !self.stream.at_eof() {
            let pat = self.parse_pat()?;

            let guard = if self.stream.eat(TokenKind::If) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            self.stream.expect(TokenKind::FatArrow).map_err(|_| {
                ParseError::missing_token(TokenKind::FatArrow, self.stream.current_span())
            })?;

            let body = self.parse_expr()?;

            arms.push(MatchArm { pat, guard, body });

            if !self.stream.eat(TokenKind::Comma) {
                break;
            }
        }

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        let end = self.stream.last_span().end;
        Ok(Expr::new(
            ExprKind::Match(Box::new(MatchExpr { expr, arms })),
            start.to(end),
        ))
    }

    /// Parse a loop expression.
    pub(crate) fn parse_loop_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::Loop).map_err(|_| {
            ParseError::missing_token(TokenKind::Loop, self.stream.current_span())
        })?;

        let body = self.parse_block()?;
        let end = self.stream.last_span().end;

        Ok(Expr::new(
            ExprKind::Loop(Box::new(LoopExpr { body })),
            start.to(end),
        ))
    }

    /// Parse a while expression.
    pub(crate) fn parse_while_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::While).map_err(|_| {
            ParseError::missing_token(TokenKind::While, self.stream.current_span())
        })?;

        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let end = self.stream.last_span().end;

        Ok(Expr::new(
            ExprKind::While(Box::new(WhileExpr { cond, body })),
            start.to(end),
        ))
    }

    /// Parse a for expression.
    pub(crate) fn parse_for_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::For).map_err(|_| {
            ParseError::missing_token(TokenKind::For, self.stream.current_span())
        })?;

        let pat = self.parse_pat()?;

        self.stream.expect(TokenKind::In).map_err(|_| {
            ParseError::missing_token(TokenKind::In, self.stream.current_span())
        })?;

        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        let end = self.stream.last_span().end;

        Ok(Expr::new(
            ExprKind::For(Box::new(ForExpr { pat, iter, body })),
            start.to(end),
        ))
    }

    /// Parse a closure expression: |params| expr
    pub(crate) fn parse_closure_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::Pipe).map_err(|_| {
            ParseError::missing_token(TokenKind::Pipe, self.stream.current_span())
        })?;

        let mut params = Vec::new();

        if !self.stream.at(TokenKind::Pipe) {
            loop {
                let pat = self.parse_pat()?;
                let ty = if self.stream.eat(TokenKind::Colon) {
                    Some(self.parse_ty()?)
                } else {
                    None
                };

                params.push(ClosureParam { pat, ty });

                if !self.stream.eat(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.stream.expect(TokenKind::Pipe).map_err(|_| {
            ParseError::missing_token(TokenKind::Pipe, self.stream.current_span())
        })?;

        let body = if self.stream.at(TokenKind::OpenBrace) {
            Expr::block(self.parse_block()?, self.stream.last_span())
        } else {
            self.parse_expr()?
        };

        let end = self.stream.last_span().end;
        Ok(Expr::new(
            ExprKind::Closure(Box::new(ClosureExpr { params, body })),
            start.to(end),
        ))
    }

    /// Parse a parenthesized expression or tuple.
    pub(crate) fn parse_paren_or_tuple(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::OpenParen).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenParen, self.stream.current_span())
        })?;

        if self.stream.eat(TokenKind::CloseParen) {
            // Unit value ()
            let end = self.stream.last_span().end;
            return Ok(Expr::new(ExprKind::Tuple(vec![]), start.to(end)));
        }

        let first = self.parse_expr()?;

        if self.stream.eat(TokenKind::Comma) {
            // Tuple
            let mut elems = vec![first];

            while !self.stream.at(TokenKind::CloseParen) {
                elems.push(self.parse_expr()?);

                if !self.stream.eat(TokenKind::Comma) {
                    break;
                }
            }

            self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
            })?;

            let end = self.stream.last_span().end;
            Ok(Expr::new(ExprKind::Tuple(elems), start.to(end)))
        } else {
            // Parenthesized expression
            self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
            })?;

            let end = self.stream.last_span().end;
            Ok(Expr::new(ExprKind::Paren(Box::new(first)), start.to(end)))
        }
    }

    /// Parse an array literal.
    pub(crate) fn parse_array_expr(&mut self) -> ParseResult<Expr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::OpenBracket).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBracket, self.stream.current_span())
        })?;

        if self.stream.eat(TokenKind::CloseBracket) {
            let end = self.stream.last_span().end;
            return Ok(Expr::new(ExprKind::Array(ArrayExpr::List(vec![])), start.to(end)));
        }

        let first = self.parse_expr()?;

        if self.stream.eat(TokenKind::Semi) {
            // Repeat syntax: [elem; len]
            let len = self.parse_expr()?;

            self.stream.expect(TokenKind::CloseBracket).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseBracket, self.stream.current_span())
            })?;

            let end = self.stream.last_span().end;
            Ok(Expr::new(
                ExprKind::Array(ArrayExpr::Repeat {
                    elem: Box::new(first),
                    len: Box::new(len),
                }),
                start.to(end),
            ))
        } else {
            // List syntax: [elem1, elem2, ...]
            let mut elems = vec![first];

            while self.stream.eat(TokenKind::Comma) {
                if self.stream.at(TokenKind::CloseBracket) {
                    break;
                }
                elems.push(self.parse_expr()?);
            }

            self.stream.expect(TokenKind::CloseBracket).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseBracket, self.stream.current_span())
            })?;

            let end = self.stream.last_span().end;
            Ok(Expr::new(ExprKind::Array(ArrayExpr::List(elems)), start.to(end)))
        }
    }

    /// Parse a struct literal expression.
    pub(crate) fn parse_struct_expr(&mut self, path: Path) -> ParseResult<Expr> {
        let start = path.span.start;

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let mut fields = Vec::new();
        let mut rest = None;

        while !self.stream.at(TokenKind::CloseBrace) && !self.stream.at_eof() {
            if self.stream.eat(TokenKind::DotDot) {
                rest = Some(Box::new(self.parse_expr()?));
                break;
            }

            let ident = self.parse_ident()?;

            let expr = if self.stream.eat(TokenKind::Colon) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            fields.push(FieldInit { ident, expr });

            if !self.stream.eat(TokenKind::Comma) {
                break;
            }
        }

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        let end = self.stream.last_span().end;
        Ok(Expr::new(
            ExprKind::Struct(StructExpr { path, fields, rest }),
            start.to(end),
        ))
    }
}

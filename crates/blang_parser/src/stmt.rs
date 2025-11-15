//! Statement parsing.

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use blang_ast::{LetStmt, Stmt, StmtKind};
use blang_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse a statement.
    pub(crate) fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.stream.current_span().start;

        // Check for let statement
        if self.stream.at(TokenKind::Let) {
            return self.parse_let_stmt();
        }

        // Check for item statements (fn, struct, enum, etc.)
        if self.at_item_start() {
            let item = self.parse_item()?;
            let end = item.span.end;
            return Ok(Stmt::new(StmtKind::Item(item), start.to(end)));
        }

        // Otherwise, parse as expression statement
        let expr = self.parse_expr()?;
        let has_semi = self.stream.eat(TokenKind::Semi);
        let end = self.stream.last_span().end;

        Ok(Stmt::new(StmtKind::Expr(expr, has_semi), start.to(end)))
    }

    /// Parse a let statement.
    fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::Let).map_err(|_| {
            ParseError::missing_token(TokenKind::Let, self.stream.current_span())
        })?;

        let mutable = self.stream.eat(TokenKind::Mut);
        let pat = self.parse_pat()?;

        let ty = if self.stream.eat(TokenKind::Colon) {
            Some(self.parse_ty()?)
        } else {
            None
        };

        let init = if self.stream.eat(TokenKind::Eq) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.expect_semi()?;

        let end = self.stream.last_span().end;
        Ok(Stmt::new(
            StmtKind::Let(LetStmt {
                pat,
                ty,
                init,
                mutable,
            }),
            start.to(end),
        ))
    }

    /// Check if we're at the start of an item.
    fn at_item_start(&mut self) -> bool {
        matches!(
            self.stream.peek_kind(),
            TokenKind::Fn
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Trait
                | TokenKind::Impl
                | TokenKind::Type
                | TokenKind::Const
                | TokenKind::Static
                | TokenKind::Pub
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_parse_let_stmt() {
        let mut parser = Parser::new("let x = 42;");
        let stmt = parser.parse_stmt().unwrap();
        match stmt.kind {
            StmtKind::Let(ref l) => {
                assert!(!l.mutable);
                assert!(l.ty.is_none());
                assert!(l.init.is_some());
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_let_mut_stmt() {
        let mut parser = Parser::new("let mut y: i32 = 10;");
        let stmt = parser.parse_stmt().unwrap();
        match stmt.kind {
            StmtKind::Let(ref l) => {
                assert!(l.mutable);
                assert!(l.ty.is_some());
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_expr_stmt() {
        let mut parser = Parser::new("x + 1;");
        let stmt = parser.parse_stmt().unwrap();
        match stmt.kind {
            StmtKind::Expr(_, has_semi) => {
                assert!(has_semi);
            }
            _ => panic!("Expected expression statement"),
        }
    }
}

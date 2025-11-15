//! Statement types for the AST.

use crate::expr::Expr;
use crate::pat::Pat;
use crate::ty::Ty;
use blang_span::Span;
use serde::{Deserialize, Serialize};

/// A statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Stmt {
    /// The kind of statement.
    pub kind: StmtKind,
    /// The span of this statement in the source.
    pub span: Span,
}

impl Stmt {
    /// Create a new statement with the given kind and span.
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Stmt { kind, span }
    }

    /// Create a let statement.
    pub fn let_stmt(
        pat: Pat,
        ty: Option<Ty>,
        init: Option<Expr>,
        mutable: bool,
        span: Span,
    ) -> Self {
        Stmt {
            kind: StmtKind::Let(LetStmt {
                pat,
                ty,
                init: init.map(Box::new),
                mutable,
            }),
            span,
        }
    }

    /// Create an expression statement.
    pub fn expr(expr: Expr, has_semi: bool) -> Self {
        let span = expr.span;
        Stmt {
            kind: StmtKind::Expr(expr, has_semi),
            span,
        }
    }

    /// Create an item statement.
    pub fn item(item: super::item::Item) -> Self {
        let span = item.span;
        Stmt {
            kind: StmtKind::Item(item),
            span,
        }
    }
}

/// The kind of a statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StmtKind {
    /// A let binding statement (e.g., `let x = 5;`, `let mut y: i32;`)
    Let(LetStmt),

    /// An expression statement (with or without semicolon)
    Expr(Expr, bool),

    /// An item statement (function, struct, etc. defined locally)
    Item(super::item::Item),

    /// A statement that couldn't be parsed (error recovery)
    Error,
}

/// A let binding statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LetStmt {
    /// The pattern being bound.
    pub pat: Pat,
    /// Optional type annotation.
    pub ty: Option<Ty>,
    /// Optional initializer expression.
    pub init: Option<Box<Expr>>,
    /// Whether this binding is mutable.
    pub mutable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lit::Lit;
    use crate::path::Ident;

    #[test]
    fn test_let_stmt() {
        let pat = Pat::ident(Ident::dummy("x".to_string()), false);
        let init = Expr::lit(Lit::decimal("42".to_string()), Span::DUMMY);
        let stmt = Stmt::let_stmt(pat.clone(), None, Some(init), false, Span::DUMMY);

        match stmt.kind {
            StmtKind::Let(ref l) => {
                assert_eq!(l.pat, pat);
                assert!(l.ty.is_none());
                assert!(l.init.is_some());
                assert!(!l.mutable);
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_mut_let_stmt() {
        let pat = Pat::ident(Ident::dummy("y".to_string()), true);
        let stmt = Stmt::let_stmt(pat, None, None, true, Span::DUMMY);

        match stmt.kind {
            StmtKind::Let(ref l) => {
                assert!(l.mutable);
                assert!(l.init.is_none());
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_expr_stmt() {
        let expr = Expr::lit(Lit::bool(true), Span::DUMMY);
        let stmt = Stmt::expr(expr.clone(), true);

        match stmt.kind {
            StmtKind::Expr(ref e, has_semi) => {
                assert_eq!(e, &expr);
                assert!(has_semi);
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_expr_stmt_no_semi() {
        let expr = Expr::lit(Lit::decimal("10".to_string()), Span::DUMMY);
        let stmt = Stmt::expr(expr, false);

        match stmt.kind {
            StmtKind::Expr(_, has_semi) => {
                assert!(!has_semi);
            }
            _ => panic!("Expected expression statement"),
        }
    }
}

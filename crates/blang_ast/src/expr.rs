//! Expression types for the AST.

use crate::lit::Lit;
use crate::pat::Pat;
use crate::path::{Ident, Path};
use crate::stmt::Stmt;
use crate::ty::Ty;
use blang_span::Span;
use serde::{Deserialize, Serialize};

/// An expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Expr {
    /// The kind of expression.
    pub kind: ExprKind,
    /// The span of this expression in the source.
    pub span: Span,
}

impl Expr {
    /// Create a new expression with the given kind and span.
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Expr { kind, span }
    }

    /// Create a literal expression.
    pub fn lit(lit: Lit, span: Span) -> Self {
        Expr {
            kind: ExprKind::Literal(lit),
            span,
        }
    }

    /// Create a path expression.
    pub fn path(path: Path) -> Self {
        let span = path.span;
        Expr {
            kind: ExprKind::Path(path),
            span,
        }
    }

    /// Create a block expression.
    pub fn block(block: Block, span: Span) -> Self {
        Expr {
            kind: ExprKind::Block(block),
            span,
        }
    }
}

/// The kind of an expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExprKind {
    /// A literal expression (e.g., `42`, `true`, `"hello"`)
    Literal(Lit),

    /// A path expression (e.g., `foo`, `std::vec::Vec`)
    Path(Path),

    /// A binary operation (e.g., `a + b`, `x == y`)
    Binary(BinaryExpr),

    /// A unary operation (e.g., `-x`, `!flag`)
    Unary(UnaryExpr),

    /// A function call (e.g., `foo(1, 2)`)
    Call(Box<CallExpr>),

    /// A method call (e.g., `obj.method(arg)`)
    MethodCall(Box<MethodCallExpr>),

    /// A field access (e.g., `obj.field`)
    Field(Box<FieldExpr>),

    /// An index operation (e.g., `arr[0]`)
    Index(Box<IndexExpr>),

    /// A range expression (e.g., `1..10`, `..=100`)
    Range(Box<RangeExpr>),

    /// A closure expression (e.g., `|x| x + 1`)
    Closure(Box<ClosureExpr>),

    /// A return expression (e.g., `return 42`)
    Return(Option<Box<Expr>>),

    /// A break expression (e.g., `break`, `break value`)
    Break(Option<Box<Expr>>),

    /// A continue expression
    Continue,

    /// An array literal (e.g., `[1, 2, 3]`)
    Array(ArrayExpr),

    /// A tuple literal (e.g., `(1, true, "hello")`)
    Tuple(Vec<Expr>),

    /// A struct literal (e.g., `Point { x: 1, y: 2 }`)
    Struct(StructExpr),

    /// An await expression (e.g., `future.await`)
    Await(Box<Expr>),

    /// A type cast (e.g., `value as i32`)
    Cast(Box<CastExpr>),

    /// A reference expression (e.g., `&x`, `&mut y`)
    Reference(Box<ReferenceExpr>),

    /// A dereference expression (e.g., `*ptr`)
    Dereference(Box<Expr>),

    /// A block expression
    Block(Block),

    /// An if expression
    If(Box<IfExpr>),

    /// A match expression
    Match(Box<MatchExpr>),

    /// A loop expression
    Loop(Box<LoopExpr>),

    /// A while expression
    While(Box<WhileExpr>),

    /// A for loop expression
    For(Box<ForExpr>),

    /// An unsafe block
    Unsafe(Block),

    /// A signal expression (e.g., `signal(0)`)
    Signal(Box<Expr>),

    /// An effect expression (e.g., `effect(|| { ... })`)
    Effect(Box<Expr>),

    /// A memo expression (e.g., `memo(|| { ... })`)
    Memo(Box<Expr>),

    /// A resource expression (e.g., `resource(fetcher, loader)`)
    Resource(Box<ResourceExpr>),

    /// A parenthesized expression
    Paren(Box<Expr>),

    /// An expression that couldn't be parsed (error recovery)
    Error,
}

/// A binary operation expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BinaryExpr {
    /// The left-hand side.
    pub left: Box<Expr>,
    /// The operator.
    pub op: BinaryOp,
    /// The right-hand side.
    pub right: Box<Expr>,
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic operators
    Add,
    Sub,
    Mul,
    Div,
    Rem,

    // Comparison operators
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical operators
    And,
    Or,

    // Bitwise operators
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,

    // Assignment operators
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}

impl BinaryOp {
    /// Check if this is an assignment operator.
    pub fn is_assignment(&self) -> bool {
        matches!(
            self,
            BinaryOp::Assign
                | BinaryOp::AddAssign
                | BinaryOp::SubAssign
                | BinaryOp::MulAssign
                | BinaryOp::DivAssign
                | BinaryOp::RemAssign
                | BinaryOp::BitAndAssign
                | BinaryOp::BitOrAssign
                | BinaryOp::BitXorAssign
                | BinaryOp::ShlAssign
                | BinaryOp::ShrAssign
        )
    }

    /// Check if this is a comparison operator.
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
        )
    }
}

/// A unary operation expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnaryExpr {
    /// The operator.
    pub op: UnaryOp,
    /// The operand.
    pub expr: Box<Expr>,
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Negation (`-x`)
    Neg,
    /// Logical NOT (`!x`)
    Not,
    /// Bitwise NOT (`~x`)
    BitNot,
}

/// A function call expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallExpr {
    /// The function being called.
    pub func: Expr,
    /// The arguments.
    pub args: Vec<Expr>,
}

/// A method call expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MethodCallExpr {
    /// The receiver (object).
    pub receiver: Expr,
    /// The method name.
    pub method: Ident,
    /// Optional generic arguments.
    pub generic_args: Option<Vec<Ty>>,
    /// The arguments.
    pub args: Vec<Expr>,
}

/// A field access expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldExpr {
    /// The object.
    pub expr: Expr,
    /// The field name or tuple index.
    pub field: Field,
}

/// A field accessor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Field {
    /// A named field.
    Named(Ident),
    /// A tuple field index.
    Index(u32),
}

/// An index expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndexExpr {
    /// The array or slice.
    pub expr: Expr,
    /// The index.
    pub index: Expr,
}

/// A range expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RangeExpr {
    /// The start of the range (None for `..end`).
    pub start: Option<Expr>,
    /// The end of the range (None for `start..`).
    pub end: Option<Expr>,
    /// Whether this is an inclusive range (`..=`).
    pub inclusive: bool,
}

/// A closure expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClosureExpr {
    /// The closure parameters.
    pub params: Vec<ClosureParam>,
    /// The closure body.
    pub body: Expr,
}

/// A closure parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClosureParam {
    /// The pattern for this parameter.
    pub pat: Pat,
    /// Optional type annotation.
    pub ty: Option<Ty>,
}

/// An array literal expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArrayExpr {
    /// An array with listed elements (e.g., `[1, 2, 3]`)
    List(Vec<Expr>),
    /// An array with a repeated element (e.g., `[0; 10]`)
    Repeat { elem: Box<Expr>, len: Box<Expr> },
}

/// A struct literal expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructExpr {
    /// The path to the struct.
    pub path: Path,
    /// The field initializers.
    pub fields: Vec<FieldInit>,
    /// Optional struct update syntax (`..rest`).
    pub rest: Option<Box<Expr>>,
}

/// A field initializer in a struct literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldInit {
    /// The field name.
    pub ident: Ident,
    /// The value (None for shorthand `{ x }` instead of `{ x: x }`).
    pub expr: Option<Expr>,
}

/// A type cast expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CastExpr {
    /// The expression being cast.
    pub expr: Expr,
    /// The target type.
    pub ty: Ty,
}

/// A reference expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReferenceExpr {
    /// Whether this is a mutable reference.
    pub mutable: bool,
    /// The referenced expression.
    pub expr: Expr,
}

/// A block expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Block {
    /// The statements in the block.
    pub stmts: Vec<Stmt>,
    /// Optional final expression (the block's value).
    pub expr: Option<Box<Expr>>,
    /// The span of the block.
    pub span: Span,
}

/// An if expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IfExpr {
    /// The condition.
    pub cond: Expr,
    /// The then block.
    pub then_block: Block,
    /// Optional else block or else-if chain.
    pub else_block: Option<Box<Expr>>,
}

/// A match expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchExpr {
    /// The expression being matched.
    pub expr: Expr,
    /// The match arms.
    pub arms: Vec<MatchArm>,
}

/// A match arm.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchArm {
    /// The pattern.
    pub pat: Pat,
    /// Optional guard condition.
    pub guard: Option<Expr>,
    /// The arm body.
    pub body: Expr,
}

/// A loop expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoopExpr {
    /// The loop body.
    pub body: Block,
}

/// A while expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WhileExpr {
    /// The loop condition.
    pub cond: Expr,
    /// The loop body.
    pub body: Block,
}

/// A for loop expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ForExpr {
    /// The loop variable pattern.
    pub pat: Pat,
    /// The iterator expression.
    pub iter: Expr,
    /// The loop body.
    pub body: Block,
}

/// A resource expression (for async resource loading).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceExpr {
    /// The fetcher function.
    pub fetcher: Expr,
    /// The loader function.
    pub loader: Expr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_expr() {
        let lit = Lit::bool(true);
        let expr = Expr::lit(lit, Span::DUMMY);

        match expr.kind {
            ExprKind::Literal(Lit::Bool(_)) => {}
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_binary_expr() {
        let left = Expr::lit(Lit::decimal("1".to_string()), Span::DUMMY);
        let right = Expr::lit(Lit::decimal("2".to_string()), Span::DUMMY);
        let binary = Expr::new(
            ExprKind::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            }),
            Span::DUMMY,
        );

        match binary.kind {
            ExprKind::Binary(ref b) => {
                assert_eq!(b.op, BinaryOp::Add);
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_binary_op_checks() {
        assert!(BinaryOp::Assign.is_assignment());
        assert!(BinaryOp::AddAssign.is_assignment());
        assert!(!BinaryOp::Add.is_assignment());

        assert!(BinaryOp::Eq.is_comparison());
        assert!(BinaryOp::Lt.is_comparison());
        assert!(!BinaryOp::Add.is_comparison());
    }

    #[test]
    fn test_block_expr() {
        let block = Block {
            stmts: vec![],
            expr: None,
            span: Span::DUMMY,
        };
        let expr = Expr::block(block.clone(), Span::DUMMY);

        match expr.kind {
            ExprKind::Block(ref b) => {
                assert_eq!(b.stmts.len(), 0);
                assert!(b.expr.is_none());
            }
            _ => panic!("Expected block expression"),
        }
    }

    #[test]
    fn test_call_expr() {
        let func = Expr::path(Path::from_ident(Ident::dummy("foo".to_string())));
        let arg = Expr::lit(Lit::decimal("42".to_string()), Span::DUMMY);
        let call = Expr::new(
            ExprKind::Call(Box::new(CallExpr {
                func,
                args: vec![arg],
            })),
            Span::DUMMY,
        );

        match call.kind {
            ExprKind::Call(ref c) => {
                assert_eq!(c.args.len(), 1);
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_if_expr() {
        let cond = Expr::lit(Lit::bool(true), Span::DUMMY);
        let then_block = Block {
            stmts: vec![],
            expr: None,
            span: Span::DUMMY,
        };
        let if_expr = Expr::new(
            ExprKind::If(Box::new(IfExpr {
                cond,
                then_block,
                else_block: None,
            })),
            Span::DUMMY,
        );

        match if_expr.kind {
            ExprKind::If(ref i) => {
                assert!(i.else_block.is_none());
            }
            _ => panic!("Expected if expression"),
        }
    }

    #[test]
    fn test_array_expr() {
        let elem1 = Expr::lit(Lit::decimal("1".to_string()), Span::DUMMY);
        let elem2 = Expr::lit(Lit::decimal("2".to_string()), Span::DUMMY);
        let array = Expr::new(
            ExprKind::Array(ArrayExpr::List(vec![elem1, elem2])),
            Span::DUMMY,
        );

        match array.kind {
            ExprKind::Array(ArrayExpr::List(ref elems)) => {
                assert_eq!(elems.len(), 2);
            }
            _ => panic!("Expected array expression"),
        }
    }
}

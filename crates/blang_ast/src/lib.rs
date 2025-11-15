//! Abstract Syntax Tree (AST) for the Blang programming language.
//!
//! This crate defines the complete AST structure for Blang, including:
//!
//! - **Literals** ([`lit`]): Integer, float, boolean, character, string, and template literals
//! - **Types** ([`ty`]): All type expressions (primitives, paths, references, arrays, etc.)
//! - **Patterns** ([`pat`]): Pattern matching constructs
//! - **Expressions** ([`expr`]): All expression types (binary ops, function calls, blocks, etc.)
//! - **Statements** ([`stmt`]): Let bindings, expression statements, item statements
//! - **Items** ([`item`]): Top-level declarations (functions, structs, enums, components, scripts, etc.)
//! - **Attributes** ([`attr`]): Annotation attributes
//! - **Paths** ([`path`]): Identifiers and qualified paths
//!
//! # Example
//!
//! ```
//! use blang_ast::{Expr, ExprKind, Lit};
//! use blang_span::Span;
//!
//! // Create a literal expression: 42
//! let lit = Lit::decimal("42".to_string());
//! let expr = Expr::lit(lit, Span::DUMMY);
//!
//! match expr.kind {
//!     ExprKind::Literal(_) => println!("It's a literal!"),
//!     _ => {}
//! }
//! ```
//!
//! # AST Structure
//!
//! The AST is designed to be:
//!
//! - **Complete**: Covers all language constructs from the grammar
//! - **Serializable**: All types implement `Serialize` and `Deserialize` via serde
//! - **Span-aware**: All major nodes track their source location via [`blang_span::Span`]
//! - **Type-safe**: Uses Rust's type system to enforce correctness
//!
//! # Module Organization
//!
//! - [`lit`]: Literal types
//! - [`path`]: Paths and identifiers
//! - [`ty`]: Type expressions
//! - [`pat`]: Pattern matching
//! - [`expr`]: Expressions
//! - [`stmt`]: Statements
//! - [`item`]: Top-level items/declarations
//! - [`attr`]: Attributes

pub mod attr;
pub mod expr;
pub mod item;
pub mod lit;
pub mod pat;
pub mod path;
pub mod stmt;
pub mod ty;

// Re-export commonly used types
pub use attr::{Attr, AttrArg, AttrArgs, AttrPath};
pub use expr::{
    ArrayExpr, BinaryExpr, BinaryOp, Block, CallExpr, CastExpr, ClosureExpr, ClosureParam, Expr,
    ExprKind, Field, FieldExpr, FieldInit, ForExpr, IfExpr, IndexExpr, LoopExpr, MatchArm,
    MatchExpr, MethodCallExpr, RangeExpr, ReferenceExpr, ResourceExpr, StructExpr, UnaryExpr,
    UnaryOp, WhileExpr,
};
pub use item::{
    ActorDecl, ActorItem, AssociatedType, AssociatedTypeImpl, ComponentDecl, ComponentItem,
    ComputedDecl, ConfigOption, ConstDecl, EnumDecl, EnumVariant, EnumVariantBody, FunctionDecl,
    FunctionSig, ImplDecl, ImplItem, Item, ItemKind, JobDecl, LifecycleHook, ModuleDecl, Param,
    PropDecl, ReceiverDecl, ScheduleOption, ScriptDecl, ScriptItem, StateDecl, StaticDecl,
    StepDecl, StepOption, StepOptionValue, StructBody, StructDecl, StructField, TraitDecl,
    TraitItem, TupleField, TypeAliasDecl, UseDecl, UseTree, ViewContent,
};
pub use lit::{BoolLit, CharLit, FloatLit, IntBase, IntLit, Lit, StrLit, TemplateLit, TemplatePart};
pub use pat::{
    EnumPat, EnumVariantPat, FieldPat, IdentPat, Pat, PatKind, ReferencePat, StructPat, TuplePat,
    TupleStructPat,
};
pub use path::{GenericArgs, Ident, Path, PathSegment};
pub use stmt::{LetStmt, Stmt, StmtKind};
pub use ty::{
    ArrayLen, ArrayTy, FunctionTy, GenericParam, PointerTy, PrimitiveTy, ReferenceTy, TraitBound,
    Ty, TyKind,
};

// Re-export span types for convenience
pub use blang_span::{BytePos, Span};
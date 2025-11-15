//! Pattern types for the AST.

use crate::lit::Lit;
use crate::path::{Ident, Path};
use blang_span::Span;
use serde::{Deserialize, Serialize};

/// A pattern for destructuring and matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pat {
    /// The kind of pattern.
    pub kind: PatKind,
    /// The span of this pattern in the source.
    pub span: Span,
}

impl Pat {
    /// Create a new pattern with the given kind and span.
    pub fn new(kind: PatKind, span: Span) -> Self {
        Pat { kind, span }
    }

    /// Create a wildcard pattern.
    pub fn wildcard(span: Span) -> Self {
        Pat {
            kind: PatKind::Wildcard,
            span,
        }
    }

    /// Create an identifier pattern.
    pub fn ident(ident: Ident, mutable: bool) -> Self {
        let span = ident.span;
        Pat {
            kind: PatKind::Ident(IdentPat {
                ident,
                mutable,
                subpattern: None,
            }),
            span,
        }
    }
}

/// The kind of a pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatKind {
    /// A literal pattern (e.g., `42`, `true`, `"hello"`)
    Literal(Lit),

    /// An identifier pattern (e.g., `x`, `mut y`)
    Ident(IdentPat),

    /// The wildcard pattern `_`
    Wildcard,

    /// The rest pattern `..`
    Rest,

    /// A reference pattern (e.g., `&x`, `&mut y`)
    Reference(Box<ReferencePat>),

    /// A struct pattern (e.g., `Point { x, y }`)
    Struct(StructPat),

    /// A tuple pattern (e.g., `(x, y, z)`)
    Tuple(TuplePat),

    /// A tuple struct pattern (e.g., `Some(x)`)
    TupleStruct(TupleStructPat),

    /// An enum pattern (e.g., `Option::Some(x)`)
    Enum(EnumPat),

    /// An OR pattern (e.g., `x | y`)
    Or(Vec<Pat>),

    /// A pattern that couldn't be parsed (error recovery)
    Error,
}

/// An identifier pattern with optional mutability and subpattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdentPat {
    /// The identifier.
    pub ident: Ident,
    /// Whether this binding is mutable.
    pub mutable: bool,
    /// Optional subpattern (e.g., `x @ pattern`).
    pub subpattern: Option<Box<Pat>>,
}

/// A reference pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReferencePat {
    /// Whether this is a mutable reference.
    pub mutable: bool,
    /// The inner pattern.
    pub pat: Pat,
}

/// A struct pattern (e.g., `Point { x, y }`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructPat {
    /// The path to the struct.
    pub path: Path,
    /// The field patterns.
    pub fields: Vec<FieldPat>,
    /// Whether this pattern has a `..` rest pattern.
    pub rest: bool,
}

/// A field pattern in a struct pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldPat {
    /// The field name.
    pub ident: Ident,
    /// The pattern for this field (defaults to identifier pattern if None).
    pub pat: Option<Pat>,
}

/// A tuple pattern (e.g., `(x, y, z)`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TuplePat {
    /// The element patterns.
    pub elems: Vec<Pat>,
}

/// A tuple struct pattern (e.g., `Some(x)`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TupleStructPat {
    /// The path to the tuple struct.
    pub path: Path,
    /// The element patterns.
    pub elems: Vec<Pat>,
}

/// An enum pattern (e.g., `Option::Some(x)` or `Result::Ok { value }`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnumPat {
    /// The path to the enum variant.
    pub path: Path,
    /// The variant pattern.
    pub variant: EnumVariantPat,
}

/// The pattern for an enum variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnumVariantPat {
    /// A unit variant (e.g., `None`)
    Unit,
    /// A tuple variant (e.g., `Some(x)`)
    Tuple(Vec<Pat>),
    /// A struct variant (e.g., `Point { x, y }`)
    Struct(Vec<FieldPat>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_pat() {
        let pat = Pat::wildcard(Span::DUMMY);
        assert_eq!(pat.kind, PatKind::Wildcard);
    }

    #[test]
    fn test_ident_pat() {
        let ident = Ident::dummy("x".to_string());
        let pat = Pat::ident(ident.clone(), false);

        match pat.kind {
            PatKind::Ident(ref i) => {
                assert_eq!(i.ident.name, "x");
                assert!(!i.mutable);
                assert!(i.subpattern.is_none());
            }
            _ => panic!("Expected Ident pattern"),
        }
    }

    #[test]
    fn test_mut_ident_pat() {
        let ident = Ident::dummy("y".to_string());
        let pat = Pat::ident(ident, true);

        match pat.kind {
            PatKind::Ident(ref i) => {
                assert_eq!(i.ident.name, "y");
                assert!(i.mutable);
            }
            _ => panic!("Expected Ident pattern"),
        }
    }

    #[test]
    fn test_tuple_pat() {
        let pat1 = Pat::wildcard(Span::DUMMY);
        let pat2 = Pat::ident(Ident::dummy("x".to_string()), false);
        let tuple_pat = Pat::new(
            PatKind::Tuple(TuplePat {
                elems: vec![pat1, pat2],
            }),
            Span::DUMMY,
        );

        match tuple_pat.kind {
            PatKind::Tuple(ref t) => {
                assert_eq!(t.elems.len(), 2);
            }
            _ => panic!("Expected Tuple pattern"),
        }
    }

    #[test]
    fn test_or_pat() {
        let pat1 = Pat::ident(Ident::dummy("x".to_string()), false);
        let pat2 = Pat::ident(Ident::dummy("y".to_string()), false);
        let or_pat = Pat::new(PatKind::Or(vec![pat1, pat2]), Span::DUMMY);

        match or_pat.kind {
            PatKind::Or(ref pats) => {
                assert_eq!(pats.len(), 2);
            }
            _ => panic!("Expected Or pattern"),
        }
    }

    #[test]
    fn test_struct_pat() {
        let path = Path::from_ident(Ident::dummy("Point".to_string()));
        let field1 = FieldPat {
            ident: Ident::dummy("x".to_string()),
            pat: None,
        };
        let field2 = FieldPat {
            ident: Ident::dummy("y".to_string()),
            pat: None,
        };

        let struct_pat = Pat::new(
            PatKind::Struct(StructPat {
                path,
                fields: vec![field1, field2],
                rest: false,
            }),
            Span::DUMMY,
        );

        match struct_pat.kind {
            PatKind::Struct(ref s) => {
                assert_eq!(s.fields.len(), 2);
                assert!(!s.rest);
            }
            _ => panic!("Expected Struct pattern"),
        }
    }
}

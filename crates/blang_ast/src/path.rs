//! Path types for the AST.

use blang_span::Span;
use serde::{Deserialize, Serialize};

/// An identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ident {
    /// The name of the identifier.
    pub name: String,
    /// The span of the identifier in the source.
    pub span: Span,
}

impl Ident {
    /// Create a new identifier.
    pub fn new(name: String, span: Span) -> Self {
        Ident { name, span }
    }

    /// Create an identifier with a dummy span.
    pub fn dummy(name: String) -> Self {
        Ident {
            name,
            span: Span::DUMMY,
        }
    }
}

/// A path like `foo::bar::Baz` or `std::collections::HashMap`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Path {
    /// The segments of the path.
    pub segments: Vec<PathSegment>,
    /// The span of the entire path.
    pub span: Span,
}

impl Path {
    /// Create a new path.
    pub fn new(segments: Vec<PathSegment>, span: Span) -> Self {
        Path { segments, span }
    }

    /// Create a path from a single identifier.
    pub fn from_ident(ident: Ident) -> Self {
        let span = ident.span;
        Path {
            segments: vec![PathSegment {
                ident,
                generic_args: None,
            }],
            span,
        }
    }

    /// Check if this path is a single identifier.
    pub fn is_simple(&self) -> bool {
        self.segments.len() == 1 && self.segments[0].generic_args.is_none()
    }

    /// Get the final segment of the path.
    pub fn last_segment(&self) -> Option<&PathSegment> {
        self.segments.last()
    }
}

/// A segment of a path with optional generic arguments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathSegment {
    /// The identifier for this segment.
    pub ident: Ident,
    /// Optional generic arguments (e.g., `<T, U>`).
    pub generic_args: Option<GenericArgs>,
}

/// Generic arguments in a path segment (e.g., `<T, U>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericArgs {
    /// The type arguments.
    pub args: Vec<super::ty::Ty>,
    /// The span of the generic arguments.
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident() {
        let ident = Ident::new("foo".to_string(), Span::DUMMY);
        assert_eq!(ident.name, "foo");
        assert_eq!(ident.span, Span::DUMMY);

        let ident2 = Ident::dummy("bar".to_string());
        assert_eq!(ident2.name, "bar");
    }

    #[test]
    fn test_path_from_ident() {
        let ident = Ident::dummy("foo".to_string());
        let path = Path::from_ident(ident.clone());
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.name, "foo");
        assert!(path.is_simple());
    }

    #[test]
    fn test_path_segments() {
        let path = Path::new(
            vec![
                PathSegment {
                    ident: Ident::dummy("std".to_string()),
                    generic_args: None,
                },
                PathSegment {
                    ident: Ident::dummy("collections".to_string()),
                    generic_args: None,
                },
                PathSegment {
                    ident: Ident::dummy("HashMap".to_string()),
                    generic_args: None,
                },
            ],
            Span::DUMMY,
        );

        assert_eq!(path.segments.len(), 3);
        assert!(!path.is_simple());
        assert_eq!(path.last_segment().unwrap().ident.name, "HashMap");
    }
}

//! Attribute types for the AST.

use crate::lit::Lit;
use crate::path::Ident;
use blang_span::Span;
use serde::{Deserialize, Serialize};

/// An attribute (e.g., `#[inline]`, `#[derive(Debug, Clone)]`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attr {
    /// The attribute path (e.g., `inline`, `derive`, `cfg`).
    pub path: AttrPath,
    /// Optional attribute arguments.
    pub args: Option<AttrArgs>,
    /// The span of this attribute.
    pub span: Span,
}

impl Attr {
    /// Create a new attribute.
    pub fn new(path: AttrPath, args: Option<AttrArgs>, span: Span) -> Self {
        Attr { path, args, span }
    }

    /// Create a simple attribute with no arguments (e.g., `#[inline]`).
    pub fn simple(name: String, span: Span) -> Self {
        Attr {
            path: AttrPath::Simple(Ident::new(name, span)),
            args: None,
            span,
        }
    }
}

/// An attribute path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttrPath {
    /// A simple identifier (e.g., `inline`)
    Simple(Ident),

    /// A path (e.g., `foo::bar`)
    Path(Vec<Ident>),
}

/// Attribute arguments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttrArgs {
    /// List of arguments (e.g., `derive(Debug, Clone)`)
    List(Vec<AttrArg>),

    /// Key-value arguments (e.g., `deprecated(since = "1.0")`)
    KeyValue(Vec<(Ident, Lit)>),
}

/// An attribute argument.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttrArg {
    /// An identifier argument (e.g., `Debug` in `#[derive(Debug)]`)
    Ident(Ident),

    /// A key-value argument (e.g., `since = "1.0"`)
    KeyValue(Ident, Lit),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_attr() {
        let attr = Attr::simple("inline".to_string(), Span::DUMMY);

        match attr.path {
            AttrPath::Simple(ref i) => {
                assert_eq!(i.name, "inline");
            }
            _ => panic!("Expected simple attribute path"),
        }
        assert!(attr.args.is_none());
    }

    #[test]
    fn test_attr_with_list_args() {
        let args = AttrArgs::List(vec![
            AttrArg::Ident(Ident::dummy("Debug".to_string())),
            AttrArg::Ident(Ident::dummy("Clone".to_string())),
        ]);

        let attr = Attr::new(
            AttrPath::Simple(Ident::dummy("derive".to_string())),
            Some(args),
            Span::DUMMY,
        );

        match attr.args {
            Some(AttrArgs::List(ref list)) => {
                assert_eq!(list.len(), 2);
            }
            _ => panic!("Expected list arguments"),
        }
    }

    #[test]
    fn test_attr_with_key_value() {
        use crate::lit::{BoolLit, Lit};

        let args = AttrArgs::KeyValue(vec![(
            Ident::dummy("feature".to_string()),
            Lit::Bool(BoolLit::True),
        )]);

        let attr = Attr::new(
            AttrPath::Simple(Ident::dummy("cfg".to_string())),
            Some(args),
            Span::DUMMY,
        );

        match attr.args {
            Some(AttrArgs::KeyValue(ref kv)) => {
                assert_eq!(kv.len(), 1);
                assert_eq!(kv[0].0.name, "feature");
            }
            _ => panic!("Expected key-value arguments"),
        }
    }

    #[test]
    fn test_attr_path() {
        let path = AttrPath::Path(vec![
            Ident::dummy("std".to_string()),
            Ident::dummy("cfg".to_string()),
        ]);

        match path {
            AttrPath::Path(ref segments) => {
                assert_eq!(segments.len(), 2);
                assert_eq!(segments[0].name, "std");
                assert_eq!(segments[1].name, "cfg");
            }
            _ => panic!("Expected path"),
        }
    }
}

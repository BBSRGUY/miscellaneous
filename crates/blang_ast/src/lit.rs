//! Literal types for the AST.

use serde::{Deserialize, Serialize};

/// An integer literal with its base.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IntLit {
    /// The raw string representation (e.g., "0xFF", "42", "0b1010")
    pub raw: String,
    /// The numeric base (decimal, hex, octal, binary)
    pub base: IntBase,
    /// Optional type suffix (e.g., "i32", "u64")
    pub suffix: Option<String>,
}

/// The base of an integer literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntBase {
    /// Decimal (base 10)
    Decimal,
    /// Hexadecimal (base 16, prefix 0x)
    Hexadecimal,
    /// Octal (base 8, prefix 0o)
    Octal,
    /// Binary (base 2, prefix 0b)
    Binary,
}

/// A floating-point literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FloatLit {
    /// The raw string representation (e.g., "3.14", "1e-10")
    pub raw: String,
    /// Optional type suffix (e.g., "f32", "f64")
    pub suffix: Option<String>,
}

/// A boolean literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoolLit {
    /// The `true` literal
    True,
    /// The `false` literal
    False,
}

impl BoolLit {
    /// Get the boolean value.
    pub fn value(&self) -> bool {
        match self {
            BoolLit::True => true,
            BoolLit::False => false,
        }
    }
}

/// A character literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharLit {
    /// The character value (after escape sequence processing)
    pub value: char,
    /// The raw string representation including quotes
    pub raw: String,
}

/// A string literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrLit {
    /// The string value (after escape sequence processing)
    pub value: String,
    /// The raw string representation including quotes
    pub raw: String,
    /// Whether this is a raw string literal
    pub is_raw: bool,
}

/// A template string literal with interpolation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateLit {
    /// The template parts and interpolations
    pub parts: Vec<TemplatePart>,
}

/// A part of a template string literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplatePart {
    /// A literal string segment
    String(String),
    /// An interpolated expression (represented as a string for now)
    Interpolation(String),
}

/// A literal value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lit {
    /// Integer literal
    Int(IntLit),
    /// Floating-point literal
    Float(FloatLit),
    /// Boolean literal
    Bool(BoolLit),
    /// Character literal
    Char(CharLit),
    /// String literal
    Str(StrLit),
    /// Template string literal
    Template(TemplateLit),
}

impl Lit {
    /// Create a new integer literal.
    pub fn int(raw: String, base: IntBase, suffix: Option<String>) -> Self {
        Lit::Int(IntLit { raw, base, suffix })
    }

    /// Create a new decimal integer literal.
    pub fn decimal(raw: String) -> Self {
        Lit::Int(IntLit {
            raw,
            base: IntBase::Decimal,
            suffix: None,
        })
    }

    /// Create a new floating-point literal.
    pub fn float(raw: String, suffix: Option<String>) -> Self {
        Lit::Float(FloatLit { raw, suffix })
    }

    /// Create a new boolean literal.
    pub fn bool(value: bool) -> Self {
        Lit::Bool(if value {
            BoolLit::True
        } else {
            BoolLit::False
        })
    }

    /// Create a new character literal.
    pub fn char(value: char, raw: String) -> Self {
        Lit::Char(CharLit { value, raw })
    }

    /// Create a new string literal.
    pub fn string(value: String, raw: String, is_raw: bool) -> Self {
        Lit::Str(StrLit {
            value,
            raw,
            is_raw,
        })
    }

    /// Create a new template literal.
    pub fn template(parts: Vec<TemplatePart>) -> Self {
        Lit::Template(TemplateLit { parts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_lit() {
        let lit = Lit::decimal("42".to_string());
        match lit {
            Lit::Int(IntLit { raw, base, suffix }) => {
                assert_eq!(raw, "42");
                assert_eq!(base, IntBase::Decimal);
                assert_eq!(suffix, None);
            }
            _ => panic!("Expected Int literal"),
        }
    }

    #[test]
    fn test_hex_lit() {
        let lit = Lit::int("FF".to_string(), IntBase::Hexadecimal, Some("u32".to_string()));
        match lit {
            Lit::Int(IntLit { raw, base, suffix }) => {
                assert_eq!(raw, "FF");
                assert_eq!(base, IntBase::Hexadecimal);
                assert_eq!(suffix, Some("u32".to_string()));
            }
            _ => panic!("Expected Int literal"),
        }
    }

    #[test]
    fn test_bool_lit() {
        let lit = Lit::bool(true);
        match lit {
            Lit::Bool(BoolLit::True) => {}
            _ => panic!("Expected True literal"),
        }

        assert!(BoolLit::True.value());
        assert!(!BoolLit::False.value());
    }

    #[test]
    fn test_string_lit() {
        let lit = Lit::string("hello".to_string(), "\"hello\"".to_string(), false);
        match lit {
            Lit::Str(StrLit { value, raw, is_raw }) => {
                assert_eq!(value, "hello");
                assert_eq!(raw, "\"hello\"");
                assert!(!is_raw);
            }
            _ => panic!("Expected Str literal"),
        }
    }

    #[test]
    fn test_template_lit() {
        let parts = vec![
            TemplatePart::String("Hello ".to_string()),
            TemplatePart::Interpolation("name".to_string()),
            TemplatePart::String("!".to_string()),
        ];
        let lit = Lit::template(parts.clone());
        match lit {
            Lit::Template(TemplateLit { parts: p }) => {
                assert_eq!(p, parts);
            }
            _ => panic!("Expected Template literal"),
        }
    }
}

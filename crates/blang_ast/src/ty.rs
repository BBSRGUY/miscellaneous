//! Type expressions for the AST.

use crate::path::{Ident, Path};
use blang_span::Span;
use serde::{Deserialize, Serialize};

/// A type expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ty {
    /// The kind of type.
    pub kind: TyKind,
    /// The span of this type in the source.
    pub span: Span,
}

impl Ty {
    /// Create a new type with the given kind and span.
    pub fn new(kind: TyKind, span: Span) -> Self {
        Ty { kind, span }
    }
}

/// The kind of a type expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TyKind {
    /// A primitive type (i32, bool, etc.)
    Primitive(PrimitiveTy),
    /// A path type (e.g., `Option<T>`, `std::Vec<i32>`)
    Path(Path),
    /// A reference type (e.g., `&T`, `&mut T`)
    Reference(Box<ReferenceTy>),
    /// A raw pointer type (e.g., `*const T`, `*mut T`)
    Pointer(Box<PointerTy>),
    /// An array type (e.g., `[T; 10]`)
    Array(Box<ArrayTy>),
    /// A slice type (e.g., `[T]`)
    Slice(Box<Ty>),
    /// A tuple type (e.g., `(i32, bool)`)
    Tuple(Vec<Ty>),
    /// A function type (e.g., `fn(i32, i32) -> i32`)
    Function(Box<FunctionTy>),
    /// The never type `!`
    Never,
    /// A type that couldn't be parsed (error recovery)
    Error,
}

/// A primitive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveTy {
    // Signed integers
    I8,
    I16,
    I32,
    I64,

    // Unsigned integers
    U8,
    U16,
    U32,
    U64,

    // Floating point
    F32,
    F64,

    // Other primitives
    Bool,
    Char,
    Str,

    /// The unit type `()`
    Unit,
}

impl PrimitiveTy {
    /// Parse a primitive type from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "i8" => Some(PrimitiveTy::I8),
            "i16" => Some(PrimitiveTy::I16),
            "i32" => Some(PrimitiveTy::I32),
            "i64" => Some(PrimitiveTy::I64),
            "u8" => Some(PrimitiveTy::U8),
            "u16" => Some(PrimitiveTy::U16),
            "u32" => Some(PrimitiveTy::U32),
            "u64" => Some(PrimitiveTy::U64),
            "f32" => Some(PrimitiveTy::F32),
            "f64" => Some(PrimitiveTy::F64),
            "bool" => Some(PrimitiveTy::Bool),
            "char" => Some(PrimitiveTy::Char),
            "str" => Some(PrimitiveTy::Str),
            _ => None,
        }
    }

    /// Check if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            PrimitiveTy::I8
                | PrimitiveTy::I16
                | PrimitiveTy::I32
                | PrimitiveTy::I64
                | PrimitiveTy::U8
                | PrimitiveTy::U16
                | PrimitiveTy::U32
                | PrimitiveTy::U64
        )
    }

    /// Check if this is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, PrimitiveTy::F32 | PrimitiveTy::F64)
    }

    /// Check if this is a signed integer type.
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            PrimitiveTy::I8 | PrimitiveTy::I16 | PrimitiveTy::I32 | PrimitiveTy::I64
        )
    }

    /// Check if this is an unsigned integer type.
    pub fn is_unsigned(&self) -> bool {
        matches!(
            self,
            PrimitiveTy::U8 | PrimitiveTy::U16 | PrimitiveTy::U32 | PrimitiveTy::U64
        )
    }
}

/// A reference type (`&T` or `&mut T`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReferenceTy {
    /// Whether this is a mutable reference.
    pub mutable: bool,
    /// The referenced type.
    pub ty: Ty,
}

/// A raw pointer type (`*const T` or `*mut T`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PointerTy {
    /// Whether this is a mutable pointer.
    pub mutable: bool,
    /// The pointed-to type.
    pub ty: Ty,
}

/// An array type (`[T; N]`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArrayTy {
    /// The element type.
    pub elem_ty: Ty,
    /// The array length (represented as an expression, but usually a literal).
    /// For simplicity, we store it as a u64 for now.
    pub len: ArrayLen,
}

/// The length of an array type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArrayLen {
    /// A literal length value.
    Literal(u64),
    /// A named constant or expression.
    Const(Path),
}

/// A function type signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionTy {
    /// The parameter types.
    pub params: Vec<Ty>,
    /// The return type (None for unit return).
    pub return_ty: Option<Ty>,
}

/// Generic parameter declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericParam {
    /// The name of the generic parameter.
    pub name: Ident,
    /// Optional trait bounds.
    pub bounds: Vec<TraitBound>,
    /// The span of this parameter.
    pub span: Span,
}

/// A trait bound (e.g., `T: Display + Clone`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitBound {
    /// The trait path.
    pub trait_path: Path,
    /// The span of this bound.
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_ty_from_str() {
        assert_eq!(PrimitiveTy::from_str("i32"), Some(PrimitiveTy::I32));
        assert_eq!(PrimitiveTy::from_str("bool"), Some(PrimitiveTy::Bool));
        assert_eq!(PrimitiveTy::from_str("f64"), Some(PrimitiveTy::F64));
        assert_eq!(PrimitiveTy::from_str("invalid"), None);
    }

    #[test]
    fn test_primitive_ty_checks() {
        assert!(PrimitiveTy::I32.is_integer());
        assert!(PrimitiveTy::I32.is_signed());
        assert!(!PrimitiveTy::I32.is_unsigned());
        assert!(!PrimitiveTy::I32.is_float());

        assert!(PrimitiveTy::U64.is_integer());
        assert!(!PrimitiveTy::U64.is_signed());
        assert!(PrimitiveTy::U64.is_unsigned());

        assert!(PrimitiveTy::F32.is_float());
        assert!(!PrimitiveTy::F32.is_integer());

        assert!(!PrimitiveTy::Bool.is_integer());
        assert!(!PrimitiveTy::Bool.is_float());
    }

    #[test]
    fn test_ty_creation() {
        let ty = Ty::new(TyKind::Primitive(PrimitiveTy::I32), Span::DUMMY);
        assert_eq!(ty.kind, TyKind::Primitive(PrimitiveTy::I32));
        assert_eq!(ty.span, Span::DUMMY);
    }

    #[test]
    fn test_reference_ty() {
        let inner = Ty::new(TyKind::Primitive(PrimitiveTy::I32), Span::DUMMY);
        let ref_ty = Ty::new(
            TyKind::Reference(Box::new(ReferenceTy {
                mutable: false,
                ty: inner,
            })),
            Span::DUMMY,
        );

        match ref_ty.kind {
            TyKind::Reference(ref r) => {
                assert!(!r.mutable);
                match r.ty.kind {
                    TyKind::Primitive(PrimitiveTy::I32) => {}
                    _ => panic!("Expected i32"),
                }
            }
            _ => panic!("Expected Reference"),
        }
    }

    #[test]
    fn test_array_ty() {
        let elem_ty = Ty::new(TyKind::Primitive(PrimitiveTy::I32), Span::DUMMY);
        let array_ty = Ty::new(
            TyKind::Array(Box::new(ArrayTy {
                elem_ty,
                len: ArrayLen::Literal(10),
            })),
            Span::DUMMY,
        );

        match array_ty.kind {
            TyKind::Array(ref a) => {
                assert_eq!(a.len, ArrayLen::Literal(10));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_function_ty() {
        let param1 = Ty::new(TyKind::Primitive(PrimitiveTy::I32), Span::DUMMY);
        let param2 = Ty::new(TyKind::Primitive(PrimitiveTy::Bool), Span::DUMMY);
        let return_ty = Ty::new(TyKind::Primitive(PrimitiveTy::U64), Span::DUMMY);

        let fn_ty = Ty::new(
            TyKind::Function(Box::new(FunctionTy {
                params: vec![param1, param2],
                return_ty: Some(return_ty),
            })),
            Span::DUMMY,
        );

        match fn_ty.kind {
            TyKind::Function(ref f) => {
                assert_eq!(f.params.len(), 2);
                assert!(f.return_ty.is_some());
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_tuple_ty() {
        let ty1 = Ty::new(TyKind::Primitive(PrimitiveTy::I32), Span::DUMMY);
        let ty2 = Ty::new(TyKind::Primitive(PrimitiveTy::Bool), Span::DUMMY);
        let tuple_ty = Ty::new(TyKind::Tuple(vec![ty1, ty2]), Span::DUMMY);

        match tuple_ty.kind {
            TyKind::Tuple(ref types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected Tuple"),
        }
    }
}

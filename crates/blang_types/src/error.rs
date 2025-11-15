//! Type errors for the Blang type system.

use crate::types::Type;
use blang_span::Span;
use std::fmt;
use thiserror::Error;

/// A type error with location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    /// The kind of type error.
    pub kind: TypeErrorKind,
    /// The span where the error occurred.
    pub span: Span,
}

/// The kind of type error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypeErrorKind {
    /// Type mismatch: expected one type, found another
    #[error("type mismatch: expected `{expected}`, found `{found}`")]
    Mismatch { expected: Type, found: Type },

    /// Undefined variable
    #[error("undefined variable `{name}`")]
    UndefinedVariable { name: String },

    /// Undefined function
    #[error("undefined function `{name}`")]
    UndefinedFunction { name: String },

    /// Undefined type
    #[error("undefined type `{name}`")]
    UndefinedType { name: String },

    /// Wrong number of arguments
    #[error("wrong number of arguments: expected {expected}, found {found}")]
    WrongArgumentCount { expected: usize, found: usize },

    /// Invalid operation for type
    #[error("invalid operation `{op}` for type `{ty}`")]
    InvalidOperation { op: String, ty: Type },

    /// Cannot apply binary operator
    #[error("cannot apply binary operator `{op}` to types `{left}` and `{right}`")]
    InvalidBinaryOp {
        op: String,
        left: Type,
        right: Type,
    },

    /// Cannot apply unary operator
    #[error("cannot apply unary operator `{op}` to type `{ty}`")]
    InvalidUnaryOp { op: String, ty: Type },

    /// Cannot dereference non-pointer type
    #[error("cannot dereference type `{ty}`")]
    CannotDereference { ty: Type },

    /// Cannot index non-array/slice type
    #[error("cannot index type `{ty}`")]
    CannotIndex { ty: Type },

    /// Cannot call non-function type
    #[error("cannot call type `{ty}`")]
    CannotCall { ty: Type },

    /// Cannot access field on non-struct type
    #[error("cannot access field `{field}` on type `{ty}`")]
    CannotAccessField { ty: Type, field: String },

    /// Field not found
    #[error("field `{field}` not found in type `{ty}`")]
    FieldNotFound { ty: Type, field: String },

    /// Cannot assign to immutable variable
    #[error("cannot assign to immutable variable `{name}`")]
    CannotAssignImmutable { name: String },

    /// Return type mismatch
    #[error("return type mismatch: expected `{expected}`, found `{found}`")]
    ReturnTypeMismatch { expected: Type, found: Type },

    /// Missing return value
    #[error("missing return value in function with return type `{expected}`")]
    MissingReturn { expected: Type },

    /// Condition must be boolean
    #[error("condition must be a boolean, found `{found}`")]
    ConditionNotBool { found: Type },

    /// Break outside loop
    #[error("break statement outside of loop")]
    BreakOutsideLoop,

    /// Continue outside loop
    #[error("continue statement outside of loop")]
    ContinueOutsideLoop,

    /// Type annotation required
    #[error("type annotation required for `{name}`")]
    TypeAnnotationRequired { name: String },

    /// Recursive type without indirection
    #[error("recursive type `{name}` has infinite size")]
    RecursiveType { name: String },

    /// Generic error message
    #[error("{message}")]
    Generic { message: String },
}

impl TypeError {
    /// Create a new type error.
    pub fn new(kind: TypeErrorKind, span: Span) -> Self {
        TypeError { kind, span }
    }

    /// Create a type mismatch error.
    pub fn mismatch(expected: Type, found: Type, span: Span) -> Self {
        TypeError::new(TypeErrorKind::Mismatch { expected, found }, span)
    }

    /// Create an undefined variable error.
    pub fn undefined_variable(name: String, span: Span) -> Self {
        TypeError::new(TypeErrorKind::UndefinedVariable { name }, span)
    }

    /// Create an undefined function error.
    pub fn undefined_function(name: String, span: Span) -> Self {
        TypeError::new(TypeErrorKind::UndefinedFunction { name }, span)
    }

    /// Create a wrong argument count error.
    pub fn wrong_arg_count(expected: usize, found: usize, span: Span) -> Self {
        TypeError::new(TypeErrorKind::WrongArgumentCount { expected, found }, span)
    }

    /// Create an invalid binary operator error.
    pub fn invalid_binary_op(op: String, left: Type, right: Type, span: Span) -> Self {
        TypeError::new(TypeErrorKind::InvalidBinaryOp { op, left, right }, span)
    }

    /// Create an invalid unary operator error.
    pub fn invalid_unary_op(op: String, ty: Type, span: Span) -> Self {
        TypeError::new(TypeErrorKind::InvalidUnaryOp { op, ty }, span)
    }

    /// Create a cannot call error.
    pub fn cannot_call(ty: Type, span: Span) -> Self {
        TypeError::new(TypeErrorKind::CannotCall { ty }, span)
    }

    /// Create a condition not bool error.
    pub fn condition_not_bool(found: Type, span: Span) -> Self {
        TypeError::new(TypeErrorKind::ConditionNotBool { found }, span)
    }

    /// Create a generic error.
    pub fn generic(message: String, span: Span) -> Self {
        TypeError::new(TypeErrorKind::Generic { message }, span)
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {:?}", self.kind, self.span)
    }
}

/// Result type for type checking operations.
pub type TypeResult<T> = Result<T, TypeError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrimitiveType;

    #[test]
    fn test_type_error_display() {
        let err = TypeError::mismatch(
            Type::primitive(PrimitiveType::I32),
            Type::primitive(PrimitiveType::Bool),
            Span::DUMMY,
        );
        assert!(err.to_string().contains("type mismatch"));
    }

    #[test]
    fn test_undefined_variable() {
        let err = TypeError::undefined_variable("x".to_string(), Span::DUMMY);
        assert!(err.to_string().contains("undefined variable"));
    }

    #[test]
    fn test_wrong_arg_count() {
        let err = TypeError::wrong_arg_count(2, 3, Span::DUMMY);
        assert!(err.to_string().contains("wrong number of arguments"));
    }
}

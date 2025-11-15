//! Type representation for the Blang type system.

use std::fmt;

/// A type in the Blang type system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Primitive types (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, char, str, unit)
    Primitive(PrimitiveType),

    /// Never type (!)
    Never,

    /// Function type: fn(T1, T2) -> R
    Function(Box<FunctionType>),

    /// Tuple type: (T1, T2, ...)
    Tuple(Vec<Type>),

    /// Array type: [T; N]
    Array(Box<Type>, usize),

    /// Slice type: [T]
    Slice(Box<Type>),

    /// Reference type: &T or &mut T
    Reference(Box<Type>, Mutability),

    /// Pointer type: *const T or *mut T
    Pointer(Box<Type>, Mutability),

    /// Named type (struct, enum, trait, type alias)
    Named(String),

    /// Generic type parameter
    Generic(String),

    /// Signal type: Signal<T>
    Signal(Box<Type>),

    /// Component props type
    Props(Vec<PropField>),

    /// Component state type
    State(Vec<StateField>),

    /// Type variable (for inference)
    Var(TypeVar),

    /// Unknown/error type (for error recovery)
    Unknown,
}

/// Primitive types in Blang.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    Str,
    Unit,
}

/// Mutability modifier for references and pointers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// Function type representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    /// Parameter types
    pub params: Vec<Type>,
    /// Return type (None means unit)
    pub return_type: Option<Box<Type>>,
}

/// Component prop field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropField {
    pub name: String,
    pub ty: Type,
}

/// Component state field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateField {
    pub name: String,
    pub ty: Type,
}

/// Type variable for type inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

impl Type {
    /// Create a primitive type.
    pub fn primitive(prim: PrimitiveType) -> Self {
        Type::Primitive(prim)
    }

    /// Create a function type.
    pub fn function(params: Vec<Type>, return_type: Option<Type>) -> Self {
        Type::Function(Box::new(FunctionType {
            params,
            return_type: return_type.map(Box::new),
        }))
    }

    /// Create a reference type.
    pub fn reference(ty: Type, mutable: bool) -> Self {
        Type::Reference(
            Box::new(ty),
            if mutable {
                Mutability::Mutable
            } else {
                Mutability::Immutable
            },
        )
    }

    /// Create a signal type.
    pub fn signal(ty: Type) -> Self {
        Type::Signal(Box::new(ty))
    }

    /// Check if this is a numeric type.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Primitive(
                PrimitiveType::I8
                    | PrimitiveType::I16
                    | PrimitiveType::I32
                    | PrimitiveType::I64
                    | PrimitiveType::U8
                    | PrimitiveType::U16
                    | PrimitiveType::U32
                    | PrimitiveType::U64
                    | PrimitiveType::F32
                    | PrimitiveType::F64
            )
        )
    }

    /// Check if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Primitive(
                PrimitiveType::I8
                    | PrimitiveType::I16
                    | PrimitiveType::I32
                    | PrimitiveType::I64
                    | PrimitiveType::U8
                    | PrimitiveType::U16
                    | PrimitiveType::U32
                    | PrimitiveType::U64
            )
        )
    }

    /// Check if this is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(
            self,
            Type::Primitive(PrimitiveType::F32 | PrimitiveType::F64)
        )
    }

    /// Check if this is a boolean type.
    pub fn is_bool(&self) -> bool {
        matches!(self, Type::Primitive(PrimitiveType::Bool))
    }

    /// Check if this is the unit type.
    pub fn is_unit(&self) -> bool {
        matches!(self, Type::Primitive(PrimitiveType::Unit))
    }

    /// Check if this is the never type.
    pub fn is_never(&self) -> bool {
        matches!(self, Type::Never)
    }

    /// Check if this type can be used in a boolean context.
    pub fn is_bool_compatible(&self) -> bool {
        self.is_bool()
    }

    /// Get the inner type of a reference.
    pub fn deref(&self) -> Option<&Type> {
        match self {
            Type::Reference(ty, _) => Some(ty),
            Type::Pointer(ty, _) => Some(ty),
            _ => None,
        }
    }

    /// Check if two types are compatible for assignment.
    pub fn is_assignable_to(&self, other: &Type) -> bool {
        self == other || matches!(other, Type::Unknown) || matches!(self, Type::Unknown)
    }
}

impl PrimitiveType {
    /// Get the default value for this primitive type.
    pub fn default_value_name(&self) -> &'static str {
        match self {
            PrimitiveType::I8
            | PrimitiveType::I16
            | PrimitiveType::I32
            | PrimitiveType::I64
            | PrimitiveType::U8
            | PrimitiveType::U16
            | PrimitiveType::U32
            | PrimitiveType::U64 => "0",
            PrimitiveType::F32 | PrimitiveType::F64 => "0.0",
            PrimitiveType::Bool => "false",
            PrimitiveType::Char => "'\\0'",
            PrimitiveType::Str => "\"\"",
            PrimitiveType::Unit => "()",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(prim) => write!(f, "{}", prim),
            Type::Never => write!(f, "!"),
            Type::Function(func) => {
                write!(f, "fn(")?;
                for (i, param) in func.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")?;
                if let Some(ret) = &func.return_type {
                    write!(f, " -> {}", ret)?;
                }
                Ok(())
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::Array(ty, size) => write!(f, "[{}; {}]", ty, size),
            Type::Slice(ty) => write!(f, "[{}]", ty),
            Type::Reference(ty, Mutability::Immutable) => write!(f, "&{}", ty),
            Type::Reference(ty, Mutability::Mutable) => write!(f, "&mut {}", ty),
            Type::Pointer(ty, Mutability::Immutable) => write!(f, "*const {}", ty),
            Type::Pointer(ty, Mutability::Mutable) => write!(f, "*mut {}", ty),
            Type::Named(name) => write!(f, "{}", name),
            Type::Generic(name) => write!(f, "{}", name),
            Type::Signal(ty) => write!(f, "Signal<{}>", ty),
            Type::Props(_) => write!(f, "Props"),
            Type::State(_) => write!(f, "State"),
            Type::Var(var) => write!(f, "?{}", var.0),
            Type::Unknown => write!(f, "<unknown>"),
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::I8 => write!(f, "i8"),
            PrimitiveType::I16 => write!(f, "i16"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::I64 => write!(f, "i64"),
            PrimitiveType::U8 => write!(f, "u8"),
            PrimitiveType::U16 => write!(f, "u16"),
            PrimitiveType::U32 => write!(f, "u32"),
            PrimitiveType::U64 => write!(f, "u64"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Char => write!(f, "char"),
            PrimitiveType::Str => write!(f, "str"),
            PrimitiveType::Unit => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        assert!(Type::primitive(PrimitiveType::I32).is_integer());
        assert!(Type::primitive(PrimitiveType::F64).is_float());
        assert!(Type::primitive(PrimitiveType::Bool).is_bool());
        assert!(!Type::primitive(PrimitiveType::I32).is_float());
    }

    #[test]
    fn test_function_type() {
        let func = Type::function(
            vec![Type::primitive(PrimitiveType::I32)],
            Some(Type::primitive(PrimitiveType::Bool)),
        );
        assert!(matches!(func, Type::Function(_)));
    }

    #[test]
    fn test_type_display() {
        assert_eq!(Type::primitive(PrimitiveType::I32).to_string(), "i32");
        assert_eq!(Type::Never.to_string(), "!");
        assert_eq!(
            Type::reference(Type::primitive(PrimitiveType::I32), false).to_string(),
            "&i32"
        );
        assert_eq!(
            Type::signal(Type::primitive(PrimitiveType::I32)).to_string(),
            "Signal<i32>"
        );
    }

    #[test]
    fn test_type_assignability() {
        let i32_ty = Type::primitive(PrimitiveType::I32);
        let i64_ty = Type::primitive(PrimitiveType::I64);

        assert!(i32_ty.is_assignable_to(&i32_ty));
        assert!(!i32_ty.is_assignable_to(&i64_ty));
        assert!(i32_ty.is_assignable_to(&Type::Unknown));
    }
}

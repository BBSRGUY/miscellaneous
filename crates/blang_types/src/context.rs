//! Type context and environment for type checking.

use crate::types::Type;
use rustc_hash::FxHashMap;

/// Type context that manages scopes and bindings.
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Stack of scopes (innermost scope is last)
    scopes: Vec<Scope>,
    /// Function signatures
    functions: FxHashMap<String, Type>,
    /// Type definitions
    types: FxHashMap<String, Type>,
    /// Current function return type (for checking return statements)
    current_function_return: Option<Type>,
    /// Are we in a loop? (for checking break/continue)
    in_loop: bool,
}

/// A single scope in the type environment.
#[derive(Debug, Clone)]
struct Scope {
    /// Variable bindings in this scope
    vars: FxHashMap<String, VarBinding>,
}

/// A variable binding with type and mutability information.
#[derive(Debug, Clone)]
struct VarBinding {
    ty: Type,
    mutable: bool,
}

impl TypeContext {
    /// Create a new type context with builtins.
    pub fn new() -> Self {
        let mut ctx = TypeContext {
            scopes: vec![Scope::new()],
            functions: FxHashMap::default(),
            types: FxHashMap::default(),
            current_function_return: None,
            in_loop: false,
        };

        // Add builtin functions
        ctx.add_builtin_functions();

        ctx
    }

    /// Add builtin functions to the context.
    fn add_builtin_functions(&mut self) {
        use crate::types::PrimitiveType;

        // println function
        self.add_function(
            "println".to_string(),
            Type::function(vec![Type::primitive(PrimitiveType::Str)], None),
        );

        // len function for arrays/slices
        self.add_function(
            "len".to_string(),
            Type::function(
                vec![Type::Generic("T".to_string())],
                Some(Type::primitive(PrimitiveType::U64)),
            ),
        );
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Add a variable binding to the current scope.
    pub fn add_variable(&mut self, name: String, ty: Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(name, VarBinding { ty, mutable });
        }
    }

    /// Look up a variable in the environment.
    pub fn get_variable(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.vars.get(name) {
                return Some(&binding.ty);
            }
        }
        None
    }

    /// Check if a variable is mutable.
    pub fn is_variable_mutable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.vars.get(name) {
                return binding.mutable;
            }
        }
        false
    }

    /// Add a function signature.
    pub fn add_function(&mut self, name: String, ty: Type) {
        self.functions.insert(name, ty);
    }

    /// Look up a function signature.
    pub fn get_function(&self, name: &str) -> Option<&Type> {
        self.functions.get(name)
    }

    /// Add a type definition.
    pub fn add_type(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    /// Look up a type definition.
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    /// Set the current function's return type.
    pub fn set_function_return(&mut self, ty: Option<Type>) {
        self.current_function_return = ty;
    }

    /// Get the current function's return type.
    pub fn get_function_return(&self) -> Option<&Type> {
        self.current_function_return.as_ref()
    }

    /// Enter a loop context.
    pub fn enter_loop(&mut self) {
        self.in_loop = true;
    }

    /// Exit a loop context.
    pub fn exit_loop(&mut self) {
        self.in_loop = false;
    }

    /// Check if we're in a loop.
    pub fn in_loop(&self) -> bool {
        self.in_loop
    }

    /// Create a child context with a new scope.
    pub fn with_scope(&self) -> TypeContext {
        let mut ctx = self.clone();
        ctx.push_scope();
        ctx
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    /// Create a new empty scope.
    fn new() -> Self {
        Scope {
            vars: FxHashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrimitiveType;

    #[test]
    fn test_variable_binding() {
        let mut ctx = TypeContext::new();
        let i32_ty = Type::primitive(PrimitiveType::I32);

        ctx.add_variable("x".to_string(), i32_ty.clone(), false);
        assert_eq!(ctx.get_variable("x"), Some(&i32_ty));
        assert!(!ctx.is_variable_mutable("x"));
    }

    #[test]
    fn test_scopes() {
        let mut ctx = TypeContext::new();
        let i32_ty = Type::primitive(PrimitiveType::I32);
        let bool_ty = Type::primitive(PrimitiveType::Bool);

        ctx.add_variable("x".to_string(), i32_ty.clone(), false);
        ctx.push_scope();
        ctx.add_variable("x".to_string(), bool_ty.clone(), false);

        assert_eq!(ctx.get_variable("x"), Some(&bool_ty));
        ctx.pop_scope();
        assert_eq!(ctx.get_variable("x"), Some(&i32_ty));
    }

    #[test]
    fn test_function_lookup() {
        let ctx = TypeContext::new();
        // println is a builtin
        assert!(ctx.get_function("println").is_some());
    }

    #[test]
    fn test_loop_context() {
        let mut ctx = TypeContext::new();
        assert!(!ctx.in_loop());
        ctx.enter_loop();
        assert!(ctx.in_loop());
        ctx.exit_loop();
        assert!(!ctx.in_loop());
    }

    #[test]
    fn test_function_return() {
        let mut ctx = TypeContext::new();
        let i32_ty = Type::primitive(PrimitiveType::I32);

        assert!(ctx.get_function_return().is_none());
        ctx.set_function_return(Some(i32_ty.clone()));
        assert_eq!(ctx.get_function_return(), Some(&i32_ty));
    }
}

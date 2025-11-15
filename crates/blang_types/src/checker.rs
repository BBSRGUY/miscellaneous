//! Type checker for Blang AST.

use crate::context::TypeContext;
use crate::error::{TypeError, TypeErrorKind, TypeResult};
use crate::types::{Mutability, PrimitiveType, Type};
use blang_ast::*;

/// Type checker for Blang programs.
pub struct TypeChecker {
    ctx: TypeContext,
}

impl TypeChecker {
    /// Create a new type checker.
    pub fn new() -> Self {
        TypeChecker {
            ctx: TypeContext::new(),
        }
    }

    /// Type check a list of items (top-level declarations).
    pub fn check_items(&mut self, items: &[Item]) -> TypeResult<()> {
        // First pass: collect function signatures and type definitions
        for item in items {
            self.collect_item_signature(item)?;
        }

        // Second pass: type check item bodies
        for item in items {
            self.check_item(item)?;
        }

        Ok(())
    }

    /// Collect the signature of an item (first pass).
    fn collect_item_signature(&mut self, item: &Item) -> TypeResult<()> {
        match &item.kind {
            ItemKind::Function(func) => {
                let func_ty = self.function_type(func)?;
                self.ctx.add_function(func.sig.name.name.clone(), func_ty);
            }
            ItemKind::Struct(struct_decl) => {
                // For now, just register the struct name as a named type
                self.ctx
                    .add_type(struct_decl.name.name.clone(), Type::Named(struct_decl.name.name.clone()));
            }
            ItemKind::Enum(enum_decl) => {
                self.ctx
                    .add_type(enum_decl.name.name.clone(), Type::Named(enum_decl.name.name.clone()));
            }
            ItemKind::TypeAlias(alias) => {
                // Convert AST type to our Type representation
                let ty = self.convert_ast_type(&alias.ty)?;
                self.ctx.add_type(alias.name.name.clone(), ty);
            }
            ItemKind::Const(const_decl) => {
                let ty = self.convert_ast_type(&const_decl.ty)?;
                self.ctx.add_variable(const_decl.name.name.clone(), ty, false);
            }
            _ => {
                // Other items don't need signature collection
            }
        }
        Ok(())
    }

    /// Type check an item (second pass).
    fn check_item(&mut self, item: &Item) -> TypeResult<()> {
        match &item.kind {
            ItemKind::Function(func) => self.check_function(func),
            ItemKind::Const(const_decl) => {
                let ty = self.convert_ast_type(&const_decl.ty)?;
                let value_ty = self.check_expr(&const_decl.value)?;
                self.check_type_match(&ty, &value_ty, const_decl.value.span)?;
                Ok(())
            }
            ItemKind::Component(comp) => self.check_component(comp),
            ItemKind::Script(script) => self.check_script(script),
            _ => Ok(()), // Other items don't have bodies to check yet
        }
    }

    /// Get the function type from a function declaration.
    fn function_type(&self, func: &FunctionDecl) -> TypeResult<Type> {
        let mut params = Vec::new();
        for param in &func.sig.params {
            let ty = self.convert_ast_type(&param.ty)?;
            params.push(ty);
        }

        let return_type = if let Some(ref ret) = func.sig.return_ty {
            Some(self.convert_ast_type(ret)?)
        } else {
            None
        };

        Ok(Type::function(params, return_type))
    }

    /// Type check a function declaration.
    fn check_function(&mut self, func: &FunctionDecl) -> TypeResult<()> {
        // Set up function context
        let return_type = if let Some(ref ret) = func.sig.return_ty {
            Some(self.convert_ast_type(ret)?)
        } else {
            None
        };

        self.ctx.set_function_return(return_type.clone());
        self.ctx.push_scope();

        // Add parameters to scope
        for param in &func.sig.params {
            let ty = self.convert_ast_type(&param.ty)?;
            self.ctx.add_variable(param.pat.to_string(), ty, false);
        }

        // Check function body
        if let Some(ref body) = func.body {
            let body_ty = self.check_block(body)?;

            // Check return type matches
            if let Some(expected) = &return_type {
                if !body_ty.is_unit() && !body_ty.is_never() {
                    self.check_type_match(expected, &body_ty, body.span)?;
                }
            }
        }

        self.ctx.pop_scope();
        self.ctx.set_function_return(None);

        Ok(())
    }

    /// Type check a component declaration.
    fn check_component(&mut self, _comp: &ComponentDecl) -> TypeResult<()> {
        // Component type checking is simplified for now
        // Full implementation would check props, state, view, etc.
        Ok(())
    }

    /// Type check a script declaration.
    fn check_script(&mut self, _script: &ScriptDecl) -> TypeResult<()> {
        // Script type checking is simplified for now
        Ok(())
    }

    /// Type check a statement.
    pub fn check_stmt(&mut self, stmt: &Stmt) -> TypeResult<Type> {
        match &stmt.kind {
            StmtKind::Let(let_stmt) => {
                let value_ty = if let Some(ref init) = let_stmt.init {
                    self.check_expr(init)?
                } else {
                    // No initializer - use annotated type or error
                    if let Some(ref ty) = let_stmt.ty {
                        self.convert_ast_type(ty)?
                    } else {
                        return Err(TypeError::generic(
                            "let binding requires either type annotation or initializer".to_string(),
                            stmt.span,
                        ));
                    }
                };

                // Check type annotation matches if present
                if let Some(ref ty) = let_stmt.ty {
                    let expected = self.convert_ast_type(ty)?;
                    self.check_type_match(&expected, &value_ty, stmt.span)?;
                }

                // Add variable to context
                let var_name = let_stmt.pat.to_string();
                self.ctx
                    .add_variable(var_name, value_ty, let_stmt.mutable);

                Ok(Type::primitive(PrimitiveType::Unit))
            }

            StmtKind::Expr(expr, _has_semi) => self.check_expr(expr),

            StmtKind::Item(item) => {
                self.check_item(item)?;
                Ok(Type::primitive(PrimitiveType::Unit))
            }

            StmtKind::Error => Ok(Type::Unknown),
        }
    }

    /// Type check an expression.
    pub fn check_expr(&mut self, expr: &Expr) -> TypeResult<Type> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(self.check_literal(lit)),

            ExprKind::Path(path) => {
                // Look up variable or function
                let name = path.to_string();
                if let Some(ty) = self.ctx.get_variable(&name) {
                    Ok(ty.clone())
                } else if let Some(ty) = self.ctx.get_function(&name) {
                    Ok(ty.clone())
                } else {
                    Err(TypeError::undefined_variable(name, expr.span))
                }
            }

            ExprKind::Binary(bin) => self.check_binary_expr(bin, expr.span),

            ExprKind::Unary(un) => self.check_unary_expr(un, expr.span),

            ExprKind::Call(call) => self.check_call_expr(call, expr.span),

            ExprKind::MethodCall(method) => self.check_method_call_expr(method, expr.span),

            ExprKind::Field(field) => {
                let _base_ty = self.check_expr(&field.expr)?;
                // Field access type checking requires struct/enum type info
                // For now, return Unknown
                Ok(Type::Unknown)
            }

            ExprKind::Index(index) => self.check_index_expr(index, expr.span),

            ExprKind::Range(_) => {
                // Range types would need proper support
                Ok(Type::Unknown)
            }

            ExprKind::Closure(closure) => self.check_closure_expr(closure, expr.span),

            ExprKind::Return(ret_expr) => {
                let ret_ty = if let Some(ref e) = ret_expr {
                    self.check_expr(e)?
                } else {
                    Type::primitive(PrimitiveType::Unit)
                };

                if let Some(expected) = self.ctx.get_function_return() {
                    self.check_type_match(expected, &ret_ty, expr.span)?;
                }

                Ok(Type::Never)
            }

            ExprKind::Break(_) => {
                if !self.ctx.in_loop() {
                    return Err(TypeError::new(
                        TypeErrorKind::BreakOutsideLoop,
                        expr.span,
                    ));
                }
                Ok(Type::Never)
            }

            ExprKind::Continue => {
                if !self.ctx.in_loop() {
                    return Err(TypeError::new(
                        TypeErrorKind::ContinueOutsideLoop,
                        expr.span,
                    ));
                }
                Ok(Type::Never)
            }

            ExprKind::Array(arr) => self.check_array_expr(arr, expr.span),

            ExprKind::Tuple(elems) => {
                let mut types = Vec::new();
                for elem in elems {
                    types.push(self.check_expr(elem)?);
                }
                Ok(Type::Tuple(types))
            }

            ExprKind::Struct(_) => {
                // Struct literal type checking requires struct type info
                Ok(Type::Unknown)
            }

            ExprKind::Await(e) => {
                // Await type checking requires async/Future support
                let _inner_ty = self.check_expr(e)?;
                Ok(Type::Unknown)
            }

            ExprKind::Cast(cast) => {
                let _from_ty = self.check_expr(&cast.expr)?;
                self.convert_ast_type(&cast.ty)
            }

            ExprKind::Reference(ref_expr) => {
                let inner_ty = self.check_expr(&ref_expr.expr)?;
                Ok(Type::reference(inner_ty, ref_expr.mutable))
            }

            ExprKind::Dereference(e) => {
                let ty = self.check_expr(e)?;
                if let Some(inner) = ty.deref() {
                    Ok(inner.clone())
                } else {
                    Err(TypeError::new(
                        TypeErrorKind::CannotDereference { ty },
                        expr.span,
                    ))
                }
            }

            ExprKind::Block(block) => self.check_block(block),

            ExprKind::If(if_expr) => self.check_if_expr(if_expr, expr.span),

            ExprKind::Match(match_expr) => self.check_match_expr(match_expr, expr.span),

            ExprKind::Loop(loop_expr) => self.check_loop_expr(loop_expr, expr.span),

            ExprKind::While(while_expr) => self.check_while_expr(while_expr, expr.span),

            ExprKind::For(for_expr) => self.check_for_expr(for_expr, expr.span),

            ExprKind::Unsafe(block) => self.check_block(block),

            ExprKind::Signal(sig_expr) => {
                let inner_ty = self.check_expr(sig_expr)?;
                Ok(Type::signal(inner_ty))
            }

            ExprKind::Effect(eff_expr) => {
                self.check_expr(eff_expr)?;
                Ok(Type::primitive(PrimitiveType::Unit))
            }

            ExprKind::Memo(memo_expr) => self.check_expr(memo_expr),

            ExprKind::Resource(_) => {
                // Resource type checking requires async support
                Ok(Type::Unknown)
            }

            ExprKind::Paren(e) => self.check_expr(e),

            ExprKind::Error => Ok(Type::Unknown),
        }
    }

    /// Check a literal expression.
    fn check_literal(&self, lit: &Lit) -> Type {
        match lit {
            Lit::Int(_) => Type::primitive(PrimitiveType::I32), // Default to i32
            Lit::Float(_) => Type::primitive(PrimitiveType::F64), // Default to f64
            Lit::Bool(_) => Type::primitive(PrimitiveType::Bool),
            Lit::Char(_) => Type::primitive(PrimitiveType::Char),
            Lit::Str(_) => Type::primitive(PrimitiveType::Str),
            Lit::Template(_) => Type::primitive(PrimitiveType::Str),
        }
    }

    /// Check a binary expression.
    fn check_binary_expr(&mut self, bin: &BinaryExpr, span: blang_span::Span) -> TypeResult<Type> {
        let left_ty = self.check_expr(&bin.left)?;
        let right_ty = self.check_expr(&bin.right)?;

        match bin.op {
            // Arithmetic operators: require numeric types
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
                if !left_ty.is_numeric() {
                    return Err(TypeError::invalid_binary_op(
                        bin.op.to_string(),
                        left_ty,
                        right_ty,
                        span,
                    ));
                }
                self.check_type_match(&left_ty, &right_ty, span)?;
                Ok(left_ty)
            }

            // Comparison operators: require same types, return bool
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                self.check_type_match(&left_ty, &right_ty, span)?;
                Ok(Type::primitive(PrimitiveType::Bool))
            }

            // Logical operators: require bool, return bool
            BinaryOp::And | BinaryOp::Or => {
                if !left_ty.is_bool() || !right_ty.is_bool() {
                    return Err(TypeError::invalid_binary_op(
                        bin.op.to_string(),
                        left_ty,
                        right_ty,
                        span,
                    ));
                }
                Ok(Type::primitive(PrimitiveType::Bool))
            }

            // Bitwise operators: require integer types
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
                if !left_ty.is_integer() || !right_ty.is_integer() {
                    return Err(TypeError::invalid_binary_op(
                        bin.op.to_string(),
                        left_ty,
                        right_ty,
                        span,
                    ));
                }
                Ok(left_ty)
            }

            // Assignment operators
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
            | BinaryOp::ShrAssign => {
                self.check_type_match(&left_ty, &right_ty, span)?;
                Ok(Type::primitive(PrimitiveType::Unit))
            }
        }
    }

    /// Check a unary expression.
    fn check_unary_expr(&mut self, un: &UnaryExpr, span: blang_span::Span) -> TypeResult<Type> {
        let ty = self.check_expr(&un.expr)?;

        match un.op {
            UnaryOp::Neg => {
                if !ty.is_numeric() {
                    return Err(TypeError::invalid_unary_op(un.op.to_string(), ty, span));
                }
                Ok(ty)
            }
            UnaryOp::Not => {
                if !ty.is_bool() {
                    return Err(TypeError::invalid_unary_op(un.op.to_string(), ty, span));
                }
                Ok(ty)
            }
            UnaryOp::BitNot => {
                if !ty.is_integer() {
                    return Err(TypeError::invalid_unary_op(un.op.to_string(), ty, span));
                }
                Ok(ty)
            }
        }
    }

    /// Check a call expression.
    fn check_call_expr(&mut self, call: &CallExpr, span: blang_span::Span) -> TypeResult<Type> {
        let func_ty = self.check_expr(&call.func)?;

        match func_ty {
            Type::Function(ref func) => {
                // Check argument count
                if call.args.len() != func.params.len() {
                    return Err(TypeError::wrong_arg_count(
                        func.params.len(),
                        call.args.len(),
                        span,
                    ));
                }

                // Check argument types
                for (arg, param_ty) in call.args.iter().zip(&func.params) {
                    let arg_ty = self.check_expr(arg)?;
                    self.check_type_match(param_ty, &arg_ty, arg.span)?;
                }

                // Return the return type
                Ok(func
                    .return_type
                    .as_ref()
                    .map(|t| (**t).clone())
                    .unwrap_or(Type::primitive(PrimitiveType::Unit)))
            }
            _ => Err(TypeError::cannot_call(func_ty, span)),
        }
    }

    /// Check a method call expression.
    fn check_method_call_expr(
        &mut self,
        method: &MethodCallExpr,
        _span: blang_span::Span,
    ) -> TypeResult<Type> {
        let _receiver_ty = self.check_expr(&method.receiver)?;

        // Method resolution requires trait/impl support
        // For now, type check arguments and return Unknown
        for arg in &method.args {
            self.check_expr(arg)?;
        }

        Ok(Type::Unknown)
    }

    /// Check an index expression.
    fn check_index_expr(&mut self, index: &IndexExpr, span: blang_span::Span) -> TypeResult<Type> {
        let base_ty = self.check_expr(&index.expr)?;
        let index_ty = self.check_expr(&index.index)?;

        // Index must be integer
        if !index_ty.is_integer() {
            return Err(TypeError::generic(
                format!("array index must be integer, found `{}`", index_ty),
                span,
            ));
        }

        match base_ty {
            Type::Array(elem_ty, _) | Type::Slice(elem_ty) => Ok((*elem_ty).clone()),
            _ => Err(TypeError::new(
                TypeErrorKind::CannotIndex { ty: base_ty },
                span,
            )),
        }
    }

    /// Check a closure expression.
    fn check_closure_expr(
        &mut self,
        closure: &ClosureExpr,
        _span: blang_span::Span,
    ) -> TypeResult<Type> {
        self.ctx.push_scope();

        // Add parameters to scope
        let mut param_types = Vec::new();
        for param in &closure.params {
            let ty = if let Some(ref ty) = param.ty {
                self.convert_ast_type(ty)?
            } else {
                // Type inference would be needed here
                Type::Unknown
            };
            self.ctx
                .add_variable(param.pat.to_string(), ty.clone(), false);
            param_types.push(ty);
        }

        // Check body
        let body_ty = self.check_expr(&closure.body)?;

        self.ctx.pop_scope();

        Ok(Type::function(param_types, Some(body_ty)))
    }

    /// Check an array expression.
    fn check_array_expr(&mut self, arr: &ArrayExpr, _span: blang_span::Span) -> TypeResult<Type> {
        match arr {
            ArrayExpr::List(elems) => {
                if elems.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Unknown), 0));
                }

                let first_ty = self.check_expr(&elems[0])?;

                // Check all elements have the same type
                for elem in &elems[1..] {
                    let elem_ty = self.check_expr(elem)?;
                    self.check_type_match(&first_ty, &elem_ty, elem.span)?;
                }

                Ok(Type::Array(Box::new(first_ty), elems.len()))
            }
            ArrayExpr::Repeat { elem, len } => {
                let elem_ty = self.check_expr(elem)?;
                // Note: len is an expression, we can't determine size at compile time in general
                // For now, return Slice type or use 0 as placeholder
                let _len_ty = self.check_expr(len)?;
                // TODO: evaluate constant expressions to get actual length
                Ok(Type::Slice(Box::new(elem_ty)))
            }
        }
    }

    /// Check a block expression.
    fn check_block(&mut self, block: &Block) -> TypeResult<Type> {
        self.ctx.push_scope();

        let mut last_ty = Type::primitive(PrimitiveType::Unit);

        for stmt in &block.stmts {
            last_ty = self.check_stmt(stmt)?;

            // If we hit a never type (return/break/continue), stop checking
            if last_ty.is_never() {
                break;
            }
        }

        self.ctx.pop_scope();
        Ok(last_ty)
    }

    /// Check an if expression.
    fn check_if_expr(&mut self, if_expr: &IfExpr, span: blang_span::Span) -> TypeResult<Type> {
        let cond_ty = self.check_expr(&if_expr.cond)?;
        if !cond_ty.is_bool_compatible() {
            return Err(TypeError::condition_not_bool(cond_ty, span));
        }

        let then_ty = self.check_block(&if_expr.then_block)?;

        if let Some(ref else_expr) = if_expr.else_block {
            let else_ty = self.check_expr(else_expr)?;
            self.check_type_match(&then_ty, &else_ty, span)?;
            Ok(then_ty)
        } else {
            Ok(Type::primitive(PrimitiveType::Unit))
        }
    }

    /// Check a match expression.
    fn check_match_expr(
        &mut self,
        match_expr: &MatchExpr,
        span: blang_span::Span,
    ) -> TypeResult<Type> {
        let scrutinee_ty = self.check_expr(&match_expr.expr)?;

        if match_expr.arms.is_empty() {
            return Ok(Type::primitive(PrimitiveType::Unit));
        }

        // Check first arm to get result type
        let first_arm_ty = self.check_expr(&match_expr.arms[0].body)?;

        // Check all arms have same type
        for arm in &match_expr.arms[1..] {
            let _pat_ty = scrutinee_ty.clone(); // Pattern matching would check this
            let arm_ty = self.check_expr(&arm.body)?;
            self.check_type_match(&first_arm_ty, &arm_ty, span)?;
        }

        Ok(first_arm_ty)
    }

    /// Check a loop expression.
    fn check_loop_expr(&mut self, loop_expr: &LoopExpr, _span: blang_span::Span) -> TypeResult<Type> {
        self.ctx.enter_loop();
        self.check_block(&loop_expr.body)?;
        self.ctx.exit_loop();

        // Loops without break never return
        Ok(Type::Never)
    }

    /// Check a while expression.
    fn check_while_expr(
        &mut self,
        while_expr: &WhileExpr,
        span: blang_span::Span,
    ) -> TypeResult<Type> {
        let cond_ty = self.check_expr(&while_expr.cond)?;
        if !cond_ty.is_bool_compatible() {
            return Err(TypeError::condition_not_bool(cond_ty, span));
        }

        self.ctx.enter_loop();
        self.check_block(&while_expr.body)?;
        self.ctx.exit_loop();

        Ok(Type::primitive(PrimitiveType::Unit))
    }

    /// Check a for expression.
    fn check_for_expr(&mut self, for_expr: &ForExpr, _span: blang_span::Span) -> TypeResult<Type> {
        self.ctx.push_scope();

        // Type check iterator
        let _iter_ty = self.check_expr(&for_expr.iter)?;

        // Add loop variable to scope (type inference would determine its type)
        self.ctx.add_variable(
            for_expr.pat.to_string(),
            Type::Unknown,
            false,
        );

        self.ctx.enter_loop();
        self.check_block(&for_expr.body)?;
        self.ctx.exit_loop();

        self.ctx.pop_scope();
        Ok(Type::primitive(PrimitiveType::Unit))
    }

    /// Convert an AST type to our Type representation.
    fn convert_ast_type(&self, ty: &Ty) -> TypeResult<Type> {
        match &ty.kind {
            TyKind::Primitive(prim) => Ok(Type::primitive(self.convert_primitive(*prim))),

            TyKind::Path(path) => {
                let name = path.to_string();
                // Check if it's a known type
                if let Some(known_ty) = self.ctx.get_type(&name) {
                    Ok(known_ty.clone())
                } else {
                    // Assume it's a named type (struct/enum)
                    Ok(Type::Named(name))
                }
            }

            TyKind::Reference(ref_ty) => {
                let inner = self.convert_ast_type(&ref_ty.ty)?;
                Ok(Type::Reference(
                    Box::new(inner),
                    if ref_ty.mutable {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    },
                ))
            }

            TyKind::Pointer(ptr_ty) => {
                let inner = self.convert_ast_type(&ptr_ty.ty)?;
                Ok(Type::Pointer(
                    Box::new(inner),
                    if ptr_ty.mutable {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    },
                ))
            }

            TyKind::Array(arr_ty) => {
                let elem_ty = self.convert_ast_type(&arr_ty.elem_ty)?;
                let size = match &arr_ty.len {
                    ArrayLen::Literal(n) => *n as usize,
                    ArrayLen::Const(_) => 0, // Would need constant evaluation
                };
                Ok(Type::Array(Box::new(elem_ty), size))
            }

            TyKind::Slice(elem_ty) => {
                let ty = self.convert_ast_type(elem_ty)?;
                Ok(Type::Slice(Box::new(ty)))
            }

            TyKind::Tuple(types) => {
                let mut converted = Vec::new();
                for ty in types {
                    converted.push(self.convert_ast_type(ty)?);
                }
                Ok(Type::Tuple(converted))
            }

            TyKind::Function(func_ty) => {
                let mut params = Vec::new();
                for param in &func_ty.params {
                    params.push(self.convert_ast_type(param)?);
                }
                let return_type = if let Some(ref ret) = func_ty.return_ty {
                    Some(self.convert_ast_type(ret)?)
                } else {
                    None
                };
                Ok(Type::function(params, return_type))
            }

            TyKind::Never => Ok(Type::Never),

            TyKind::Error => Ok(Type::Unknown),
        }
    }

    /// Convert an AST primitive type to our PrimitiveType.
    fn convert_primitive(&self, prim: blang_ast::PrimitiveTy) -> PrimitiveType {
        match prim {
            blang_ast::PrimitiveTy::I8 => PrimitiveType::I8,
            blang_ast::PrimitiveTy::I16 => PrimitiveType::I16,
            blang_ast::PrimitiveTy::I32 => PrimitiveType::I32,
            blang_ast::PrimitiveTy::I64 => PrimitiveType::I64,
            blang_ast::PrimitiveTy::U8 => PrimitiveType::U8,
            blang_ast::PrimitiveTy::U16 => PrimitiveType::U16,
            blang_ast::PrimitiveTy::U32 => PrimitiveType::U32,
            blang_ast::PrimitiveTy::U64 => PrimitiveType::U64,
            blang_ast::PrimitiveTy::F32 => PrimitiveType::F32,
            blang_ast::PrimitiveTy::F64 => PrimitiveType::F64,
            blang_ast::PrimitiveTy::Bool => PrimitiveType::Bool,
            blang_ast::PrimitiveTy::Char => PrimitiveType::Char,
            blang_ast::PrimitiveTy::Str => PrimitiveType::Str,
            blang_ast::PrimitiveTy::Unit => PrimitiveType::Unit,
        }
    }

    /// Check that two types match.
    fn check_type_match(
        &self,
        expected: &Type,
        found: &Type,
        span: blang_span::Span,
    ) -> TypeResult<()> {
        if expected == found || matches!(expected, Type::Unknown) || matches!(found, Type::Unknown)
        {
            Ok(())
        } else {
            Err(TypeError::mismatch(expected.clone(), found.clone(), span))
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

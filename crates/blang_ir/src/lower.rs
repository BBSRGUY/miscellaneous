//! AST to IR lowering
//!
//! This module converts type-checked AST into the Blang IR.

use crate::ir::*;
use blang_ast::{
    BinaryOp as AstBinaryOp, Block, Expr, ExprKind, FunctionDecl, Item, ItemKind, Lit, Stmt,
    StmtKind, UnaryOp as AstUnaryOp,
};
use blang_types::{PrimitiveType, Type};
use std::collections::HashMap;

/// Error type for lowering
#[derive(Debug, thiserror::Error)]
pub enum LowerError {
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
    #[error("Unsupported expression: {0}")]
    UnsupportedExpr(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Invalid constant: {0}")]
    InvalidConstant(String),
}

pub type LowerResult<T> = Result<T, LowerError>;

/// Context for lowering
struct LowerContext {
    /// Current function being lowered
    function: Function,
    /// Current basic block
    current_block: BlockId,
    /// Variable to register mapping
    variables: HashMap<String, Register>,
    /// Loop context stack (continue_block, break_block)
    loop_stack: Vec<(BlockId, BlockId)>,
}

impl LowerContext {
    fn new(function: Function, entry_block: BlockId) -> Self {
        Self {
            function,
            current_block: entry_block,
            variables: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    fn new_register(&mut self) -> Register {
        self.function.new_register()
    }

    fn new_block(&mut self) -> BlockId {
        self.function.new_block()
    }

    fn add_instruction(&mut self, inst: Instruction) {
        if let Some(block) = self.function.get_block_mut(self.current_block) {
            block.instructions.push(inst);
        }
    }

    fn set_terminator(&mut self, term: Terminator) {
        if let Some(block) = self.function.get_block_mut(self.current_block) {
            block.terminator = Some(term);
        }
    }

    fn bind_variable(&mut self, name: String, reg: Register) {
        self.variables.insert(name, reg);
    }

    fn get_variable(&self, name: &str) -> Option<Register> {
        self.variables.get(name).copied()
    }

    fn enter_loop(&mut self, continue_block: BlockId, break_block: BlockId) {
        self.loop_stack.push((continue_block, break_block));
    }

    fn exit_loop(&mut self) {
        self.loop_stack.pop();
    }

    fn current_loop(&self) -> Option<(BlockId, BlockId)> {
        self.loop_stack.last().copied()
    }
}

/// IR Lowerer
pub struct IrLowerer {
    module: Module,
}

impl IrLowerer {
    pub fn new(module_name: String) -> Self {
        Self {
            module: Module::new(module_name),
        }
    }

    /// Lower a list of items (top-level declarations)
    pub fn lower_items(&mut self, items: &[Item]) -> LowerResult<()> {
        for item in items {
            self.lower_item(item)?;
        }
        Ok(())
    }

    /// Lower a single item
    fn lower_item(&mut self, item: &Item) -> LowerResult<()> {
        match &item.kind {
            ItemKind::Function(func) => {
                let ir_func = self.lower_function(func)?;
                self.module.add_function(ir_func);
            }
            ItemKind::Const(const_decl) => {
                // Lower constants as globals
                let ty = convert_type(&Type::primitive(PrimitiveType::I32))?; // Placeholder
                let init = if let ExprKind::Literal(lit) = &const_decl.value.kind {
                    Some(lower_literal(lit)?)
                } else {
                    None
                };
                self.module.add_global(Global {
                    name: const_decl.name.name.clone(),
                    ty,
                    init,
                    mutable: false,
                });
            }
            _ => {
                // Skip other items for now (components, scripts, etc.)
            }
        }
        Ok(())
    }

    /// Lower a function declaration
    fn lower_function(&mut self, func: &FunctionDecl) -> LowerResult<Function> {
        let func_id = FunctionId(func.sig.name.name.clone());

        // Convert parameter types
        let mut params = Vec::new();
        for (i, param) in func.sig.params.iter().enumerate() {
            let ty = convert_type(&Type::primitive(PrimitiveType::I32))?; // Placeholder
            params.push(Parameter {
                name: param.pat.to_string(),
                ty,
                register: Register(i as u32),
            });
        }

        // Convert return type
        let return_type = if let Some(ref _ret_ty) = func.sig.return_ty {
            convert_type(&Type::primitive(PrimitiveType::I32))? // Placeholder
        } else {
            IrType::Unit
        };

        let mut function = Function::new(func_id, params.clone(), return_type);
        let entry_block = function.new_block();

        let mut ctx = LowerContext::new(function, entry_block);

        // Bind parameters to registers
        for param in &params {
            ctx.bind_variable(param.name.clone(), param.register);
        }

        // Lower function body
        if let Some(ref body) = func.body {
            self.lower_block(&mut ctx, body)?;
        } else {
            // Empty function, just return
            ctx.set_terminator(Terminator::Return(None));
        }

        Ok(ctx.function)
    }

    /// Lower a block of statements
    fn lower_block(&mut self, ctx: &mut LowerContext, block: &Block) -> LowerResult<Option<Value>> {
        let mut last_value = None;

        for stmt in &block.stmts {
            last_value = self.lower_stmt(ctx, stmt)?;
        }

        Ok(last_value)
    }

    /// Lower a statement
    fn lower_stmt(&mut self, ctx: &mut LowerContext, stmt: &Stmt) -> LowerResult<Option<Value>> {
        match &stmt.kind {
            StmtKind::Let(let_stmt) => {
                // Lower the initializer
                if let Some(ref init) = let_stmt.init {
                    let value = self.lower_expr(ctx, init)?;
                    let reg = ctx.new_register();
                    let var_name = let_stmt.pat.to_string();

                    // Create a copy instruction
                    ctx.add_instruction(Instruction::Copy {
                        result: reg,
                        value,
                        ty: IrType::I32, // Placeholder
                    });

                    ctx.bind_variable(var_name, reg);
                }
                Ok(None)
            }
            StmtKind::Expr(expr, _has_semi) => {
                let value = self.lower_expr(ctx, expr)?;
                Ok(Some(value))
            }
            StmtKind::Item(_) => {
                // Skip nested items
                Ok(None)
            }
            StmtKind::Error => {
                // Skip error nodes
                Ok(None)
            }
        }
    }

    /// Lower an expression to a value
    fn lower_expr(&mut self, ctx: &mut LowerContext, expr: &Expr) -> LowerResult<Value> {
        match &expr.kind {
            ExprKind::Literal(lit) => {
                let constant = lower_literal(lit)?;
                Ok(Value::Constant(constant))
            }

            ExprKind::Path(path) => {
                let var_name = path.to_string();
                if let Some(reg) = ctx.get_variable(&var_name) {
                    Ok(Value::Register(reg))
                } else {
                    Err(LowerError::UndefinedVariable(var_name))
                }
            }

            ExprKind::Binary(bin) => {
                let lhs = self.lower_expr(ctx, &bin.left)?;
                let rhs = self.lower_expr(ctx, &bin.right)?;
                let result = ctx.new_register();

                let op = convert_binary_op(&bin.op);

                ctx.add_instruction(Instruction::Binary {
                    result,
                    op,
                    lhs,
                    rhs,
                    ty: IrType::I32, // Placeholder
                });

                Ok(Value::Register(result))
            }

            ExprKind::Unary(un) => {
                let operand = self.lower_expr(ctx, &un.expr)?;
                let result = ctx.new_register();

                let op = convert_unary_op(&un.op);

                ctx.add_instruction(Instruction::Unary {
                    result,
                    op,
                    operand,
                    ty: IrType::I32, // Placeholder
                });

                Ok(Value::Register(result))
            }

            ExprKind::Call(call) => {
                // Lower function arguments
                let mut args = Vec::new();
                for arg in &call.args {
                    args.push(self.lower_expr(ctx, arg)?);
                }

                let func_id = if let ExprKind::Path(path) = &call.func.kind {
                    FunctionId(path.to_string())
                } else {
                    return Err(LowerError::UnsupportedExpr("complex call".to_string()));
                };

                let result = ctx.new_register();

                ctx.add_instruction(Instruction::Call {
                    result: Some(result),
                    func: func_id,
                    args,
                    ret_ty: IrType::I32, // Placeholder
                });

                Ok(Value::Register(result))
            }

            ExprKind::Block(block) => {
                let value = self.lower_block(ctx, block)?;
                Ok(value.unwrap_or(Value::Constant(Constant::Unit)))
            }

            ExprKind::If(if_expr) => {
                let cond = self.lower_expr(ctx, &if_expr.cond)?;

                let then_block = ctx.new_block();
                let else_block = ctx.new_block();
                let merge_block = ctx.new_block();

                // Conditional branch
                ctx.set_terminator(Terminator::CondBranch {
                    cond,
                    true_block: then_block,
                    false_block: else_block,
                });

                // Lower then block
                ctx.current_block = then_block;
                let then_value = self.lower_block(ctx, &if_expr.then_block)?;
                ctx.set_terminator(Terminator::Branch(merge_block));

                // Lower else block
                ctx.current_block = else_block;
                let else_value = if let Some(ref else_expr) = if_expr.else_block {
                    self.lower_expr(ctx, else_expr)?
                } else {
                    Value::Constant(Constant::Unit)
                };
                ctx.set_terminator(Terminator::Branch(merge_block));

                // Merge block with phi
                ctx.current_block = merge_block;

                if let (Some(then_val), else_val) = (then_value, else_value) {
                    let result = ctx.new_register();
                    ctx.add_instruction(Instruction::Phi {
                        result,
                        incoming: vec![(then_val, then_block), (else_val, else_block)],
                        ty: IrType::I32, // Placeholder
                    });
                    Ok(Value::Register(result))
                } else {
                    Ok(Value::Constant(Constant::Unit))
                }
            }

            ExprKind::While(while_expr) => {
                let cond_block = ctx.new_block();
                let body_block = ctx.new_block();
                let exit_block = ctx.new_block();

                // Branch to condition block
                ctx.set_terminator(Terminator::Branch(cond_block));

                // Condition block
                ctx.current_block = cond_block;
                let cond = self.lower_expr(ctx, &while_expr.cond)?;
                ctx.set_terminator(Terminator::CondBranch {
                    cond,
                    true_block: body_block,
                    false_block: exit_block,
                });

                // Body block
                ctx.current_block = body_block;
                ctx.enter_loop(cond_block, exit_block);
                self.lower_block(ctx, &while_expr.body)?;
                ctx.exit_loop();
                ctx.set_terminator(Terminator::Branch(cond_block));

                // Exit block
                ctx.current_block = exit_block;

                Ok(Value::Constant(Constant::Unit))
            }

            ExprKind::Loop(loop_expr) => {
                let body_block = ctx.new_block();
                let exit_block = ctx.new_block();

                // Branch to body
                ctx.set_terminator(Terminator::Branch(body_block));

                // Body block
                ctx.current_block = body_block;
                ctx.enter_loop(body_block, exit_block);
                self.lower_block(ctx, &loop_expr.body)?;
                ctx.exit_loop();
                ctx.set_terminator(Terminator::Branch(body_block));

                // Exit block (unreachable unless break)
                ctx.current_block = exit_block;

                Ok(Value::Constant(Constant::Unit))
            }

            ExprKind::Return(opt_expr) => {
                let value = if let Some(ref expr) = opt_expr {
                    Some(self.lower_expr(ctx, expr)?)
                } else {
                    None
                };
                ctx.set_terminator(Terminator::Return(value));
                Ok(Value::Constant(Constant::Unit))
            }

            ExprKind::Break(_opt_expr) => {
                if let Some((_, break_block)) = ctx.current_loop() {
                    ctx.set_terminator(Terminator::Branch(break_block));
                }
                Ok(Value::Constant(Constant::Unit))
            }

            ExprKind::Continue => {
                if let Some((continue_block, _)) = ctx.current_loop() {
                    ctx.set_terminator(Terminator::Branch(continue_block));
                }
                Ok(Value::Constant(Constant::Unit))
            }

            _ => Err(LowerError::UnsupportedExpr(format!("{:?}", expr.kind))),
        }
    }

    /// Consume the lowerer and return the IR module
    pub fn into_module(self) -> Module {
        self.module
    }
}

/// Convert AST type to IR type
fn convert_type(ty: &Type) -> LowerResult<IrType> {
    match ty {
        Type::Primitive(prim) => match prim {
            PrimitiveType::I8 => Ok(IrType::I8),
            PrimitiveType::I16 => Ok(IrType::I16),
            PrimitiveType::I32 => Ok(IrType::I32),
            PrimitiveType::I64 => Ok(IrType::I64),
            PrimitiveType::U8 => Ok(IrType::U8),
            PrimitiveType::U16 => Ok(IrType::U16),
            PrimitiveType::U32 => Ok(IrType::U32),
            PrimitiveType::U64 => Ok(IrType::U64),
            PrimitiveType::F32 => Ok(IrType::F32),
            PrimitiveType::F64 => Ok(IrType::F64),
            PrimitiveType::Bool => Ok(IrType::Bool),
            PrimitiveType::Unit => Ok(IrType::Unit),
            _ => Err(LowerError::UnsupportedType(format!("{:?}", prim))),
        },
        Type::Array(elem, size) => {
            let elem_ty = convert_type(elem)?;
            Ok(IrType::Array(Box::new(elem_ty), *size))
        }
        Type::Pointer(_inner, _) => {
            // Simplified: just use Ptr type
            Ok(IrType::Ptr)
        }
        Type::Reference(_inner, _) => {
            // Simplified: just use Ptr type
            Ok(IrType::Ptr)
        }
        _ => Err(LowerError::UnsupportedType(format!("{:?}", ty))),
    }
}

/// Convert AST binary operator to IR binary operator
fn convert_binary_op(op: &AstBinaryOp) -> BinaryOp {
    match op {
        AstBinaryOp::Add => BinaryOp::Add,
        AstBinaryOp::Sub => BinaryOp::Sub,
        AstBinaryOp::Mul => BinaryOp::Mul,
        AstBinaryOp::Div => BinaryOp::Div,
        AstBinaryOp::Rem => BinaryOp::Rem,
        AstBinaryOp::BitAnd => BinaryOp::And,
        AstBinaryOp::BitOr => BinaryOp::Or,
        AstBinaryOp::BitXor => BinaryOp::Xor,
        AstBinaryOp::Shl => BinaryOp::Shl,
        AstBinaryOp::Shr => BinaryOp::Shr,
        AstBinaryOp::Eq => BinaryOp::Eq,
        AstBinaryOp::Ne => BinaryOp::Ne,
        AstBinaryOp::Lt => BinaryOp::Lt,
        AstBinaryOp::Le => BinaryOp::Le,
        AstBinaryOp::Gt => BinaryOp::Gt,
        AstBinaryOp::Ge => BinaryOp::Ge,
        // Logical operators treated as bitwise for now
        AstBinaryOp::And => BinaryOp::And,
        AstBinaryOp::Or => BinaryOp::Or,
        // Default cases
        _ => BinaryOp::Add,
    }
}

/// Convert AST unary operator to IR unary operator
fn convert_unary_op(op: &AstUnaryOp) -> UnaryOp {
    match op {
        AstUnaryOp::Neg => UnaryOp::Neg,
        AstUnaryOp::Not => UnaryOp::Not,
        AstUnaryOp::BitNot => UnaryOp::BitNot,
    }
}

/// Lower a literal to a constant
fn lower_literal(lit: &Lit) -> LowerResult<Constant> {
    match lit {
        Lit::Bool(b) => Ok(Constant::Bool(b.value())),
        Lit::Int(i) => {
            // Parse the integer (simplified)
            let value: i32 = i.raw.parse().unwrap_or(0);
            Ok(Constant::I32(value))
        }
        Lit::Float(f) => {
            // Parse the float (simplified)
            let value: f64 = f.raw.parse().unwrap_or(0.0);
            Ok(Constant::F64(value))
        }
        Lit::Str(s) => Ok(Constant::String(s.value.clone())),
        Lit::Char(c) => Ok(Constant::U32(c.value as u32)),
        _ => Err(LowerError::InvalidConstant(format!("{:?}", lit))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_type() {
        let ty = Type::primitive(PrimitiveType::I32);
        let ir_ty = convert_type(&ty).unwrap();
        assert_eq!(ir_ty, IrType::I32);
    }

    #[test]
    fn test_convert_binary_op() {
        let op = convert_binary_op(&AstBinaryOp::Add);
        assert_eq!(op, BinaryOp::Add);
    }

    #[test]
    fn test_convert_unary_op() {
        let op = convert_unary_op(&AstUnaryOp::Neg);
        assert_eq!(op, UnaryOp::Neg);
    }

    #[test]
    fn test_lower_literal_int() {
        use blang_ast::{IntBase, IntLit};
        let lit = Lit::Int(IntLit {
            raw: "42".to_string(),
            base: IntBase::Decimal,
            suffix: None,
        });
        let constant = lower_literal(&lit).unwrap();
        assert_eq!(constant, Constant::I32(42));
    }

    #[test]
    fn test_lower_literal_bool() {
        use blang_ast::BoolLit;
        let lit = Lit::Bool(BoolLit::True);
        let constant = lower_literal(&lit).unwrap();
        assert_eq!(constant, Constant::Bool(true));
    }
}

//! Blang Intermediate Representation (IR)
//!
//! This crate provides an SSA-like intermediate representation for the Blang compiler,
//! suitable for optimization and code generation to WebAssembly and other targets.
//!
//! # Modules
//!
//! - `ir`: Core IR data structures (Module, Function, BasicBlock, Instruction)
//! - `lower`: AST to IR lowering pass
//! - `opt`: Optimization passes (constant folding, dead code elimination)
//!
//! # Example
//!
//! ```ignore
//! use blang_ir::{IrLowerer, optimize_module};
//!
//! // Create lowerer
//! let mut lowerer = IrLowerer::new("my_module".to_string());
//!
//! // Lower AST items to IR
//! lowerer.lower_items(&items)?;
//!
//! // Get the IR module
//! let mut module = lowerer.into_module();
//!
//! // Optimize
//! optimize_module(&mut module);
//!
//! // Validate
//! validate_module(&module)?;
//! ```

pub mod ir;
pub mod lower;
pub mod opt;

pub use ir::*;
pub use lower::{IrLowerer, LowerError, LowerResult};
pub use opt::{constant_fold, eliminate_dead_code, optimize_function, optimize_module};

use rustc_hash::FxHashSet;

/// Validation error
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Block {0} has no terminator")]
    MissingTerminator(BlockId),
    #[error("Block {0} not found")]
    BlockNotFound(BlockId),
    #[error("Unreachable block {0}")]
    UnreachableBlock(BlockId),
    #[error("Undefined register {0}")]
    UndefinedRegister(Register),
    #[error("Multiple definitions of register {0}")]
    MultipleDefinitions(Register),
    #[error("Empty function")]
    EmptyFunction,
}

pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validate an IR module
pub fn validate_module(module: &Module) -> ValidationResult<()> {
    for func in module.functions.values() {
        validate_function(func)?;
    }
    Ok(())
}

/// Validate an IR function
pub fn validate_function(func: &Function) -> ValidationResult<()> {
    if func.blocks.is_empty() {
        return Err(ValidationError::EmptyFunction);
    }

    // Check that all blocks have terminators
    for block in &func.blocks {
        if !block.is_terminated() {
            return Err(ValidationError::MissingTerminator(block.id));
        }
    }

    // Check that all blocks are reachable from entry
    let reachable = find_reachable_blocks_for_validation(func);
    for block in &func.blocks {
        if !reachable.contains(&block.id) && block.id.0 > 0 {
            // Entry block (bb0) is always considered reachable
            return Err(ValidationError::UnreachableBlock(block.id));
        }
    }

    // Check that all referenced blocks exist
    for block in &func.blocks {
        if let Some(ref term) = block.terminator {
            check_terminator_blocks(term, func)?;
        }

        // Check phi nodes
        for inst in &block.instructions {
            if let Instruction::Phi { incoming, .. } = inst {
                for (_, block_id) in incoming {
                    if func.get_block(*block_id).is_none() {
                        return Err(ValidationError::BlockNotFound(*block_id));
                    }
                }
            }
        }
    }

    // Check SSA properties: each register is defined exactly once
    let mut defined_registers = FxHashSet::default();
    for param in &func.params {
        if !defined_registers.insert(param.register) {
            return Err(ValidationError::MultipleDefinitions(param.register));
        }
    }

    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(def) = get_defined_register(inst) {
                if !defined_registers.insert(def) {
                    return Err(ValidationError::MultipleDefinitions(def));
                }
            }
        }
    }

    // Check that all used registers are defined
    for block in &func.blocks {
        for inst in &block.instructions {
            check_instruction_uses(inst, &defined_registers)?;
        }

        if let Some(ref term) = block.terminator {
            check_terminator_uses(term, &defined_registers)?;
        }
    }

    Ok(())
}

/// Find reachable blocks for validation
fn find_reachable_blocks_for_validation(func: &Function) -> FxHashSet<BlockId> {
    let mut reachable = FxHashSet::default();
    let mut worklist = Vec::new();

    if let Some(entry) = func.entry_block() {
        worklist.push(entry.id);
        reachable.insert(entry.id);
    }

    while let Some(block_id) = worklist.pop() {
        if let Some(block) = func.get_block(block_id) {
            if let Some(ref term) = block.terminator {
                for successor in get_terminator_successors(term) {
                    if reachable.insert(successor) {
                        worklist.push(successor);
                    }
                }
            }
        }
    }

    reachable
}

/// Get successor blocks from a terminator
fn get_terminator_successors(term: &Terminator) -> Vec<BlockId> {
    match term {
        Terminator::Return(_) => vec![],
        Terminator::Branch(block) => vec![*block],
        Terminator::CondBranch {
            true_block,
            false_block,
            ..
        } => vec![*true_block, *false_block],
        Terminator::Unreachable => vec![],
    }
}

/// Check that all blocks referenced in a terminator exist
fn check_terminator_blocks(term: &Terminator, func: &Function) -> ValidationResult<()> {
    for block_id in get_terminator_successors(term) {
        if func.get_block(block_id).is_none() {
            return Err(ValidationError::BlockNotFound(block_id));
        }
    }
    Ok(())
}

/// Get the register defined by an instruction
fn get_defined_register(inst: &Instruction) -> Option<Register> {
    match inst {
        Instruction::Binary { result, .. }
        | Instruction::Unary { result, .. }
        | Instruction::Load { result, .. }
        | Instruction::Alloca { result, .. }
        | Instruction::GetElementPtr { result, .. }
        | Instruction::Cast { result, .. }
        | Instruction::Phi { result, .. }
        | Instruction::Copy { result, .. } => Some(*result),
        Instruction::Call {
            result: Some(r), ..
        } => Some(*r),
        _ => None,
    }
}

/// Check that all registers used in an instruction are defined
fn check_instruction_uses(
    inst: &Instruction,
    defined: &FxHashSet<Register>,
) -> ValidationResult<()> {
    match inst {
        Instruction::Binary { lhs, rhs, .. } => {
            check_value_use(lhs, defined)?;
            check_value_use(rhs, defined)?;
        }
        Instruction::Unary { operand, .. } => {
            check_value_use(operand, defined)?;
        }
        Instruction::Load { addr, .. } => {
            check_value_use(addr, defined)?;
        }
        Instruction::Store { addr, value, .. } => {
            check_value_use(addr, defined)?;
            check_value_use(value, defined)?;
        }
        Instruction::GetElementPtr { base, offset, .. } => {
            check_value_use(base, defined)?;
            check_value_use(offset, defined)?;
        }
        Instruction::Call { args, .. } => {
            for arg in args {
                check_value_use(arg, defined)?;
            }
        }
        Instruction::Cast { value, .. } => {
            check_value_use(value, defined)?;
        }
        Instruction::Phi { incoming, .. } => {
            for (val, _) in incoming {
                check_value_use(val, defined)?;
            }
        }
        Instruction::Copy { value, .. } => {
            check_value_use(value, defined)?;
        }
        _ => {}
    }
    Ok(())
}

/// Check that all registers used in a terminator are defined
fn check_terminator_uses(
    term: &Terminator,
    defined: &FxHashSet<Register>,
) -> ValidationResult<()> {
    match term {
        Terminator::Return(Some(val)) => {
            check_value_use(val, defined)?;
        }
        Terminator::CondBranch { cond, .. } => {
            check_value_use(cond, defined)?;
        }
        _ => {}
    }
    Ok(())
}

/// Check that a value's register (if any) is defined
fn check_value_use(value: &Value, defined: &FxHashSet<Register>) -> ValidationResult<()> {
    if let Value::Register(reg) = value {
        if !defined.contains(reg) {
            return Err(ValidationError::UndefinedRegister(*reg));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use blang_ast::*;

    #[test]
    fn test_empty_module() {
        let module = Module::new("test".to_string());
        assert_eq!(module.name, "test");
        assert!(module.functions.is_empty());
        assert!(module.globals.is_empty());
    }

    #[test]
    fn test_function_creation() {
        let func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::Unit,
        );
        assert_eq!(func.id.0, "test");
        assert_eq!(func.return_type, IrType::Unit);
    }

    #[test]
    fn test_validate_empty_function() {
        let func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::Unit,
        );
        let result = validate_function(&func);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_terminator() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::Unit,
        );
        func.new_block();

        let result = validate_function(&func);
        assert!(matches!(
            result,
            Err(ValidationError::MissingTerminator(_))
        ));
    }

    #[test]
    fn test_validate_simple_function() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::Unit,
        );

        let entry = func.new_block();
        if let Some(block) = func.get_block_mut(entry) {
            block.terminator = Some(Terminator::Return(None));
        }

        let result = validate_function(&func);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_undefined_register() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::I32,
        );

        let entry = func.new_block();
        let r0 = Register(999); // Undefined register

        if let Some(block) = func.get_block_mut(entry) {
            block.terminator = Some(Terminator::Return(Some(Value::Register(r0))));
        }

        let result = validate_function(&func);
        assert!(matches!(
            result,
            Err(ValidationError::UndefinedRegister(_))
        ));
    }

    #[test]
    fn test_ir_lowerer_creation() {
        let lowerer = IrLowerer::new("test_module".to_string());
        let module = lowerer.into_module();
        assert_eq!(module.name, "test_module");
    }

    /// Integration test: lower a simple function and validate
    #[test]
    fn test_lower_simple_function() {
        use blang_span::Span;

        // Create a simple function: fn test() -> i32 { 42 }
        let func = FunctionDecl {
            attrs: vec![],
            sig: FunctionSig {
                public: false,
                unsafe_: false,
                name: Ident::new("test".to_string(), Span::DUMMY),
                generic_params: vec![],
                params: vec![],
                return_ty: Some(Ty {
                    kind: TyKind::Primitive(PrimitiveTy::I32),
                    span: Span::DUMMY,
                }),
            },
            body: Some(Block {
                stmts: vec![Stmt {
                    kind: StmtKind::Expr(
                        Expr {
                            kind: ExprKind::Literal(Lit::Int(IntLit {
                                raw: "42".to_string(),
                                base: IntBase::Decimal,
                                suffix: None,
                            })),
                            span: Span::DUMMY,
                        },
                        false, // no semicolon
                    ),
                    span: Span::DUMMY,
                }],
                expr: None,
                span: Span::DUMMY,
            }),
        };

        let item = Item {
            kind: ItemKind::Function(func),
            span: Span::DUMMY,
        };

        let mut lowerer = IrLowerer::new("test_module".to_string());
        let result = lowerer.lower_items(&[item]);
        assert!(result.is_ok());

        let module = lowerer.into_module();
        assert!(!module.functions.is_empty());

        // The function should have at least an entry block
        let test_func = module.get_function(&FunctionId("test".to_string()));
        assert!(test_func.is_some());
    }

    /// Integration test: constant folding
    #[test]
    fn test_constant_folding_integration() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::I32,
        );

        let entry = func.new_block();
        let r0 = func.new_register();

        if let Some(block) = func.get_block_mut(entry) {
            // r0 = 2 + 3
            block.instructions.push(Instruction::Binary {
                result: r0,
                op: crate::ir::BinaryOp::Add,
                lhs: Value::Constant(Constant::I32(2)),
                rhs: Value::Constant(Constant::I32(3)),
                ty: IrType::I32,
            });

            block.terminator = Some(Terminator::Return(Some(Value::Register(r0))));
        }

        // Before optimization: 1 binary instruction
        assert_eq!(func.get_block(entry).unwrap().instructions.len(), 1);

        // Run constant folding
        constant_fold(&mut func);

        // After optimization: should have a copy instruction instead
        let block = func.get_block(entry).unwrap();
        assert!(matches!(
            block.instructions[0],
            Instruction::Copy { .. }
        ));
    }

    /// Integration test: dead code elimination
    #[test]
    fn test_dead_code_elimination_integration() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::I32,
        );

        let entry = func.new_block();
        let r0 = func.new_register();
        let r1 = func.new_register();

        if let Some(block) = func.get_block_mut(entry) {
            // r0 = 10 (used)
            block.instructions.push(Instruction::Copy {
                result: r0,
                value: Value::Constant(Constant::I32(10)),
                ty: IrType::I32,
            });

            // r1 = 20 (unused)
            block.instructions.push(Instruction::Copy {
                result: r1,
                value: Value::Constant(Constant::I32(20)),
                ty: IrType::I32,
            });

            block.terminator = Some(Terminator::Return(Some(Value::Register(r0))));
        }

        // Before DCE: 2 instructions
        assert_eq!(func.get_block(entry).unwrap().instructions.len(), 2);

        // Run DCE
        eliminate_dead_code(&mut func);

        // After DCE: should have only 1 instruction (r1 is unused)
        assert_eq!(func.get_block(entry).unwrap().instructions.len(), 1);
    }
}

//! IR Optimizations
//!
//! This module provides basic optimizations for the IR:
//! - Constant folding
//! - Dead code elimination

use crate::ir::*;
use rustc_hash::FxHashSet;

/// Optimize a module
pub fn optimize_module(module: &mut Module) {
    for func in module.functions.values_mut() {
        optimize_function(func);
    }
}

/// Optimize a function
pub fn optimize_function(func: &mut Function) {
    // Run multiple optimization passes
    let mut changed = true;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 10;

    while changed && iterations < MAX_ITERATIONS {
        changed = false;
        changed |= constant_fold(func);
        changed |= eliminate_dead_code(func);
        iterations += 1;
    }
}

/// Constant folding optimization
///
/// Evaluates binary and unary operations on constants at compile time.
pub fn constant_fold(func: &mut Function) -> bool {
    let mut changed = false;

    for block in &mut func.blocks {
        let mut new_instructions = Vec::new();
        let mut folded_values: rustc_hash::FxHashMap<Register, Constant> =
            rustc_hash::FxHashMap::default();

        for inst in &block.instructions {
            match inst {
                Instruction::Binary {
                    result,
                    op,
                    lhs,
                    rhs,
                    ty,
                } => {
                    // Try to fold if both operands are constants
                    if let (Some(lhs_const), Some(rhs_const)) =
                        (get_constant(lhs, &folded_values), get_constant(rhs, &folded_values))
                    {
                        if let Some(folded) = fold_binary_op(*op, lhs_const, rhs_const) {
                            folded_values.insert(*result, folded.clone());
                            // Replace with a copy of the constant
                            new_instructions.push(Instruction::Copy {
                                result: *result,
                                value: Value::Constant(folded),
                                ty: ty.clone(),
                            });
                            changed = true;
                            continue;
                        }
                    }
                    new_instructions.push(inst.clone());
                }

                Instruction::Unary {
                    result,
                    op,
                    operand,
                    ty,
                } => {
                    // Try to fold if operand is constant
                    if let Some(operand_const) = get_constant(operand, &folded_values) {
                        if let Some(folded) = fold_unary_op(*op, operand_const) {
                            folded_values.insert(*result, folded.clone());
                            new_instructions.push(Instruction::Copy {
                                result: *result,
                                value: Value::Constant(folded),
                                ty: ty.clone(),
                            });
                            changed = true;
                            continue;
                        }
                    }
                    new_instructions.push(inst.clone());
                }

                Instruction::Copy { result, value, .. } => {
                    // Track constant copies
                    if let Value::Constant(c) = value {
                        folded_values.insert(*result, c.clone());
                    }
                    new_instructions.push(inst.clone());
                }

                _ => {
                    new_instructions.push(inst.clone());
                }
            }
        }

        if changed {
            block.instructions = new_instructions;
        }
    }

    changed
}

/// Get a constant value from a Value, checking the folded values map
fn get_constant<'a>(value: &'a Value, folded: &'a rustc_hash::FxHashMap<Register, Constant>) -> Option<&'a Constant> {
    match value {
        Value::Constant(c) => Some(c),
        Value::Register(r) => folded.get(r),
    }
}

/// Fold a binary operation on constants
fn fold_binary_op(op: BinaryOp, lhs: &Constant, rhs: &Constant) -> Option<Constant> {
    match (lhs, rhs) {
        (Constant::I32(a), Constant::I32(b)) => {
            let result = match op {
                BinaryOp::Add => a.checked_add(*b)?,
                BinaryOp::Sub => a.checked_sub(*b)?,
                BinaryOp::Mul => a.checked_mul(*b)?,
                BinaryOp::Div => a.checked_div(*b)?,
                BinaryOp::Rem => a.checked_rem(*b)?,
                BinaryOp::And => *a & *b,
                BinaryOp::Or => *a | *b,
                BinaryOp::Xor => *a ^ *b,
                BinaryOp::Shl => a.checked_shl(*b as u32)?,
                BinaryOp::Shr => a.checked_shr(*b as u32)?,
                BinaryOp::Eq => return Some(Constant::Bool(*a == *b)),
                BinaryOp::Ne => return Some(Constant::Bool(*a != *b)),
                BinaryOp::Lt => return Some(Constant::Bool(*a < *b)),
                BinaryOp::Le => return Some(Constant::Bool(*a <= *b)),
                BinaryOp::Gt => return Some(Constant::Bool(*a > *b)),
                BinaryOp::Ge => return Some(Constant::Bool(*a >= *b)),
            };
            Some(Constant::I32(result))
        }

        (Constant::I64(a), Constant::I64(b)) => {
            let result = match op {
                BinaryOp::Add => a.checked_add(*b)?,
                BinaryOp::Sub => a.checked_sub(*b)?,
                BinaryOp::Mul => a.checked_mul(*b)?,
                BinaryOp::Div => a.checked_div(*b)?,
                BinaryOp::Rem => a.checked_rem(*b)?,
                BinaryOp::And => *a & *b,
                BinaryOp::Or => *a | *b,
                BinaryOp::Xor => *a ^ *b,
                BinaryOp::Shl => a.checked_shl(*b as u32)?,
                BinaryOp::Shr => a.checked_shr(*b as u32)?,
                BinaryOp::Eq => return Some(Constant::Bool(*a == *b)),
                BinaryOp::Ne => return Some(Constant::Bool(*a != *b)),
                BinaryOp::Lt => return Some(Constant::Bool(*a < *b)),
                BinaryOp::Le => return Some(Constant::Bool(*a <= *b)),
                BinaryOp::Gt => return Some(Constant::Bool(*a > *b)),
                BinaryOp::Ge => return Some(Constant::Bool(*a >= *b)),
            };
            Some(Constant::I64(result))
        }

        (Constant::Bool(a), Constant::Bool(b)) => {
            let result = match op {
                BinaryOp::And => *a && *b,
                BinaryOp::Or => *a || *b,
                BinaryOp::Xor => *a ^ *b,
                BinaryOp::Eq => *a == *b,
                BinaryOp::Ne => *a != *b,
                _ => return None,
            };
            Some(Constant::Bool(result))
        }

        (Constant::F64(a), Constant::F64(b)) => {
            let result = match op {
                BinaryOp::Add => a + b,
                BinaryOp::Sub => a - b,
                BinaryOp::Mul => a * b,
                BinaryOp::Div => a / b,
                BinaryOp::Eq => return Some(Constant::Bool(*a == *b)),
                BinaryOp::Ne => return Some(Constant::Bool(*a != *b)),
                BinaryOp::Lt => return Some(Constant::Bool(*a < *b)),
                BinaryOp::Le => return Some(Constant::Bool(*a <= *b)),
                BinaryOp::Gt => return Some(Constant::Bool(*a > *b)),
                BinaryOp::Ge => return Some(Constant::Bool(*a >= *b)),
                _ => return None,
            };
            Some(Constant::F64(result))
        }

        _ => None,
    }
}

/// Fold a unary operation on a constant
fn fold_unary_op(op: UnaryOp, operand: &Constant) -> Option<Constant> {
    match operand {
        Constant::I32(n) => {
            let result = match op {
                UnaryOp::Neg => n.checked_neg()?,
                UnaryOp::Not => return None, // Not is for booleans
                UnaryOp::BitNot => !*n,
            };
            Some(Constant::I32(result))
        }

        Constant::I64(n) => {
            let result = match op {
                UnaryOp::Neg => n.checked_neg()?,
                UnaryOp::Not => return None,
                UnaryOp::BitNot => !*n,
            };
            Some(Constant::I64(result))
        }

        Constant::Bool(b) => {
            if matches!(op, UnaryOp::Not) {
                Some(Constant::Bool(!*b))
            } else {
                None
            }
        }

        Constant::F64(f) => {
            if matches!(op, UnaryOp::Neg) {
                Some(Constant::F64(-*f))
            } else {
                None
            }
        }

        _ => None,
    }
}

/// Dead code elimination
///
/// Removes unreachable basic blocks and unused instructions.
pub fn eliminate_dead_code(func: &mut Function) -> bool {
    let mut changed = false;

    // Step 1: Find reachable blocks
    let reachable = find_reachable_blocks(func);

    // Step 2: Remove unreachable blocks
    let original_count = func.blocks.len();
    func.blocks.retain(|block| reachable.contains(&block.id));
    changed |= func.blocks.len() < original_count;

    // Step 3: Remove unused instructions within reachable blocks
    for block in &mut func.blocks {
        changed |= eliminate_dead_instructions(block);
    }

    changed
}

/// Find all reachable blocks from the entry block
fn find_reachable_blocks(func: &Function) -> FxHashSet<BlockId> {
    let mut reachable = FxHashSet::default();
    let mut worklist = Vec::new();

    // Start from entry block
    if let Some(entry) = func.entry_block() {
        worklist.push(entry.id);
        reachable.insert(entry.id);
    }

    while let Some(block_id) = worklist.pop() {
        if let Some(block) = func.get_block(block_id) {
            // Add successors to worklist
            if let Some(ref term) = block.terminator {
                for successor in get_successors(term) {
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
fn get_successors(term: &Terminator) -> Vec<BlockId> {
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

/// Eliminate dead instructions within a block
fn eliminate_dead_instructions(block: &mut BasicBlock) -> bool {
    // Simple approach: mark all registers that are used
    let mut used_registers = FxHashSet::default();

    // Collect uses from terminator
    if let Some(ref term) = block.terminator {
        collect_terminator_uses(term, &mut used_registers);
    }

    // Collect uses from instructions (backward pass)
    for inst in block.instructions.iter().rev() {
        collect_instruction_uses(inst, &mut used_registers);
    }

    // Remove instructions that define unused registers
    let original_count = block.instructions.len();
    block.instructions.retain(|inst| {
        if let Some(def) = get_instruction_def(inst) {
            used_registers.contains(&def)
        } else {
            // Keep instructions with side effects (store, call without result, etc.)
            has_side_effects(inst)
        }
    });

    block.instructions.len() < original_count
}

/// Get the register defined by an instruction
fn get_instruction_def(inst: &Instruction) -> Option<Register> {
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

/// Check if an instruction has side effects
fn has_side_effects(inst: &Instruction) -> bool {
    matches!(
        inst,
        Instruction::Store { .. } | Instruction::Call { .. }
    )
}

/// Collect registers used by an instruction
fn collect_instruction_uses(inst: &Instruction, used: &mut FxHashSet<Register>) {
    match inst {
        Instruction::Binary { lhs, rhs, .. } => {
            add_value_use(lhs, used);
            add_value_use(rhs, used);
        }
        Instruction::Unary { operand, .. } => {
            add_value_use(operand, used);
        }
        Instruction::Load { addr, .. } => {
            add_value_use(addr, used);
        }
        Instruction::Store { addr, value, .. } => {
            add_value_use(addr, used);
            add_value_use(value, used);
        }
        Instruction::GetElementPtr { base, offset, .. } => {
            add_value_use(base, used);
            add_value_use(offset, used);
        }
        Instruction::Call { args, .. } => {
            for arg in args {
                add_value_use(arg, used);
            }
        }
        Instruction::Cast { value, .. } => {
            add_value_use(value, used);
        }
        Instruction::Phi { incoming, .. } => {
            for (val, _) in incoming {
                add_value_use(val, used);
            }
        }
        Instruction::Copy { value, .. } => {
            add_value_use(value, used);
        }
        _ => {}
    }
}

/// Collect registers used by a terminator
fn collect_terminator_uses(term: &Terminator, used: &mut FxHashSet<Register>) {
    match term {
        Terminator::Return(Some(val)) => {
            add_value_use(val, used);
        }
        Terminator::CondBranch { cond, .. } => {
            add_value_use(cond, used);
        }
        _ => {}
    }
}

/// Add a register to the used set if the value is a register
fn add_value_use(value: &Value, used: &mut FxHashSet<Register>) {
    if let Value::Register(reg) = value {
        used.insert(*reg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_binary_add() {
        let result = fold_binary_op(
            BinaryOp::Add,
            &Constant::I32(10),
            &Constant::I32(32),
        );
        assert_eq!(result, Some(Constant::I32(42)));
    }

    #[test]
    fn test_fold_binary_comparison() {
        let result = fold_binary_op(
            BinaryOp::Lt,
            &Constant::I32(10),
            &Constant::I32(20),
        );
        assert_eq!(result, Some(Constant::Bool(true)));
    }

    #[test]
    fn test_fold_unary_neg() {
        let result = fold_unary_op(UnaryOp::Neg, &Constant::I32(42));
        assert_eq!(result, Some(Constant::I32(-42)));
    }

    #[test]
    fn test_fold_unary_not() {
        let result = fold_unary_op(UnaryOp::Not, &Constant::Bool(true));
        assert_eq!(result, Some(Constant::Bool(false)));
    }

    #[test]
    fn test_constant_fold_chain() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::I32,
        );

        let entry = func.new_block();
        let r0 = func.new_register();
        let r1 = func.new_register();

        if let Some(block) = func.get_block_mut(entry) {
            // r0 = 10 + 5
            block.instructions.push(Instruction::Binary {
                result: r0,
                op: BinaryOp::Add,
                lhs: Value::Constant(Constant::I32(10)),
                rhs: Value::Constant(Constant::I32(5)),
                ty: IrType::I32,
            });

            // r1 = r0 * 2
            block.instructions.push(Instruction::Binary {
                result: r1,
                op: BinaryOp::Mul,
                lhs: Value::Register(r0),
                rhs: Value::Constant(Constant::I32(2)),
                ty: IrType::I32,
            });
        }

        let changed = constant_fold(&mut func);
        assert!(changed);

        // After first pass, r0 should be folded to 15
        // After second pass, r1 should be folded to 30
        constant_fold(&mut func);
    }
}

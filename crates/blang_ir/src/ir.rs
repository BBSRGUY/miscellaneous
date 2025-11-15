//! Blang Intermediate Representation (IR)
//!
//! This module defines an SSA-like intermediate representation suitable for
//! code generation to WebAssembly and other targets.

use rustc_hash::FxHashMap;
use std::fmt;

/// A unique identifier for a virtual register (SSA value)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register(pub u32);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// A unique identifier for a basic block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// A unique identifier for a function
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId(pub String);

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// IR Type system (simplified from AST types)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IrType {
    /// Void/unit type
    Unit,
    /// Boolean
    Bool,
    /// 8-bit signed integer
    I8,
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer
    U64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// Pointer type
    Ptr,
    /// Array type with element type and size
    Array(Box<IrType>, usize),
    /// Struct type with fields
    Struct(Vec<IrType>),
    /// Function type
    Function(Vec<IrType>, Box<IrType>),
}

impl IrType {
    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            IrType::I8
                | IrType::I16
                | IrType::I32
                | IrType::I64
                | IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::U64
                | IrType::F32
                | IrType::F64
        )
    }

    /// Check if this type is an integer
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            IrType::I8
                | IrType::I16
                | IrType::I32
                | IrType::I64
                | IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::U64
        )
    }

    /// Check if this type is a float
    pub fn is_float(&self) -> bool {
        matches!(self, IrType::F32 | IrType::F64)
    }

    /// Get the size in bytes (simplified, assumes 64-bit target)
    pub fn size_bytes(&self) -> usize {
        match self {
            IrType::Unit => 0,
            IrType::Bool | IrType::I8 | IrType::U8 => 1,
            IrType::I16 | IrType::U16 => 2,
            IrType::I32 | IrType::U32 | IrType::F32 => 4,
            IrType::I64 | IrType::U64 | IrType::F64 | IrType::Ptr => 8,
            IrType::Array(elem_ty, size) => elem_ty.size_bytes() * size,
            IrType::Struct(fields) => fields.iter().map(|f| f.size_bytes()).sum(),
            IrType::Function(_, _) => 8, // Function pointer
        }
    }
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrType::Unit => write!(f, "unit"),
            IrType::Bool => write!(f, "bool"),
            IrType::I8 => write!(f, "i8"),
            IrType::I16 => write!(f, "i16"),
            IrType::I32 => write!(f, "i32"),
            IrType::I64 => write!(f, "i64"),
            IrType::U8 => write!(f, "u8"),
            IrType::U16 => write!(f, "u16"),
            IrType::U32 => write!(f, "u32"),
            IrType::U64 => write!(f, "u64"),
            IrType::F32 => write!(f, "f32"),
            IrType::F64 => write!(f, "f64"),
            IrType::Ptr => write!(f, "ptr"),
            IrType::Array(elem, size) => write!(f, "[{}; {}]", elem, size),
            IrType::Struct(fields) => {
                write!(f, "{{")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", field)?;
                }
                write!(f, "}}")
            }
            IrType::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
        }
    }
}

/// A constant value in the IR
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Unit,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Unit => write!(f, "()"),
            Constant::Bool(b) => write!(f, "{}", b),
            Constant::I8(n) => write!(f, "{}i8", n),
            Constant::I16(n) => write!(f, "{}i16", n),
            Constant::I32(n) => write!(f, "{}", n),
            Constant::I64(n) => write!(f, "{}i64", n),
            Constant::U8(n) => write!(f, "{}u8", n),
            Constant::U16(n) => write!(f, "{}u16", n),
            Constant::U32(n) => write!(f, "{}u32", n),
            Constant::U64(n) => write!(f, "{}u64", n),
            Constant::F32(n) => write!(f, "{}f32", n),
            Constant::F64(n) => write!(f, "{}f64", n),
            Constant::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// A value in the IR (either a register or a constant)
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Register(Register),
    Constant(Constant),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Register(reg) => write!(f, "{}", reg),
            Value::Constant(c) => write!(f, "{}", c),
        }
    }
}

impl Value {
    pub fn is_constant(&self) -> bool {
        matches!(self, Value::Constant(_))
    }

    pub fn as_constant(&self) -> Option<&Constant> {
        match self {
            Value::Constant(c) => Some(c),
            _ => None,
        }
    }
}

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    // Bitwise
    And,
    Or,
    Xor,
    Shl,
    Shr,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "add"),
            BinaryOp::Sub => write!(f, "sub"),
            BinaryOp::Mul => write!(f, "mul"),
            BinaryOp::Div => write!(f, "div"),
            BinaryOp::Rem => write!(f, "rem"),
            BinaryOp::And => write!(f, "and"),
            BinaryOp::Or => write!(f, "or"),
            BinaryOp::Xor => write!(f, "xor"),
            BinaryOp::Shl => write!(f, "shl"),
            BinaryOp::Shr => write!(f, "shr"),
            BinaryOp::Eq => write!(f, "eq"),
            BinaryOp::Ne => write!(f, "ne"),
            BinaryOp::Lt => write!(f, "lt"),
            BinaryOp::Le => write!(f, "le"),
            BinaryOp::Gt => write!(f, "gt"),
            BinaryOp::Ge => write!(f, "ge"),
        }
    }
}

/// Unary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "neg"),
            UnaryOp::Not => write!(f, "not"),
            UnaryOp::BitNot => write!(f, "bitnot"),
        }
    }
}

/// IR Instructions
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Binary operation: result = lhs op rhs
    Binary {
        result: Register,
        op: BinaryOp,
        lhs: Value,
        rhs: Value,
        ty: IrType,
    },
    /// Unary operation: result = op operand
    Unary {
        result: Register,
        op: UnaryOp,
        operand: Value,
        ty: IrType,
    },
    /// Load from memory: result = *addr
    Load {
        result: Register,
        addr: Value,
        ty: IrType,
    },
    /// Store to memory: *addr = value
    Store {
        addr: Value,
        value: Value,
        ty: IrType,
    },
    /// Allocate stack memory: result = alloca(ty)
    Alloca {
        result: Register,
        ty: IrType,
    },
    /// Get element pointer: result = base + offset
    GetElementPtr {
        result: Register,
        base: Value,
        offset: Value,
        ty: IrType,
    },
    /// Function call: result = call func(args...)
    Call {
        result: Option<Register>,
        func: FunctionId,
        args: Vec<Value>,
        ret_ty: IrType,
    },
    /// Cast/conversion: result = cast value to ty
    Cast {
        result: Register,
        value: Value,
        from_ty: IrType,
        to_ty: IrType,
    },
    /// Phi node (for SSA): result = phi [val1, block1], [val2, block2], ...
    Phi {
        result: Register,
        incoming: Vec<(Value, BlockId)>,
        ty: IrType,
    },
    /// Copy/move: result = value
    Copy {
        result: Register,
        value: Value,
        ty: IrType,
    },
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Binary {
                result,
                op,
                lhs,
                rhs,
                ty,
            } => write!(f, "{} = {} {}, {} : {}", result, op, lhs, rhs, ty),
            Instruction::Unary {
                result,
                op,
                operand,
                ty,
            } => write!(f, "{} = {} {} : {}", result, op, operand, ty),
            Instruction::Load { result, addr, ty } => {
                write!(f, "{} = load {} : {}", result, addr, ty)
            }
            Instruction::Store { addr, value, ty } => {
                write!(f, "store {} <- {} : {}", addr, value, ty)
            }
            Instruction::Alloca { result, ty } => write!(f, "{} = alloca {}", result, ty),
            Instruction::GetElementPtr {
                result,
                base,
                offset,
                ty,
            } => write!(f, "{} = gep {}, {} : {}", result, base, offset, ty),
            Instruction::Call {
                result,
                func,
                args,
                ret_ty,
            } => {
                if let Some(res) = result {
                    write!(f, "{} = ", res)?;
                }
                write!(f, "call {}(", func)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") : {}", ret_ty)
            }
            Instruction::Cast {
                result,
                value,
                from_ty,
                to_ty,
            } => write!(f, "{} = cast {} from {} to {}", result, value, from_ty, to_ty),
            Instruction::Phi {
                result,
                incoming,
                ty,
            } => {
                write!(f, "{} = phi {} [", result, ty)?;
                for (i, (val, block)) in incoming.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} from {}", val, block)?;
                }
                write!(f, "]")
            }
            Instruction::Copy { result, value, ty } => {
                write!(f, "{} = copy {} : {}", result, value, ty)
            }
        }
    }
}

/// Block terminator (control flow)
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    /// Return from function
    Return(Option<Value>),
    /// Unconditional branch
    Branch(BlockId),
    /// Conditional branch: if cond then true_block else false_block
    CondBranch {
        cond: Value,
        true_block: BlockId,
        false_block: BlockId,
    },
    /// Unreachable (never returns)
    Unreachable,
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Return(Some(val)) => write!(f, "return {}", val),
            Terminator::Return(None) => write!(f, "return"),
            Terminator::Branch(block) => write!(f, "br {}", block),
            Terminator::CondBranch {
                cond,
                true_block,
                false_block,
            } => write!(f, "br {} ? {} : {}", cond, true_block, false_block),
            Terminator::Unreachable => write!(f, "unreachable"),
        }
    }
}

/// A basic block in the control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Option<Terminator>,
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator: None,
        }
    }

    pub fn is_terminated(&self) -> bool {
        self.terminator.is_some()
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", self.id)?;
        for inst in &self.instructions {
            writeln!(f, "  {}", inst)?;
        }
        if let Some(term) = &self.terminator {
            writeln!(f, "  {}", term)?;
        }
        Ok(())
    }
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: IrType,
    pub register: Register,
}

/// An IR function
#[derive(Debug, Clone)]
pub struct Function {
    pub id: FunctionId,
    pub params: Vec<Parameter>,
    pub return_type: IrType,
    pub blocks: Vec<BasicBlock>,
    pub next_register: u32,
    pub next_block: u32,
}

impl Function {
    pub fn new(id: FunctionId, params: Vec<Parameter>, return_type: IrType) -> Self {
        let next_register = params
            .iter()
            .map(|p| p.register.0 + 1)
            .max()
            .unwrap_or(0);
        Self {
            id,
            params,
            return_type,
            blocks: Vec::new(),
            next_register,
            next_block: 0,
        }
    }

    /// Allocate a new virtual register
    pub fn new_register(&mut self) -> Register {
        let reg = Register(self.next_register);
        self.next_register += 1;
        reg
    }

    /// Create a new basic block
    pub fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        self.blocks.push(BasicBlock::new(id));
        id
    }

    /// Get a mutable reference to a block
    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }

    /// Get a reference to a block
    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.id == id)
    }

    /// Get the entry block (first block)
    pub fn entry_block(&self) -> Option<&BasicBlock> {
        self.blocks.first()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}(", self.id)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", param.register, param.ty)?;
        }
        writeln!(f, ") -> {} {{", self.return_type)?;

        for block in &self.blocks {
            write!(f, "{}", block)?;
        }

        writeln!(f, "}}")
    }
}

/// Global variable
#[derive(Debug, Clone)]
pub struct Global {
    pub name: String,
    pub ty: IrType,
    pub init: Option<Constant>,
    pub mutable: bool,
}

/// An IR module (collection of functions and globals)
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: FxHashMap<FunctionId, Function>,
    pub globals: Vec<Global>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: FxHashMap::default(),
            globals: Vec::new(),
        }
    }

    /// Add a function to the module
    pub fn add_function(&mut self, func: Function) {
        self.functions.insert(func.id.clone(), func);
    }

    /// Add a global variable
    pub fn add_global(&mut self, global: Global) {
        self.globals.push(global);
    }

    /// Get a function by ID
    pub fn get_function(&self, id: &FunctionId) -> Option<&Function> {
        self.functions.get(id)
    }

    /// Get a mutable reference to a function
    pub fn get_function_mut(&mut self, id: &FunctionId) -> Option<&mut Function> {
        self.functions.get_mut(id)
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "module {} {{", self.name)?;

        for global in &self.globals {
            write!(f, "  global ")?;
            if global.mutable {
                write!(f, "mut ")?;
            }
            write!(f, "{}: {}", global.name, global.ty)?;
            if let Some(init) = &global.init {
                write!(f, " = {}", init)?;
            }
            writeln!(f)?;
        }

        if !self.globals.is_empty() && !self.functions.is_empty() {
            writeln!(f)?;
        }

        for func in self.functions.values() {
            writeln!(f, "{}", func)?;
        }

        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_display() {
        let reg = Register(0);
        assert_eq!(format!("{}", reg), "%0");
    }

    #[test]
    fn test_block_id_display() {
        let block = BlockId(0);
        assert_eq!(format!("{}", block), "bb0");
    }

    #[test]
    fn test_ir_type_size() {
        assert_eq!(IrType::I32.size_bytes(), 4);
        assert_eq!(IrType::I64.size_bytes(), 8);
        assert_eq!(IrType::Bool.size_bytes(), 1);
        assert_eq!(IrType::Ptr.size_bytes(), 8);
    }

    #[test]
    fn test_constant_display() {
        assert_eq!(format!("{}", Constant::I32(42)), "42");
        assert_eq!(format!("{}", Constant::Bool(true)), "true");
    }

    #[test]
    fn test_basic_block() {
        let mut block = BasicBlock::new(BlockId(0));
        assert!(!block.is_terminated());

        block.terminator = Some(Terminator::Return(None));
        assert!(block.is_terminated());
    }

    #[test]
    fn test_function_register_allocation() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::Unit,
        );

        let r0 = func.new_register();
        let r1 = func.new_register();

        assert_eq!(r0.0, 0);
        assert_eq!(r1.0, 1);
    }

    #[test]
    fn test_function_block_creation() {
        let mut func = Function::new(
            FunctionId("test".to_string()),
            vec![],
            IrType::Unit,
        );

        let b0 = func.new_block();
        let b1 = func.new_block();

        assert_eq!(b0.0, 0);
        assert_eq!(b1.0, 1);
        assert_eq!(func.blocks.len(), 2);
    }
}

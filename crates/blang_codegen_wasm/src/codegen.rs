//! WebAssembly code generation from Blang IR

use blang_ir::{self, BinaryOp, Constant, FunctionId, IrType, Terminator, Value};
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, EntityType, ExportKind, ExportSection, FunctionSection, ImportSection,
    Instruction as WI, TypeSection, ValType,
};

/// Code generation error
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("Unsupported IR type: {0}")]
    UnsupportedType(String),
    #[error("Unsupported instruction: {0}")]
    UnsupportedInstruction(String),
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

pub type CodegenResult<T> = Result<T, CodegenError>;

/// WebAssembly code generator
pub struct WasmCodegen {
    function_indices: HashMap<FunctionId, u32>,
    import_count: u32,
}

impl WasmCodegen {
    pub fn new() -> Self {
        Self {
            function_indices: HashMap::new(),
            import_count: 0,
        }
    }

    /// Compile an IR module to WASM bytes
    pub fn compile_module(&mut self, module: &blang_ir::Module) -> CodegenResult<Vec<u8>> {
        let mut wasm_module = wasm_encoder::Module::new();

        // Type section
        let mut types = TypeSection::new();
        let mut type_indices = HashMap::new();

        // Add println type first: (i32) -> ()
        types.ty().function(vec![ValType::I32], vec![]);
        let println_type_idx = 0;

        // Add function types, offset by 1 to account for println
        for (idx, func) in module.functions.values().enumerate() {
            let params: Vec<_> = func
                .params
                .iter()
                .map(|p| convert_type(&p.ty))
                .collect::<CodegenResult<_>>()?;
            let results = vec![convert_type(&func.return_type)?];

            types.ty().function(params, results);
            type_indices.insert(func.id.clone(), (idx + 1) as u32);
        }

        wasm_module.section(&types);

        // Import section
        let mut imports = ImportSection::new();
        imports.import("env", "println", EntityType::Function(println_type_idx));
        self.import_count = 1;
        wasm_module.section(&imports);

        // Function section
        let mut functions = FunctionSection::new();
        for (idx, func) in module.functions.values().enumerate() {
            functions.function(*type_indices.get(&func.id).unwrap());
            self.function_indices
                .insert(func.id.clone(), self.import_count + idx as u32);
        }
        wasm_module.section(&functions);

        // Export section
        let mut exports = ExportSection::new();
        for (idx, func) in module.functions.values().enumerate() {
            exports.export(&func.id.0, ExportKind::Func, self.import_count + idx as u32);
        }
        wasm_module.section(&exports);

        // Code section
        let mut code = CodeSection::new();
        for func in module.functions.values() {
            let body = self.compile_function(func)?;
            code.function(&body);
        }
        wasm_module.section(&code);

        Ok(wasm_module.finish())
    }

    fn compile_function(&self, func: &blang_ir::Function) -> CodegenResult<wasm_encoder::Function> {
        let num_params = func.params.len() as u32;
        let max_register = func.next_register;

        let mut locals = vec![];
        if max_register > num_params {
            for _ in num_params..max_register {
                locals.push((1, ValType::I32));
            }
        }

        let mut wasm_func = wasm_encoder::Function::new(locals);

        if !func.blocks.is_empty() {
            let entry = &func.blocks[0];

            for inst in &entry.instructions {
                self.compile_inst(inst, &mut wasm_func)?;
            }

            if let Some(ref term) = entry.terminator {
                self.compile_term(term, &mut wasm_func)?;
            }
        }

        wasm_func.instruction(&WI::End);
        Ok(wasm_func)
    }

    fn compile_inst(&self, inst: &blang_ir::Instruction, f: &mut wasm_encoder::Function) -> CodegenResult<()> {
        match inst {
            blang_ir::Instruction::Binary {result, op, lhs, rhs, ty} => {
                self.compile_val(lhs, f)?;
                self.compile_val(rhs, f)?;

                let wi = match (op, ty) {
                    (BinaryOp::Add, IrType::I32) => WI::I32Add,
                    (BinaryOp::Sub, IrType::I32) => WI::I32Sub,
                    (BinaryOp::Mul, IrType::I32) => WI::I32Mul,
                    (BinaryOp::Div, IrType::I32) => WI::I32DivS,
                    (BinaryOp::Eq, IrType::I32) => WI::I32Eq,
                    (BinaryOp::Ne, IrType::I32) => WI::I32Ne,
                    (BinaryOp::Lt, IrType::I32) => WI::I32LtS,
                    (BinaryOp::Le, IrType::I32) => WI::I32LeS,
                    (BinaryOp::Gt, IrType::I32) => WI::I32GtS,
                    (BinaryOp::Ge, IrType::I32) => WI::I32GeS,
                    _ => return Err(CodegenError::UnsupportedInstruction(format!("{:?}", op))),
                };

                f.instruction(&wi);
                f.instruction(&WI::LocalSet(result.0));
            }
            blang_ir::Instruction::Copy {result, value, ..} => {
                self.compile_val(value, f)?;
                f.instruction(&WI::LocalSet(result.0));
            }
            _ => return Err(CodegenError::UnsupportedInstruction("complex inst".to_string())),
        }
        Ok(())
    }

    fn compile_term(&self, term: &Terminator, f: &mut wasm_encoder::Function) -> CodegenResult<()> {
        match term {
            Terminator::Return(Some(val)) => {
                self.compile_val(val, f)?;
                f.instruction(&WI::Return);
            }
            Terminator::Return(None) => {
                f.instruction(&WI::Return);
            }
            _ => {
                f.instruction(&WI::Return);
            }
        }
        Ok(())
    }

    fn compile_val(&self, val: &Value, f: &mut wasm_encoder::Function) -> CodegenResult<()> {
        match val {
            Value::Constant(c) => match c {
                Constant::I32(n) => {
                    f.instruction(&WI::I32Const(*n));
                }
                Constant::I64(n) => {
                    f.instruction(&WI::I64Const(*n));
                }
                Constant::Bool(b) => {
                    f.instruction(&WI::I32Const(if *b { 1 } else { 0 }));
                }
                _ => return Err(CodegenError::UnsupportedType(format!("{:?}", c))),
            },
            Value::Register(reg) => {
                f.instruction(&WI::LocalGet(reg.0));
            }
        }
        Ok(())
    }
}

impl Default for WasmCodegen {
    fn default() -> Self {
        Self::new()
    }
}

fn convert_type(ty: &IrType) -> CodegenResult<ValType> {
    match ty {
        IrType::I8 | IrType::I16 | IrType::I32 | IrType::U8 | IrType::U16 | IrType::U32 | IrType::Bool => Ok(ValType::I32),
        IrType::I64 | IrType::U64 => Ok(ValType::I64),
        IrType::F32 => Ok(ValType::F32),
        IrType::F64 => Ok(ValType::F64),
        IrType::Unit => Ok(ValType::I32),
        IrType::Ptr => Ok(ValType::I32),
        _ => Err(CodegenError::UnsupportedType(format!("{:?}", ty))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_type() {
        assert_eq!(convert_type(&IrType::I32).unwrap(), ValType::I32);
        assert_eq!(convert_type(&IrType::I64).unwrap(), ValType::I64);
    }

    #[test]
    fn test_codegen_creation() {
        let codegen = WasmCodegen::new();
        assert_eq!(codegen.import_count, 0);
    }
}

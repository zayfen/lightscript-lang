//! x86-64 assembly code generator

use crate::ir::instructions::*;
use crate::codegen::CodeGenerator;

pub struct X86_64Generator;

impl X86_64Generator {
    pub fn new() -> Self {
        X86_64Generator
    }
}

impl CodeGenerator for X86_64Generator {
    fn generate(&self, module: &crate::ir::IRModule) -> Result<String, String> {
        let mut output = String::new();
        
        output.push_str(".section .text\n");
        output.push_str(".globl _start\n\n");
        
        for func in &module.functions {
            output.push_str(&format!("{}:\n", func.name));
            output.push_str("    pushq %rbp\n");
            output.push_str("    movq %rsp, %rbp\n\n");
            
            for instr in &func.instructions {
                output.push_str(&self.generate_instruction(instr));
            }
            
            output.push_str("\n");
        }
        
        output.push_str("_start:\n");
        output.push_str("    call main\n");
        output.push_str("    movq %rax, %rdi\n");
        output.push_str("    movq $60, %rax\n");
        output.push_str("    syscall\n");
        
        Ok(output)
    }
}

impl X86_64Generator {
    fn generate_instruction(&self, instr: &IRInstruction) -> String {
        match instr {
            IRInstruction::Add { dest: _, ty: _, lhs, rhs } => {
                format!("    # add {}, {}\n", lhs, rhs)
            },
            IRInstruction::Sub { dest: _, ty: _, lhs, rhs } => {
                format!("    # sub {}, {}\n", lhs, rhs)
            },
            IRInstruction::Mul { dest: _, ty: _, lhs, rhs } => {
                format!("    # mul {}, {}\n", lhs, rhs)
            },
            IRInstruction::Div { dest: _, ty: _, lhs, rhs } => {
                format!("    # div {}, {}\n", lhs, rhs)
            },
            IRInstruction::Ret { ty: _, value } => {
                if let Some(v) = value {
                    match v {
                        IRValue::Const(n) => format!("    movq ${}, %rax\n", n),
                        _ => "    movq %rax, %rax\n".to_string(),
                    }
                } else {
                    "    xorq %rax, %rax\n".to_string()
                }
            },
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_64_generator() {
        let gen = X86_64Generator::new();
        let module = crate::ir::IRModule::new();
        let result = gen.generate(&module);
        assert!(result.is_ok());
    }
}

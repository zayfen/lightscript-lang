//! IR Builder - converts AST to IR

use crate::parser::ast::*;
use crate::ir::instructions::*;
use std::collections::HashMap;

pub struct IRBuilder {
    module: crate::ir::IRModule,
    current_function: Option<IRFunction>,
    var_counter: usize,
    label_counter: usize,
    variables: HashMap<String, String>,
}

impl IRBuilder {
    pub fn new() -> Self {
        IRBuilder {
            module: crate::ir::IRModule::new(),
            current_function: None,
            var_counter: 0,
            label_counter: 0,
            variables: HashMap::new(),
        }
    }
    
    fn fresh_var(&mut self) -> String {
        let name = format!("t{}", self.var_counter);
        self.var_counter += 1;
        name
    }
    
    pub fn build(mut self, program: &Program) -> crate::ir::IRModule {
        let mut main_func = IRFunction::new("main".to_string(), IRType::I64);
        
        for stmt in &program.statements {
            self.build_stmt(stmt, &mut main_func);
        }
        
        main_func.add_instruction(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });
        
        self.module.add_function(main_func);
        self.module
    }
    
    fn build_stmt(&mut self, stmt: &Stmt, func: &mut IRFunction) {
        match stmt {
            Stmt::Expression(expr) => {
                self.build_expr(expr, func);
            },
            
            Stmt::VariableDecl { name, init, .. } => {
                let ptr = self.fresh_var();
                func.add_instruction(IRInstruction::Alloc {
                    dest: ptr.clone(),
                    ty: IRType::I64,
                });
                
                if let Some(init_expr) = init {
                    let value = self.build_expr(init_expr, func);
                    func.add_instruction(IRInstruction::Store {
                        dest: ptr.clone(),
                        ty: IRType::I64,
                        value,
                    });
                }
                
                self.variables.insert(name.clone(), ptr);
            },
        }
    }
    
    fn build_expr(&mut self, expr: &Expr, func: &mut IRFunction) -> IRValue {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Number(n) => IRValue::Const(*n),
                    Literal::Float(f) => IRValue::ConstF(*f),
                    Literal::String(s) => IRValue::ConstStr(s.clone()),
                    Literal::Boolean(b) => IRValue::Const(if *b { 1 } else { 0 }),
                }
            },
            
            Expr::Identifier(name) => {
                let ptr = self.variables.get(name).cloned();
                if let Some(ptr) = ptr {
                    let dest = self.fresh_var();
                    func.add_instruction(IRInstruction::Load {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        ptr: ptr,
                    });
                    IRValue::Var(dest)
                } else {
                    IRValue::Const(0)
                }
            },
            
            Expr::Binary { left, op, right } => {
                let lhs = self.build_expr(left, func);
                let rhs = self.build_expr(right, func);
                let dest = self.fresh_var();
                
                let instr = match op {
                    BinaryOp::Add => IRInstruction::Add {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Sub => IRInstruction::Sub {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Mul => IRInstruction::Mul {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Div => IRInstruction::Div {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                };
                
                func.add_instruction(instr);
                IRValue::Var(dest)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_builder() {
        let program = Program::new(vec![]);
        let builder = IRBuilder::new();
        let module = builder.build(&program);
        assert_eq!(module.functions.len(), 1);
    }
}

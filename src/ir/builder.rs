//! IR Builder - converts AST to IR

use crate::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use crate::parser::ast::*;
use std::collections::HashMap;

pub struct IRBuilder {
    module: IRModule,
    var_counter: usize,
    label_counter: usize,
    variables: HashMap<String, String>,
    last_expr_value: Option<IRValue>,
}

impl IRBuilder {
    pub fn new() -> Self {
        IRBuilder {
            module: IRModule::new(),
            var_counter: 0,
            label_counter: 0,
            variables: HashMap::new(),
            last_expr_value: None,
        }
    }

    fn fresh_var(&mut self) -> String {
        let name = format!("t{}", self.var_counter);
        self.var_counter += 1;
        name
    }

    fn fresh_label(&mut self) -> String {
        let name = format!("L{}", self.label_counter);
        self.label_counter += 1;
        name
    }

    pub fn build(mut self, program: &Program) -> IRModule {
        // First pass: collect all function definitions
        for stmt in &program.statements {
            if let Stmt::FunctionDecl {
                name, params, body, ..
            } = stmt
            {
                let mut func = IRFunction::new(name.clone(), IRType::I64);

                for (i, param) in params.iter().enumerate() {
                    let ptr = format!("arg{}", i);
                    func.add_instruction(IRInstruction::Alloc {
                        dest: ptr.clone(),
                        ty: IRType::I64,
                    });
                    self.variables.insert(param.name.clone(), ptr);
                }

                // Build function body
                for body_stmt in body {
                    self.build_stmt(body_stmt, &mut func);
                }

                // Add implicit return if not present
                func.add_instruction(IRInstruction::Ret {
                    ty: IRType::I64,
                    value: Some(IRValue::Const(0)),
                });

                self.module.add_function(func);
            }
        }

        // Second pass: build main function with non-function statements
        let mut main_func = IRFunction::new("main".to_string(), IRType::I64);

        for stmt in &program.statements {
            match stmt {
                Stmt::FunctionDecl { .. } => {} // Skip, already processed
                _ => self.build_stmt(stmt, &mut main_func),
            }
        }

        let ret_value = if let Some(value) = self.last_expr_value.take() {
            Some(value)
        } else {
            Some(IRValue::Const(0))
        };

        main_func.add_instruction(IRInstruction::Ret {
            ty: IRType::I64,
            value: ret_value,
        });

        self.module.add_function(main_func);
        self.module
    }

    fn build_stmt(&mut self, stmt: &Stmt, func: &mut IRFunction) {
        match stmt {
            Stmt::Expression(expr) => {
                let value = self.build_expr(expr, func);
                self.last_expr_value = Some(value);
            }

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
                    self.last_expr_value = Some(IRValue::Var(ptr.clone()));
                }

                self.variables.insert(name.clone(), ptr);
            }

            Stmt::Assignment { name, value } => {
                if let Some(ptr) = self.variables.get(name).cloned() {
                    let val = self.build_expr(value, func);
                    func.add_instruction(IRInstruction::Store {
                        dest: ptr,
                        ty: IRType::I64,
                        value: val,
                    });
                }
            }

            Stmt::FunctionDecl { .. } => {}

            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    Some(self.build_expr(e, func))
                } else {
                    None
                };
                func.add_instruction(IRInstruction::Ret {
                    ty: IRType::I64,
                    value,
                });
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.build_expr(condition, func);
                let then_label = self.fresh_label();
                let else_label = self.fresh_label();
                let end_label = self.fresh_label();

                func.add_instruction(IRInstruction::CondBranch {
                    condition: cond_val,
                    true_label: then_label.clone(),
                    false_label: else_label.clone(),
                });

                func.add_instruction(IRInstruction::Label(then_label));
                for stmt in then_branch {
                    self.build_stmt(stmt, func);
                }
                func.add_instruction(IRInstruction::Jump(end_label.clone()));

                func.add_instruction(IRInstruction::Label(else_label));
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.build_stmt(stmt, func);
                    }
                }
                func.add_instruction(IRInstruction::Jump(end_label.clone()));

                func.add_instruction(IRInstruction::Label(end_label));
            }

            Stmt::While { condition, body } => {
                let start_label = self.fresh_label();
                let body_label = self.fresh_label();
                let end_label = self.fresh_label();

                func.add_instruction(IRInstruction::Label(start_label.clone()));
                let cond_val = self.build_expr(condition, func);
                func.add_instruction(IRInstruction::CondBranch {
                    condition: cond_val,
                    true_label: body_label.clone(),
                    false_label: end_label.clone(),
                });

                func.add_instruction(IRInstruction::Label(body_label));
                for stmt in body {
                    self.build_stmt(stmt, func);
                }
                func.add_instruction(IRInstruction::Jump(start_label));

                func.add_instruction(IRInstruction::Label(end_label));
            }

            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.build_stmt(stmt, func);
                }
            }
        }
    }

    fn build_expr(&mut self, expr: &Expr, func: &mut IRFunction) -> IRValue {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => IRValue::Const(*n),
                _ => IRValue::Const(0),
            },

            Expr::Identifier(name) => {
                if let Some(ptr) = self.variables.get(name).cloned() {
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
            }

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
                    BinaryOp::Eq => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Eq,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Ne => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Ne,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Lt => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Lt,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Le => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Le,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Gt => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Gt,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Ge => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Ge,
                        lhs,
                        rhs,
                    },
                    _ => IRInstruction::Add {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs: IRValue::Const(0),
                        rhs: IRValue::Const(0),
                    },
                };

                func.add_instruction(instr);
                IRValue::Var(dest)
            }

            Expr::Call { callee, args } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.build_expr(arg, func));
                }

                let dest = self.fresh_var();
                func.add_instruction(IRInstruction::Call {
                    result: Some(dest.clone()),
                    function: callee.clone(),
                    args: arg_values,
                });
                IRValue::Var(dest)
            }
        }
    }
}

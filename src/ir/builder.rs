//! IR Builder - converts AST to IR

use crate::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use crate::parser::ast::*;
use crate::stdlib::Stdlib;
use std::collections::{HashMap, HashSet};

pub struct IRBuilder {
    module: IRModule,
    var_counter: usize,
    label_counter: usize,
    variables: HashMap<String, String>,
    defined_functions: HashSet<String>,
    builtin_functions: HashSet<String>,
    last_expr_value: Option<IRValue>,
    // Track if current block has a terminator (return/branch)
    current_block_terminated: bool,
    struct_defs: HashMap<String, Vec<String>>,
    struct_var_types: HashMap<String, String>,
    struct_field_ptrs: HashMap<(String, String), String>,
}

impl IRBuilder {
    const PRINT_I64: &'static str = "ziv_print_i64";
    const PRINTLN_I64: &'static str = "ziv_println_i64";
    const PRINT_STR: &'static str = "ziv_print_str";
    const PRINTLN_STR: &'static str = "ziv_println_str";

    pub fn new() -> Self {
        let builtin_functions = Stdlib::new()
            .all_functions()
            .into_iter()
            .map(|func| func.name.clone())
            .collect();

        IRBuilder {
            module: IRModule::new(),
            var_counter: 0,
            label_counter: 0,
            variables: HashMap::new(),
            defined_functions: HashSet::new(),
            builtin_functions,
            last_expr_value: None,
            current_block_terminated: false,
            struct_defs: HashMap::new(),
            struct_var_types: HashMap::new(),
            struct_field_ptrs: HashMap::new(),
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

    fn add_instr(&mut self, func: &mut IRFunction, instr: IRInstruction) {
        // Label always starts a new block, even if previous was terminated
        if let IRInstruction::Label(label_name) = &instr {
            // If previous block wasn't terminated, add a jump to this label
            // This handles fall-through between basic blocks
            if !self.current_block_terminated {
                func.add_instruction(IRInstruction::Jump(label_name.clone()));
            }
            self.current_block_terminated = false;
            func.add_instruction(instr);
            return;
        }

        // Don't add other instructions if current block is already terminated
        if self.current_block_terminated {
            return;
        }

        // Check if this instruction terminates the block
        match &instr {
            IRInstruction::Ret { .. }
            | IRInstruction::Jump(_)
            | IRInstruction::CondBranch { .. } => {
                self.current_block_terminated = true;
            }
            _ => {}
        }

        func.add_instruction(instr);
    }

    pub fn build(mut self, program: &Program) -> IRModule {
        for stmt in &program.statements {
            if let Stmt::StructDecl { name, fields } = stmt {
                self.register_struct_decl(name, fields);
            }
        }

        self.defined_functions = program
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::FunctionDecl { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // First pass: collect all function definitions
        for stmt in &program.statements {
            if let Stmt::FunctionDecl {
                name, params, body, ..
            } = stmt
            {
                // Reset state for each function
                self.current_block_terminated = false;
                self.var_counter = 0;
                self.label_counter = 0;
                self.variables.clear();
                self.struct_var_types.clear();
                self.struct_field_ptrs.clear();

                let mut func = IRFunction::new(name.clone(), IRType::I64);

                for (i, param) in params.iter().enumerate() {
                    let ptr = format!("arg{}", i);
                    // Add parameter to function signature
                    func.params.push((ptr.clone(), IRType::I64));
                    self.add_instr(
                        &mut func,
                        IRInstruction::Alloc {
                            dest: ptr.clone(),
                            ty: IRType::I64,
                        },
                    );
                    self.variables.insert(param.name.clone(), ptr);
                }

                // Build function body
                for body_stmt in body {
                    self.build_stmt(body_stmt, &mut func);
                }

                // Add implicit return if not present
                self.add_instr(
                    &mut func,
                    IRInstruction::Ret {
                        ty: IRType::I64,
                        value: Some(IRValue::Const(0)),
                    },
                );

                self.module.add_function(func);
            }
        }

        // Second pass: build main function with non-function statements
        self.current_block_terminated = false;
        self.var_counter = 0;
        self.variables.clear();
        self.struct_var_types.clear();
        self.struct_field_ptrs.clear();

        // Use _user_main to avoid conflict with C runtime's main
        let mut main_func = IRFunction::new("_user_main".to_string(), IRType::I64);

        for stmt in &program.statements {
            match stmt {
                Stmt::FunctionDecl { .. } | Stmt::Import { .. } | Stmt::StructDecl { .. } => {} // Skip, already processed
                _ => self.build_stmt(stmt, &mut main_func),
            }
        }

        self.add_instr(
            &mut main_func,
            IRInstruction::Ret {
                ty: IRType::I64,
                // Keep process exit deterministic; examples that represent importable modules
                // should still run successfully with code 0.
                value: Some(IRValue::Const(0)),
            },
        );

        self.module.add_function(main_func);
        self.module
    }

    fn register_struct_decl(&mut self, name: &str, fields: &[StructFieldDecl]) {
        let field_names = fields.iter().map(|field| field.name.clone()).collect();
        self.struct_defs.insert(name.to_string(), field_names);
    }

    fn resolve_struct_var_type(
        &self,
        type_annotation: &Option<String>,
        init: &Option<Expr>,
    ) -> Option<String> {
        if let Some(type_name) = type_annotation {
            if self.struct_defs.contains_key(type_name) {
                return Some(type_name.clone());
            }
        }

        if let Some(Expr::StructInit { struct_name, .. }) = init {
            if self.struct_defs.contains_key(struct_name) {
                return Some(struct_name.clone());
            }
        }

        None
    }

    fn build_struct_var_decl(
        &mut self,
        func: &mut IRFunction,
        var_name: &str,
        struct_name: &str,
        init: Option<&Expr>,
    ) {
        let Some(field_order) = self.struct_defs.get(struct_name).cloned() else {
            return;
        };

        self.struct_var_types
            .insert(var_name.to_string(), struct_name.to_string());

        for field in field_order {
            let ptr = self.fresh_var();
            self.add_instr(
                func,
                IRInstruction::Alloc {
                    dest: ptr.clone(),
                    ty: IRType::I64,
                },
            );

            let value = match init {
                Some(Expr::StructInit { fields, .. }) => self
                    .find_struct_field_init(fields, &field)
                    .map(|expr| self.build_expr(expr, func))
                    .unwrap_or(IRValue::Const(0)),
                _ => IRValue::Const(0),
            };

            self.add_instr(
                func,
                IRInstruction::Store {
                    dest: ptr.clone(),
                    ty: IRType::I64,
                    value,
                },
            );
            self.struct_field_ptrs
                .insert((var_name.to_string(), field), ptr);
        }
    }

    fn find_struct_field_init<'a>(
        &self,
        fields: &'a [StructFieldInit],
        name: &str,
    ) -> Option<&'a Expr> {
        fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| &field.value)
    }

    fn apply_struct_update(
        &mut self,
        func: &mut IRFunction,
        var_name: &str,
        struct_name: &str,
        fields: &[StructFieldInit],
        partial: bool,
    ) {
        let Some(var_struct_name) = self.struct_var_types.get(var_name).cloned() else {
            return;
        };
        if var_struct_name != struct_name {
            return;
        }

        if partial {
            for field in fields {
                if let Some(ptr) = self
                    .struct_field_ptrs
                    .get(&(var_name.to_string(), field.name.clone()))
                    .cloned()
                {
                    let value = self.build_expr(&field.value, func);
                    self.add_instr(
                        func,
                        IRInstruction::Store {
                            dest: ptr,
                            ty: IRType::I64,
                            value,
                        },
                    );
                }
            }
            return;
        }

        let Some(field_order) = self.struct_defs.get(&var_struct_name).cloned() else {
            return;
        };
        for field_name in field_order {
            if let Some(ptr) = self
                .struct_field_ptrs
                .get(&(var_name.to_string(), field_name.clone()))
                .cloned()
            {
                let value = self
                    .find_struct_field_init(fields, &field_name)
                    .map(|expr| self.build_expr(expr, func))
                    .unwrap_or(IRValue::Const(0));
                self.add_instr(
                    func,
                    IRInstruction::Store {
                        dest: ptr,
                        ty: IRType::I64,
                        value,
                    },
                );
            }
        }
    }

    fn build_stmt(&mut self, stmt: &Stmt, func: &mut IRFunction) {
        match stmt {
            Stmt::Import { .. } => {}

            Stmt::StructDecl { name, fields } => {
                self.register_struct_decl(name, fields);
            }

            Stmt::Expression(expr) => {
                let value = self.build_expr(expr, func);
                self.last_expr_value = Some(value);
            }

            Stmt::VariableDecl {
                name,
                type_annotation,
                init,
                ..
            } => {
                if let Some(struct_name) = self.resolve_struct_var_type(type_annotation, init) {
                    self.build_struct_var_decl(func, name, &struct_name, init.as_ref());
                    return;
                }

                let ptr = self.fresh_var();
                self.add_instr(
                    func,
                    IRInstruction::Alloc {
                        dest: ptr.clone(),
                        ty: IRType::I64,
                    },
                );

                if let Some(init_expr) = init {
                    let value = self.build_expr(init_expr, func);
                    self.add_instr(
                        func,
                        IRInstruction::Store {
                            dest: ptr.clone(),
                            ty: IRType::I64,
                            value,
                        },
                    );
                    self.last_expr_value = Some(IRValue::Var(ptr.clone()));
                }

                self.variables.insert(name.clone(), ptr);
            }

            Stmt::Assignment { name, value } => {
                if self.struct_var_types.contains_key(name) {
                    if let Expr::StructInit {
                        struct_name,
                        fields,
                    } = value
                    {
                        self.apply_struct_update(func, name, struct_name, fields, false);
                    }
                    return;
                }

                if let Some(ptr) = self.variables.get(name).cloned() {
                    let val = self.build_expr(value, func);
                    self.add_instr(
                        func,
                        IRInstruction::Store {
                            dest: ptr,
                            ty: IRType::I64,
                            value: val,
                        },
                    );
                }
            }

            Stmt::StructMergeAssign { name, value } => {
                if let Expr::StructInit {
                    struct_name,
                    fields,
                } = value
                {
                    self.apply_struct_update(func, name, struct_name, fields, true);
                }
            }

            Stmt::FunctionDecl { .. } => {}

            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    Some(self.build_expr(e, func))
                } else {
                    None
                };
                self.add_instr(
                    func,
                    IRInstruction::Ret {
                        ty: IRType::I64,
                        value,
                    },
                );
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

                self.add_instr(
                    func,
                    IRInstruction::CondBranch {
                        condition: cond_val,
                        true_label: then_label.clone(),
                        false_label: else_label.clone(),
                    },
                );

                // Then branch
                self.add_instr(func, IRInstruction::Label(then_label));
                for stmt in then_branch {
                    self.build_stmt(stmt, func);
                }
                // Only add jump if block wasn't terminated by return
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(end_label.clone()));
                }

                // Else branch
                self.add_instr(func, IRInstruction::Label(else_label));
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.build_stmt(stmt, func);
                    }
                }
                // Only add jump if block wasn't terminated by return
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(end_label.clone()));
                }

                // End label
                self.add_instr(func, IRInstruction::Label(end_label));
            }

            Stmt::While { condition, body } => {
                let start_label = self.fresh_label();
                let body_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.add_instr(func, IRInstruction::Label(start_label.clone()));
                let cond_val = self.build_expr(condition, func);
                self.add_instr(
                    func,
                    IRInstruction::CondBranch {
                        condition: cond_val,
                        true_label: body_label.clone(),
                        false_label: end_label.clone(),
                    },
                );

                self.add_instr(func, IRInstruction::Label(body_label));
                for stmt in body {
                    self.build_stmt(stmt, func);
                }
                // Jump back to start if not terminated
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(start_label));
                }

                self.add_instr(func, IRInstruction::Label(end_label));
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
                Literal::String(s) => IRValue::Str(s.clone()),
                _ => IRValue::Const(0),
            },

            Expr::Identifier(name) => {
                if let Some(ptr) = self.variables.get(name).cloned() {
                    let dest = self.fresh_var();
                    self.add_instr(
                        func,
                        IRInstruction::Load {
                            dest: dest.clone(),
                            ty: IRType::I64,
                            ptr: ptr,
                        },
                    );
                    IRValue::Var(dest)
                } else {
                    IRValue::Const(0)
                }
            }

            Expr::StructInit { .. } => IRValue::Const(0),

            Expr::FieldAccess { object, field } => {
                if let Expr::Identifier(var_name) = object.as_ref() {
                    if let Some(ptr) = self
                        .struct_field_ptrs
                        .get(&(var_name.clone(), field.clone()))
                        .cloned()
                    {
                        let dest = self.fresh_var();
                        self.add_instr(
                            func,
                            IRInstruction::Load {
                                dest: dest.clone(),
                                ty: IRType::I64,
                                ptr,
                            },
                        );
                        return IRValue::Var(dest);
                    }
                }
                IRValue::Const(0)
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

                self.add_instr(func, instr);
                IRValue::Var(dest)
            }

            Expr::Call { callee, args } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.build_expr(arg, func));
                }

                // Keep most built-ins semantic-only for now, but lower print/println to
                // concrete runtime calls so executable output matches source behavior.
                if self.builtin_functions.contains(callee)
                    && !self.defined_functions.contains(callee)
                {
                    if matches!(callee.as_str(), "print" | "println") {
                        if let Some(value) = arg_values.first() {
                            let function = match (callee.as_str(), value) {
                                ("print", IRValue::Str(_)) => Self::PRINT_STR,
                                ("println", IRValue::Str(_)) => Self::PRINTLN_STR,
                                ("print", _) => Self::PRINT_I64,
                                ("println", _) => Self::PRINTLN_I64,
                                _ => unreachable!(),
                            };

                            self.add_instr(
                                func,
                                IRInstruction::Call {
                                    result: None,
                                    function: function.to_string(),
                                    args: arg_values,
                                },
                            );
                        }
                    }
                    return IRValue::Const(0);
                }

                let dest = self.fresh_var();
                self.add_instr(
                    func,
                    IRInstruction::Call {
                        result: Some(dest.clone()),
                        function: callee.clone(),
                        args: arg_values,
                    },
                );
                IRValue::Var(dest)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_builtin_call_lowers_println_to_runtime_helper() {
        let mut parser = Parser::new(
            r#"
            function helper() { return 1; }
            helper();
            println(42);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let has_direct_println_call = main.instructions.iter().any(|instr| match instr {
            IRInstruction::Call { function, .. } => function == "println",
            _ => false,
        });
        let has_runtime_println = main.instructions.iter().any(|instr| match instr {
            IRInstruction::Call { function, .. } => function == IRBuilder::PRINTLN_I64,
            _ => false,
        });
        assert!(!has_direct_println_call);
        assert!(has_runtime_println);
    }

    #[test]
    fn test_build_no_else_block_div_eq_and_nested_function_decl_paths() {
        let nested = Stmt::FunctionDecl {
            name: "inner".to_string(),
            params: vec![],
            return_type: None,
            body: vec![Stmt::Return(Some(Expr::Literal(Literal::Number(1))))],
        };
        let outer = Stmt::FunctionDecl {
            name: "outer".to_string(),
            params: vec![],
            return_type: None,
            body: vec![nested, Stmt::Return(None)],
        };

        let program = Program::new(vec![
            outer,
            Stmt::If {
                condition: Expr::Literal(Literal::Boolean(true)),
                then_branch: vec![Stmt::Expression(Expr::Literal(Literal::Number(1)))],
                else_branch: None,
            },
            Stmt::Block(vec![Stmt::Expression(Expr::Literal(Literal::Number(2)))]),
            Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(8))),
                op: BinaryOp::Div,
                right: Box::new(Expr::Literal(Literal::Number(2))),
            }),
            Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(1))),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Literal(Literal::Number(1))),
            }),
        ]);

        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::Div { .. })));
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Cmp {
                op: crate::ir::CmpOp::Eq,
                ..
            }
        )));

        let outer_fn = module
            .functions
            .iter()
            .find(|func| func.name == "outer")
            .unwrap();
        assert!(outer_fn
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::Ret { value: None, .. })));
    }

    #[test]
    fn test_user_defined_function_call_is_preserved() {
        let mut parser = Parser::new(
            r#"
            function print(x) { return x; }
            print(1);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let has_user_print_call = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == "print"
            )
        });
        assert!(has_user_print_call);
    }

    #[test]
    fn test_string_print_lowering_uses_string_runtime_helper() {
        let mut parser = Parser::new(
            r#"
            print("a");
            println("b");
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let has_print_str = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, args, .. }
                    if function == IRBuilder::PRINT_STR
                        && matches!(args.first(), Some(IRValue::Str(value)) if value == "a")
            )
        });
        let has_println_str = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, args, .. }
                    if function == IRBuilder::PRINTLN_STR
                        && matches!(args.first(), Some(IRValue::Str(value)) if value == "b")
            )
        });

        assert!(has_print_str);
        assert!(has_println_str);
    }

    #[test]
    fn test_build_control_flow_and_assignment_paths() {
        let mut parser = Parser::new(
            r#"
            let x;
            x = 1;
            if (x != 0) { x = x + 1; } else { x = x - 1; }
            while (x > 0) { x = x - 1; }
            x;
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::CondBranch { .. })));
        assert!(main
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::Jump(_))));
    }

    #[test]
    fn test_build_literal_fallback_and_unknown_identifier() {
        let program = Program::new(vec![
            Stmt::Expression(Expr::Literal(Literal::String("s".to_string()))),
            Stmt::Expression(Expr::Identifier("missing".to_string())),
        ]);
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Ret {
                value: Some(IRValue::Const(0)),
                ..
            }
        )));
    }

    #[test]
    fn test_build_all_comparison_ops_and_logical_fallback() {
        let exprs = vec![
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(1))),
                op: BinaryOp::Lt,
                right: Box::new(Expr::Literal(Literal::Number(2))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(1))),
                op: BinaryOp::Le,
                right: Box::new(Expr::Literal(Literal::Number(2))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(2))),
                op: BinaryOp::Gt,
                right: Box::new(Expr::Literal(Literal::Number(1))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(2))),
                op: BinaryOp::Ge,
                right: Box::new(Expr::Literal(Literal::Number(1))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Boolean(true))),
                op: BinaryOp::And,
                right: Box::new(Expr::Literal(Literal::Boolean(false))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Boolean(true))),
                op: BinaryOp::Or,
                right: Box::new(Expr::Literal(Literal::Boolean(false))),
            },
        ];

        let program = Program::new(exprs.into_iter().map(Stmt::Expression).collect::<Vec<_>>());
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Cmp {
                op: crate::ir::CmpOp::Lt,
                ..
            }
        )));
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Cmp {
                op: crate::ir::CmpOp::Ge,
                ..
            }
        )));
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Add {
                lhs: IRValue::Const(0),
                rhs: IRValue::Const(0),
                ..
            }
        )));
    }

    #[test]
    fn test_struct_field_access_and_merge_lowering() {
        let mut parser = Parser::new(
            r#"
            struct Person { age: int; score: int; }
            let p: Person = Person.(age = 18, score = 90);
            println(p.age);
            p += Person.(age = 20);
            println(p.age);
            println(p.score);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let stores = main
            .instructions
            .iter()
            .filter(|instr| matches!(instr, IRInstruction::Store { .. }))
            .count();
        let loads = main
            .instructions
            .iter()
            .filter(|instr| matches!(instr, IRInstruction::Load { .. }))
            .count();
        let print_calls = main
            .instructions
            .iter()
            .filter(|instr| {
                matches!(
                    instr,
                    IRInstruction::Call { function, .. } if function == IRBuilder::PRINTLN_I64
                )
            })
            .count();

        assert_eq!(stores, 3);
        assert_eq!(loads, 3);
        assert_eq!(print_calls, 3);
    }

    #[test]
    fn test_struct_assignment_replaces_all_fields() {
        let mut parser = Parser::new(
            r#"
            struct Person { age: int; score: int; }
            let p: Person = Person.(age = 1, score = 2);
            p = Person.(age = 3, score = 4);
            println(p.score);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let stores = main
            .instructions
            .iter()
            .filter(|instr| matches!(instr, IRInstruction::Store { .. }))
            .count();
        assert_eq!(stores, 4);
    }
}

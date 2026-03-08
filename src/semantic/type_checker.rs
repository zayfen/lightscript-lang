//! Type checker for LightLang

use super::symbols::*;
use super::types::*;
use crate::parser::ast::*;
use crate::stdlib::Stdlib;
use std::collections::{HashMap, HashSet};

pub type TypeCheckResult<T> = Result<T, String>;

/// Type checker
#[derive(Debug)]
pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    struct_defs: HashMap<String, StructDefinition>,
}

#[derive(Debug, Clone)]
struct StructDefinition {
    fields: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = TypeChecker {
            symbol_table: SymbolTable::new(),
            struct_defs: HashMap::new(),
        };
        checker.register_builtin_functions();
        checker
    }

    fn register_builtin_functions(&mut self) {
        let stdlib = Stdlib::new();
        let scope_level = self.symbol_table.current_scope_level();

        for builtin in stdlib.all_functions() {
            let param_types = builtin
                .params
                .iter()
                .map(|param| Type::from(param.ty.as_str()))
                .collect();
            let return_type = builtin
                .return_type
                .as_deref()
                .map(Type::from)
                .unwrap_or(Type::Void);

            let symbol = Symbol::new(
                builtin.name.clone(),
                SymbolKind::Function,
                Type::Function {
                    params: param_types,
                    return_type: Box::new(return_type),
                },
                scope_level,
            );
            self.symbol_table.define(symbol);
        }
    }

    pub fn check_program(&mut self, program: &Program) -> TypeCheckResult<()> {
        // First pass: register all struct definitions.
        for stmt in &program.statements {
            if matches!(stmt, Stmt::StructDecl { .. }) {
                self.check_stmt(stmt)?;
            }
        }
        self.validate_struct_field_types()?;

        // Second pass: type-check executable statements.
        for stmt in &program.statements {
            if matches!(stmt, Stmt::StructDecl { .. }) {
                continue;
            }
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn check_stmt(&mut self, stmt: &Stmt) -> TypeCheckResult<()> {
        match stmt {
            Stmt::Import { .. } => Ok(()),

            Stmt::StructDecl { name, fields } => {
                if self.struct_defs.contains_key(name) {
                    return Err(format!("Struct '{}' already defined", name));
                }

                let mut field_types = HashMap::new();
                for field in fields {
                    if field_types.contains_key(&field.name) {
                        return Err(format!(
                            "Duplicate field '{}' in struct '{}'",
                            field.name, name
                        ));
                    }
                    field_types.insert(
                        field.name.clone(),
                        self.resolve_struct_field_type(field.ty.as_str()),
                    );
                }

                self.struct_defs
                    .insert(name.clone(), StructDefinition { fields: field_types });
                Ok(())
            }

            Stmt::Expression(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }

            Stmt::VariableDecl {
                name,
                type_annotation,
                init,
                is_const,
            } => {
                let declared_ty = type_annotation
                    .as_deref()
                    .map(|type_name| self.resolve_declared_type(type_name));
                let inferred_ty = if let Some(init_expr) = init {
                    Some(self.check_expr(init_expr)?)
                } else {
                    None
                };

                let ty = match (declared_ty, inferred_ty) {
                    (Some(declared), Some(inferred)) => {
                        if !Self::is_assignable(&declared, &inferred) {
                            return Err(format!(
                                "Type mismatch for '{}': expected {}, got {}",
                                name, declared, inferred
                            ));
                        }
                        declared
                    }
                    (Some(declared), None) => declared,
                    (None, Some(inferred)) => inferred,
                    (None, None) => Type::Any,
                };

                let kind = if *is_const {
                    SymbolKind::Constant
                } else {
                    SymbolKind::Variable
                };

                let scope_level = self.symbol_table.current_scope_level();
                let symbol = Symbol::new(name.clone(), kind, ty, scope_level);
                self.symbol_table.define(symbol);

                Ok(())
            }

            Stmt::Assignment { name, value } => {
                let value_ty = self.check_expr(value)?;
                let symbol = self
                    .symbol_table
                    .lookup(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name))?;

                if symbol.kind == SymbolKind::Constant {
                    return Err("Cannot assign to constant".to_string());
                }

                if !Self::is_assignable(&symbol.ty, &value_ty) {
                    return Err(format!(
                        "Type mismatch for '{}': expected {}, got {}",
                        name, symbol.ty, value_ty
                    ));
                }

                Ok(())
            }

            Stmt::StructMergeAssign { name, value } => {
                let symbol = self
                    .symbol_table
                    .lookup(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name))?;

                if symbol.kind == SymbolKind::Constant {
                    return Err("Cannot assign to constant".to_string());
                }

                let struct_name = match symbol.ty {
                    Type::Struct(name) => name,
                    other => {
                        return Err(format!(
                            "Cannot use '+=' on non-struct variable '{}': {}",
                            name, other
                        ))
                    }
                };

                match value {
                    Expr::StructInit {
                        struct_name: init_name,
                        fields,
                    } => {
                        if init_name != &struct_name {
                            return Err(format!(
                                "Struct merge type mismatch: '{}' and '{}'",
                                struct_name, init_name
                            ));
                        }
                        self.check_struct_init(init_name, fields, true)?;
                    }
                    _ => {
                        return Err(
                            "Struct merge assignment requires struct initializer expression"
                                .to_string(),
                        )
                    }
                }

                Ok(())
            }

            Stmt::FunctionDecl {
                name,
                params,
                return_type,
                body,
            } => {
                let func_type = Type::Function {
                    params: params
                        .iter()
                        .map(|p| {
                            if let Some(type_name) = &p.type_annotation {
                                self.resolve_declared_type(type_name)
                            } else {
                                Type::Any
                            }
                        })
                        .collect(),
                    return_type: Box::new(if let Some(ret_type) = return_type {
                        self.resolve_declared_type(ret_type)
                    } else {
                        Type::Any
                    }),
                };

                let symbol = Symbol::new(
                    name.clone(),
                    SymbolKind::Function,
                    func_type,
                    self.symbol_table.current_scope_level(),
                );
                self.symbol_table.define(symbol);

                self.symbol_table.enter_scope();

                for param in params {
                    let param_type = if let Some(type_name) = &param.type_annotation {
                        self.resolve_declared_type(type_name)
                    } else {
                        Type::Any
                    };

                    let symbol = Symbol::new(
                        param.name.clone(),
                        SymbolKind::Parameter,
                        param_type,
                        self.symbol_table.current_scope_level(),
                    );
                    self.symbol_table.define(symbol);
                }

                for body_stmt in body {
                    self.check_stmt(body_stmt)?;
                }

                self.symbol_table.exit_scope();

                Ok(())
            }

            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.check_expr(expr)?;
                }
                Ok(())
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_expr(condition)?;

                self.symbol_table.enter_scope();
                for stmt in then_branch {
                    self.check_stmt(stmt)?;
                }
                self.symbol_table.exit_scope();

                if let Some(else_stmts) = else_branch {
                    self.symbol_table.enter_scope();
                    for stmt in else_stmts {
                        self.check_stmt(stmt)?;
                    }
                    self.symbol_table.exit_scope();
                }

                Ok(())
            }

            Stmt::While { condition, body } => {
                self.check_expr(condition)?;

                self.symbol_table.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.symbol_table.exit_scope();

                Ok(())
            }

            Stmt::Block(stmts) => {
                self.symbol_table.enter_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                self.symbol_table.exit_scope();
                Ok(())
            }
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> TypeCheckResult<Type> {
        match expr {
            Expr::Literal(lit) => {
                let ty = match lit {
                    Literal::Number(_) => Type::Int,
                    Literal::Float(_) => Type::Float,
                    Literal::String(_) => Type::String,
                    Literal::Boolean(_) => Type::Bool,
                };
                Ok(ty)
            }

            Expr::Identifier(name) => self
                .symbol_table
                .lookup(name)
                .map(|s| s.ty.clone())
                .ok_or_else(|| format!("Undefined variable: {}", name)),

            Expr::StructInit {
                struct_name,
                fields,
            } => self.check_struct_init(struct_name, fields, false),

            Expr::FieldAccess { object, field } => {
                let object_ty = self.check_expr(object)?;
                match object_ty {
                    Type::Struct(struct_name) => {
                        let def = self
                            .struct_defs
                            .get(&struct_name)
                            .ok_or_else(|| format!("Unknown struct '{}'", struct_name))?;
                        def.fields.get(field).cloned().ok_or_else(|| {
                            format!("Struct '{}' has no field '{}'", struct_name, field)
                        })
                    }
                    Type::Any => Ok(Type::Any),
                    other => Err(format!(
                        "Cannot access field '{}' on non-struct type {}",
                        field, other
                    )),
                }
            }

            Expr::Binary { left, op, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                let result_type = match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        if left_type == Type::Any || right_type == Type::Any {
                            Type::Any
                        } else if left_type == Type::Int && right_type == Type::Int {
                            Type::Int
                        } else {
                            return Err(format!("Type mismatch: {} and {}", left_type, right_type));
                        }
                    }

                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => Type::Bool,

                    BinaryOp::And | BinaryOp::Or => Type::Bool,
                };

                Ok(result_type)
            }

            Expr::Call { callee, args } => {
                // Check arguments
                for arg in args {
                    self.check_expr(arg)?;
                }

                // Look up function
                let func_type = self
                    .symbol_table
                    .lookup(callee)
                    .map(|s| s.ty.clone())
                    .ok_or_else(|| format!("Undefined function: {}", callee))?;

                // Return the function's return type
                match func_type {
                    Type::Function { return_type, .. } => Ok(*return_type),
                    _ => Err(format!("{} is not a function", callee)),
                }
            }
        }
    }

    fn resolve_declared_type(&self, type_name: &str) -> Type {
        let primitive = Type::from(type_name);
        if primitive != Type::Any || type_name == "any" {
            primitive
        } else if self.struct_defs.contains_key(type_name) {
            Type::Struct(type_name.to_string())
        } else {
            Type::Any
        }
    }

    fn resolve_struct_field_type(&self, type_name: &str) -> Type {
        let primitive = Type::from(type_name);
        if primitive != Type::Any || type_name == "any" {
            primitive
        } else {
            Type::Struct(type_name.to_string())
        }
    }

    fn validate_struct_field_types(&self) -> TypeCheckResult<()> {
        for (struct_name, def) in &self.struct_defs {
            for (field_name, field_ty) in &def.fields {
                if let Type::Struct(dep_name) = field_ty {
                    if !self.struct_defs.contains_key(dep_name) {
                        return Err(format!(
                            "Struct '{}' field '{}' references unknown type '{}'",
                            struct_name, field_name, dep_name
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_struct_init(
        &mut self,
        struct_name: &str,
        fields: &[StructFieldInit],
        allow_partial: bool,
    ) -> TypeCheckResult<Type> {
        let field_types = self
            .struct_defs
            .get(struct_name)
            .ok_or_else(|| format!("Unknown struct '{}'", struct_name))?
            .fields
            .clone();

        let mut seen = HashSet::new();
        for field in fields {
            if !seen.insert(field.name.clone()) {
                return Err(format!(
                    "Duplicate field '{}' in struct initializer for '{}'",
                    field.name, struct_name
                ));
            }

            let expected = field_types.get(&field.name).ok_or_else(|| {
                format!("Struct '{}' has no field '{}'", struct_name, field.name)
            })?;
            let actual = self.check_expr(&field.value)?;
            if !Self::is_assignable(expected, &actual) {
                return Err(format!(
                    "Type mismatch for '{}.{}': expected {}, got {}",
                    struct_name, field.name, expected, actual
                ));
            }
        }

        if !allow_partial {
            for field_name in field_types.keys() {
                if !seen.contains(field_name) {
                    return Err(format!(
                        "Missing field '{}' in struct initializer for '{}'",
                        field_name, struct_name
                    ));
                }
            }
        }

        Ok(Type::Struct(struct_name.to_string()))
    }

    fn is_assignable(target: &Type, value: &Type) -> bool {
        *target == Type::Any || *value == Type::Any || target == value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtins_registered_in_global_scope() {
        let checker = TypeChecker::new();
        assert!(checker.symbol_table.lookup("print").is_some());
        assert!(checker.symbol_table.lookup("println").is_some());
    }

    #[test]
    fn test_assignment_to_constant_errors() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: None,
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: true,
            })
            .unwrap();

        let err = checker
            .check_stmt(&Stmt::Assignment {
                name: "x".to_string(),
                value: Expr::Literal(Literal::Number(2)),
            })
            .unwrap_err();
        assert!(err.contains("Cannot assign to constant"));
    }

    #[test]
    fn test_typed_decl_and_assignment_to_mutable_variable() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: Some("int".to_string()),
                init: None,
                is_const: false,
            })
            .unwrap();
        checker
            .check_stmt(&Stmt::Assignment {
                name: "x".to_string(),
                value: Expr::Literal(Literal::Number(2)),
            })
            .unwrap();
    }

    #[test]
    fn test_assignment_lookup_branch_for_mutable_symbol() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "m".to_string(),
                type_annotation: None,
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: false,
            })
            .unwrap();
        assert!(checker.symbol_table.lookup("m").is_some());
        checker
            .check_stmt(&Stmt::Assignment {
                name: "m".to_string(),
                value: Expr::Literal(Literal::Number(2)),
            })
            .unwrap();
    }

    #[test]
    fn test_binary_type_rules() {
        let mut checker = TypeChecker::new();

        let int_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(2))),
        };
        assert_eq!(checker.check_expr(&int_expr).unwrap(), Type::Int);

        let cmp_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Literal(Literal::Number(1))),
        };
        assert_eq!(checker.check_expr(&cmp_expr).unwrap(), Type::Bool);

        let and_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(true))),
            op: BinaryOp::And,
            right: Box::new(Expr::Literal(Literal::Boolean(false))),
        };
        assert_eq!(checker.check_expr(&and_expr).unwrap(), Type::Bool);

        let or_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(true))),
            op: BinaryOp::Or,
            right: Box::new(Expr::Literal(Literal::Boolean(false))),
        };
        assert_eq!(checker.check_expr(&or_expr).unwrap(), Type::Bool);

        let mismatch = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::String("x".to_string()))),
        };
        let err = checker.check_expr(&mismatch).unwrap_err();
        assert!(err.contains("Type mismatch"));
    }

    #[test]
    fn test_any_type_short_circuit_for_arithmetic() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: None,
                init: None,
                is_const: false,
            })
            .unwrap();
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(1))),
        };
        assert_eq!(checker.check_expr(&expr).unwrap(), Type::Any);
    }

    #[test]
    fn test_literal_float_type() {
        let mut checker = TypeChecker::new();
        let ty = checker
            .check_expr(&Expr::Literal(Literal::Float(1.25)))
            .unwrap();
        assert_eq!(ty, Type::Float);
    }

    #[test]
    fn test_function_decl_and_call_paths() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::FunctionDecl {
                name: "f".to_string(),
                params: vec![Param {
                    name: "a".to_string(),
                    type_annotation: Some("int".to_string()),
                }],
                return_type: Some("int".to_string()),
                body: vec![Stmt::Return(Some(Expr::Identifier("a".to_string())))],
            })
            .unwrap();

        let call = Expr::Call {
            callee: "f".to_string(),
            args: vec![Expr::Literal(Literal::Number(1))],
        };
        assert_eq!(checker.check_expr(&call).unwrap(), Type::Int);

        checker
            .check_stmt(&Stmt::FunctionDecl {
                name: "g".to_string(),
                params: vec![Param {
                    name: "x".to_string(),
                    type_annotation: None,
                }],
                return_type: None,
                body: vec![Stmt::Return(None)],
            })
            .unwrap();
    }

    #[test]
    fn test_call_error_paths() {
        let mut checker = TypeChecker::new();
        let undefined_err = checker
            .check_expr(&Expr::Call {
                callee: "not_found".to_string(),
                args: vec![],
            })
            .unwrap_err();
        assert!(undefined_err.contains("Undefined function"));

        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: None,
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: false,
            })
            .unwrap();
        let non_func_err = checker
            .check_expr(&Expr::Call {
                callee: "x".to_string(),
                args: vec![],
            })
            .unwrap_err();
        assert!(non_func_err.contains("is not a function"));
    }

    #[test]
    fn test_scope_statements_paths() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::If {
                condition: Expr::Literal(Literal::Boolean(true)),
                then_branch: vec![Stmt::VariableDecl {
                    name: "a".to_string(),
                    type_annotation: None,
                    init: Some(Expr::Literal(Literal::Number(1))),
                    is_const: false,
                }],
                else_branch: Some(vec![Stmt::VariableDecl {
                    name: "b".to_string(),
                    type_annotation: None,
                    init: Some(Expr::Literal(Literal::Number(2))),
                    is_const: false,
                }]),
            })
            .unwrap();

        checker
            .check_stmt(&Stmt::While {
                condition: Expr::Literal(Literal::Boolean(true)),
                body: vec![Stmt::Expression(Expr::Literal(Literal::Number(0)))],
            })
            .unwrap();

        checker
            .check_stmt(&Stmt::Block(vec![Stmt::Return(None)]))
            .unwrap();
    }

    #[test]
    fn test_if_else_scope_exits_back_to_parent_scope() {
        let mut checker = TypeChecker::new();
        let level_before = checker.symbol_table.current_scope_level();
        checker
            .check_stmt(&Stmt::If {
                condition: Expr::Literal(Literal::Boolean(true)),
                then_branch: vec![Stmt::Expression(Expr::Literal(Literal::Number(1)))],
                else_branch: Some(vec![Stmt::Expression(Expr::Literal(Literal::Number(2)))]),
            })
            .unwrap();
        assert_eq!(checker.symbol_table.current_scope_level(), level_before);
    }

    #[test]
    fn test_struct_type_check_happy_path() {
        let program = Program::new(vec![
            Stmt::StructDecl {
                name: "Person".to_string(),
                fields: vec![
                    StructFieldDecl {
                        name: "age".to_string(),
                        ty: "int".to_string(),
                    },
                    StructFieldDecl {
                        name: "score".to_string(),
                        ty: "int".to_string(),
                    },
                ],
            },
            Stmt::VariableDecl {
                name: "p".to_string(),
                type_annotation: Some("Person".to_string()),
                init: Some(Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![
                        StructFieldInit {
                            name: "age".to_string(),
                            value: Expr::Literal(Literal::Number(18)),
                        },
                        StructFieldInit {
                            name: "score".to_string(),
                            value: Expr::Literal(Literal::Number(90)),
                        },
                    ],
                }),
                is_const: false,
            },
            Stmt::VariableDecl {
                name: "age".to_string(),
                type_annotation: Some("int".to_string()),
                init: Some(Expr::FieldAccess {
                    object: Box::new(Expr::Identifier("p".to_string())),
                    field: "age".to_string(),
                }),
                is_const: false,
            },
            Stmt::StructMergeAssign {
                name: "p".to_string(),
                value: Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(20)),
                    }],
                },
            },
        ]);

        let mut checker = TypeChecker::new();
        checker.check_program(&program).unwrap();
    }

    #[test]
    fn test_struct_init_missing_field_errors() {
        let program = Program::new(vec![
            Stmt::StructDecl {
                name: "Person".to_string(),
                fields: vec![
                    StructFieldDecl {
                        name: "age".to_string(),
                        ty: "int".to_string(),
                    },
                    StructFieldDecl {
                        name: "score".to_string(),
                        ty: "int".to_string(),
                    },
                ],
            },
            Stmt::VariableDecl {
                name: "p".to_string(),
                type_annotation: Some("Person".to_string()),
                init: Some(Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(18)),
                    }],
                }),
                is_const: false,
            },
        ]);

        let mut checker = TypeChecker::new();
        let err = checker.check_program(&program).unwrap_err();
        assert!(err.contains("Missing field"));
    }

    #[test]
    fn test_struct_merge_and_field_errors() {
        let program = Program::new(vec![
            Stmt::StructDecl {
                name: "Person".to_string(),
                fields: vec![StructFieldDecl {
                    name: "age".to_string(),
                    ty: "int".to_string(),
                }],
            },
            Stmt::VariableDecl {
                name: "p".to_string(),
                type_annotation: Some("Person".to_string()),
                init: Some(Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(18)),
                    }],
                }),
                is_const: false,
            },
            Stmt::StructMergeAssign {
                name: "p".to_string(),
                value: Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "missing".to_string(),
                        value: Expr::Literal(Literal::Number(20)),
                    }],
                },
            },
        ]);

        let mut checker = TypeChecker::new();
        let err = checker.check_program(&program).unwrap_err();
        assert!(err.contains("has no field"));
    }

    #[test]
    fn test_struct_field_type_reference_validation() {
        let program = Program::new(vec![Stmt::StructDecl {
            name: "Node".to_string(),
            fields: vec![StructFieldDecl {
                name: "next".to_string(),
                ty: "UnknownType".to_string(),
            }],
        }]);

        let mut checker = TypeChecker::new();
        let err = checker.check_program(&program).unwrap_err();
        assert!(err.contains("references unknown type"));
    }

    #[test]
    fn test_import_and_if_without_else_paths() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::Import {
                path: "./m.ziv".to_string(),
                modules: vec!["x".to_string()],
            })
            .unwrap();
        checker
            .check_stmt(&Stmt::If {
                condition: Expr::Literal(Literal::Boolean(true)),
                then_branch: vec![Stmt::Expression(Expr::Literal(Literal::Number(1)))],
                else_branch: None,
            })
            .unwrap();
    }

    #[test]
    fn test_struct_decl_duplicate_name_and_field_errors() {
        let mut checker = TypeChecker::new();
        let dup_field_err = checker
            .check_stmt(&Stmt::StructDecl {
                name: "Bad".to_string(),
                fields: vec![
                    StructFieldDecl {
                        name: "x".to_string(),
                        ty: "int".to_string(),
                    },
                    StructFieldDecl {
                        name: "x".to_string(),
                        ty: "int".to_string(),
                    },
                ],
            })
            .unwrap_err();
        assert!(dup_field_err.contains("Duplicate field"));

        checker
            .check_stmt(&Stmt::StructDecl {
                name: "Ok".to_string(),
                fields: vec![StructFieldDecl {
                    name: "x".to_string(),
                    ty: "int".to_string(),
                }],
            })
            .unwrap();
        let dup_name_err = checker
            .check_stmt(&Stmt::StructDecl {
                name: "Ok".to_string(),
                fields: vec![StructFieldDecl {
                    name: "y".to_string(),
                    ty: "int".to_string(),
                }],
            })
            .unwrap_err();
        assert!(dup_name_err.contains("already defined"));
    }

    #[test]
    fn test_assignment_and_struct_merge_error_paths() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::StructDecl {
                name: "Person".to_string(),
                fields: vec![StructFieldDecl {
                    name: "age".to_string(),
                    ty: "int".to_string(),
                }],
            })
            .unwrap();
        checker
            .check_stmt(&Stmt::StructDecl {
                name: "Other".to_string(),
                fields: vec![StructFieldDecl {
                    name: "v".to_string(),
                    ty: "int".to_string(),
                }],
            })
            .unwrap();

        let decl_mismatch = checker
            .check_stmt(&Stmt::VariableDecl {
                name: "typed".to_string(),
                type_annotation: Some("int".to_string()),
                init: Some(Expr::Literal(Literal::String("x".to_string()))),
                is_const: false,
            })
            .unwrap_err();
        assert!(decl_mismatch.contains("Type mismatch"));

        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: Some("int".to_string()),
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: false,
            })
            .unwrap();
        let assign_mismatch = checker
            .check_stmt(&Stmt::Assignment {
                name: "x".to_string(),
                value: Expr::Literal(Literal::String("x".to_string())),
            })
            .unwrap_err();
        assert!(assign_mismatch.contains("Type mismatch"));

        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "pc".to_string(),
                type_annotation: Some("Person".to_string()),
                init: Some(Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(1)),
                    }],
                }),
                is_const: true,
            })
            .unwrap();
        let merge_const_err = checker
            .check_stmt(&Stmt::StructMergeAssign {
                name: "pc".to_string(),
                value: Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(2)),
                    }],
                },
            })
            .unwrap_err();
        assert!(merge_const_err.contains("Cannot assign to constant"));

        let merge_non_struct = checker
            .check_stmt(&Stmt::StructMergeAssign {
                name: "x".to_string(),
                value: Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(2)),
                    }],
                },
            })
            .unwrap_err();
        assert!(merge_non_struct.contains("non-struct variable"));

        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "p".to_string(),
                type_annotation: Some("Person".to_string()),
                init: Some(Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(1)),
                    }],
                }),
                is_const: false,
            })
            .unwrap();
        let merge_type_mismatch = checker
            .check_stmt(&Stmt::StructMergeAssign {
                name: "p".to_string(),
                value: Expr::StructInit {
                    struct_name: "Other".to_string(),
                    fields: vec![StructFieldInit {
                        name: "v".to_string(),
                        value: Expr::Literal(Literal::Number(2)),
                    }],
                },
            })
            .unwrap_err();
        assert!(merge_type_mismatch.contains("Struct merge type mismatch"));

        let merge_non_init = checker
            .check_stmt(&Stmt::StructMergeAssign {
                name: "p".to_string(),
                value: Expr::Literal(Literal::Number(1)),
            })
            .unwrap_err();
        assert!(merge_non_init.contains("requires struct initializer"));
    }

    #[test]
    fn test_field_access_error_paths_and_any_path() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::StructDecl {
                name: "Person".to_string(),
                fields: vec![StructFieldDecl {
                    name: "age".to_string(),
                    ty: "int".to_string(),
                }],
            })
            .unwrap();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "p".to_string(),
                type_annotation: Some("Person".to_string()),
                init: Some(Expr::StructInit {
                    struct_name: "Person".to_string(),
                    fields: vec![StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(18)),
                    }],
                }),
                is_const: false,
            })
            .unwrap();
        let missing_field_err = checker
            .check_expr(&Expr::FieldAccess {
                object: Box::new(Expr::Identifier("p".to_string())),
                field: "missing".to_string(),
            })
            .unwrap_err();
        assert!(missing_field_err.contains("has no field"));

        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "any_var".to_string(),
                type_annotation: None,
                init: None,
                is_const: false,
            })
            .unwrap();
        let any_ty = checker
            .check_expr(&Expr::FieldAccess {
                object: Box::new(Expr::Identifier("any_var".to_string())),
                field: "x".to_string(),
            })
            .unwrap();
        assert_eq!(any_ty, Type::Any);

        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "n".to_string(),
                type_annotation: Some("int".to_string()),
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: false,
            })
            .unwrap();
        let non_struct_err = checker
            .check_expr(&Expr::FieldAccess {
                object: Box::new(Expr::Identifier("n".to_string())),
                field: "x".to_string(),
            })
            .unwrap_err();
        assert!(non_struct_err.contains("Cannot access field"));

        checker.symbol_table.define(Symbol::new(
            "ghost".to_string(),
            SymbolKind::Variable,
            Type::Struct("Ghost".to_string()),
            checker.symbol_table.current_scope_level(),
        ));
        let unknown_struct_err = checker
            .check_expr(&Expr::FieldAccess {
                object: Box::new(Expr::Identifier("ghost".to_string())),
                field: "x".to_string(),
            })
            .unwrap_err();
        assert!(unknown_struct_err.contains("Unknown struct"));
    }

    #[test]
    fn test_resolve_declared_type_and_struct_reference_success_path() {
        let mut checker = TypeChecker::new();
        let program = Program::new(vec![Stmt::StructDecl {
            name: "Node".to_string(),
            fields: vec![StructFieldDecl {
                name: "next".to_string(),
                ty: "Node".to_string(),
            }],
        }]);
        checker.check_program(&program).unwrap();

        assert_eq!(checker.resolve_declared_type("Node"), Type::Struct("Node".to_string()));
        assert_eq!(checker.resolve_declared_type("Unknown"), Type::Any);
    }

    #[test]
    fn test_struct_init_duplicate_and_type_mismatch_paths() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::StructDecl {
                name: "Person".to_string(),
                fields: vec![StructFieldDecl {
                    name: "age".to_string(),
                    ty: "int".to_string(),
                }],
            })
            .unwrap();

        let duplicate_err = checker
            .check_expr(&Expr::StructInit {
                struct_name: "Person".to_string(),
                fields: vec![
                    StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(1)),
                    },
                    StructFieldInit {
                        name: "age".to_string(),
                        value: Expr::Literal(Literal::Number(2)),
                    },
                ],
            })
            .unwrap_err();
        assert!(duplicate_err.contains("Duplicate field"));

        let type_mismatch_err = checker
            .check_expr(&Expr::StructInit {
                struct_name: "Person".to_string(),
                fields: vec![StructFieldInit {
                    name: "age".to_string(),
                    value: Expr::Literal(Literal::String("x".to_string())),
                }],
            })
            .unwrap_err();
        assert!(type_mismatch_err.contains("Type mismatch"));
    }
}

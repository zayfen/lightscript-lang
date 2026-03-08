use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;
use ziv::parser::ast::{Expr, Program, Stmt};
use ziv::parser::Parser;
use ziv::stdlib::Stdlib;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ziv")
}

fn stdlib_examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/stdlib")
}

fn stdlib_example_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(stdlib_examples_dir())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("ziv"))
        .collect();
    files.sort();
    files
}

fn expected_outputs(stem: &str) -> (&'static str, &'static str) {
    match stem {
        "hello" => ("Hello, Ziv!\n42\n30\n", ""),
        "io_demo" => ("io demo\n12\n0\nprompt> 0\n0\nfmt:10\n1\nio done\n", "err line\n"),
        "math_demo" => ("math demo\n10\n3\n7\n9\n1024\n5\n6\n7\nmath done\n", ""),
        "string_demo" => ("string demo\n3\n2\n3\n98\n65\n122\n1\n1\nstring done\n", ""),
        "array_demo" => ("array demo\n3\n10\n30\n20\n99\n30\n2\n99\n10\narray done\n", ""),
        "container_demo" => (
            "container demo\n0\n3\n15\n1\n10\n99\n20\n1\n0\n2\n100\n1\n200\n0\n300\n2\n2\n0\ncontainer done\n",
            "",
        ),
        "js_demo" => (
            "js demo\n42\n3\n0\n1\n12\n3\n0\n7\n2\n1\n1\n1\n1\n3\n2\n3\n1\n5\n3\n0\n4\n2\n14\njs done\n",
            "",
        ),
        "filesystem_demo" => (
            "filesystem demo\n1\n1\n3\n1\n5\n5\n1\n5\n1\n1\n1\n0\n1\n0\n1\n1\n0\n1\n1\nfilesystem done\n",
            "",
        ),
        "net_demo" => ("net demo\n5\n5\n8\n7\n8\n1\n1\n13\n1\n1\n1\n1\nnet done\n", ""),
        "crypto_demo" => ("crypto demo\n32\n40\n64\n128\n64\n64\n16\n5\n64\n1\n32\n36\ncrypto done\n", ""),
        "encoding_demo" => ("encoding demo\n4\n3\n6\n3\n5\n3\n3\n3\n8\n3\n8\n2\nencoding done\n", ""),
        _ => ("", ""),
    }
}

fn parse_program(source: &str) -> Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

fn collect_calls_in_expr(expr: &Expr, calls: &mut HashSet<String>) {
    match expr {
        Expr::Call { callee, args } => {
            calls.insert(callee.clone());
            for arg in args {
                collect_calls_in_expr(arg, calls);
            }
        }
        Expr::StructInit { fields, .. } => {
            for field in fields {
                collect_calls_in_expr(&field.value, calls);
            }
        }
        Expr::FieldAccess { object, .. } => collect_calls_in_expr(object, calls),
        Expr::Binary { left, right, .. } => {
            collect_calls_in_expr(left, calls);
            collect_calls_in_expr(right, calls);
        }
        Expr::Literal(_) | Expr::Identifier(_) => {}
    }
}

fn collect_calls_in_stmt(stmt: &Stmt, calls: &mut HashSet<String>) {
    match stmt {
        Stmt::Import { .. } => {}
        Stmt::StructDecl { .. } => {}
        Stmt::Expression(expr) => collect_calls_in_expr(expr, calls),
        Stmt::VariableDecl { init, .. } => {
            if let Some(expr) = init {
                collect_calls_in_expr(expr, calls);
            }
        }
        Stmt::Assignment { value, .. } => collect_calls_in_expr(value, calls),
        Stmt::StructMergeAssign { value, .. } => collect_calls_in_expr(value, calls),
        Stmt::FunctionDecl { body, .. } => {
            for stmt in body {
                collect_calls_in_stmt(stmt, calls);
            }
        }
        Stmt::Return(value) => {
            if let Some(expr) = value {
                collect_calls_in_expr(expr, calls);
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_calls_in_expr(condition, calls);
            for stmt in then_branch {
                collect_calls_in_stmt(stmt, calls);
            }
            if let Some(else_branch) = else_branch {
                for stmt in else_branch {
                    collect_calls_in_stmt(stmt, calls);
                }
            }
        }
        Stmt::While { condition, body } => {
            collect_calls_in_expr(condition, calls);
            for stmt in body {
                collect_calls_in_stmt(stmt, calls);
            }
        }
        Stmt::Block(stmts) => {
            for stmt in stmts {
                collect_calls_in_stmt(stmt, calls);
            }
        }
    }
}

#[test]
fn test_stdlib_examples_cover_all_registered_functions() {
    let files = stdlib_example_files();
    assert!(
        !files.is_empty(),
        "expected examples in {}, found none",
        stdlib_examples_dir().display()
    );

    let mut calls = HashSet::new();
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        let program = parse_program(&source);
        for stmt in &program.statements {
            collect_calls_in_stmt(stmt, &mut calls);
        }
    }

    let stdlib = Stdlib::new();
    let mut missing: Vec<String> = stdlib
        .all_functions()
        .into_iter()
        .map(|func| func.name.clone())
        .filter(|name| !calls.contains(name))
        .collect();
    missing.sort();

    assert!(
        missing.is_empty(),
        "stdlib functions missing from examples: {}",
        missing.join(", ")
    );
}

#[test]
fn test_stdlib_examples_compile_and_run() {
    let files = stdlib_example_files();
    let dir = tempdir().unwrap();

    for file in files {
        let stem = file.file_stem().unwrap().to_string_lossy().to_string();
        let output = dir.path().join(format!("{stem}_bin"));

        let compile = Command::new(bin())
            .arg(&file)
            .arg("-o")
            .arg(&output)
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            compile.status.success(),
            "failed to compile {}\nstdout:\n{}\nstderr:\n{}",
            file.display(),
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        assert!(
            output.exists(),
            "missing output binary for {}",
            file.display()
        );

        let run = Command::new(&output)
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            run.status.success(),
            "failed to run {}\nstdout:\n{}\nstderr:\n{}",
            output.display(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let (expected_stdout, expected_stderr) = expected_outputs(&stem);
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected_stdout,
            "unexpected stdout for {}",
            file.display()
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            expected_stderr,
            "unexpected stderr for {}",
            file.display()
        );
    }
}

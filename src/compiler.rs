//! LightLang compiler driver

use crate::codegen::ARM64Generator;
use crate::codegen::CodeGenerator;
use crate::codegen::CraneliftGenerator;
use crate::codegen::X86_64Generator;
use crate::ir::IRBuilder;
use crate::lexer::Lexer;
use crate::parser::ast::{Program, Stmt};
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub enum Target {
    X86_64,
    ARM64,
    Cranelift,
}

pub struct Compiler {
    output_name: String,
    keep_asm: bool,
    target: Target,
    assembler_cmd: String,
    linker_cmd: String,
    source_path: Option<PathBuf>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            output_name: "a.out".to_string(),
            keep_asm: false,
            target: Target::Cranelift, // Default to Cranelift for better code quality
            assembler_cmd: "as".to_string(),
            linker_cmd: "clang".to_string(),
            source_path: None,
        }
    }

    pub fn output(mut self, name: &str) -> Self {
        self.output_name = name.to_string();
        self
    }

    pub fn keep_asm(mut self, keep: bool) -> Self {
        self.keep_asm = keep;
        self
    }

    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn assembler(mut self, cmd: &str) -> Self {
        self.assembler_cmd = cmd.to_string();
        self
    }

    pub fn linker(mut self, cmd: &str) -> Self {
        self.linker_cmd = cmd.to_string();
        self
    }

    pub fn source_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.source_path = Some(path.as_ref().to_path_buf());
        self
    }

    fn top_level_symbol_name(stmt: &Stmt) -> Option<&str> {
        match stmt {
            Stmt::FunctionDecl { name, .. } => Some(name.as_str()),
            Stmt::VariableDecl { name, .. } => Some(name.as_str()),
            Stmt::StructDecl { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }

    fn resolve_import_path(&self, base_dir: &Path, import_path: &str) -> Result<PathBuf, String> {
        let candidate = if Path::new(import_path).is_absolute() {
            PathBuf::from(import_path)
        } else {
            base_dir.join(import_path)
        };

        fs::canonicalize(&candidate).map_err(|e| {
            format!(
                "Failed to resolve import path '{}' from '{}': {}",
                import_path,
                base_dir.display(),
                e
            )
        })
    }

    fn validate_imported_modules(
        &self,
        import_file: &Path,
        modules: &[String],
        imported_program: &Program,
    ) -> Result<(), String> {
        let available: HashSet<String> = imported_program
            .statements
            .iter()
            .filter_map(Self::top_level_symbol_name)
            .map(ToString::to_string)
            .collect();

        for module in modules {
            if !available.contains(module) {
                return Err(format!(
                    "Module '{}' not found in '{}'",
                    module,
                    import_file.display()
                ));
            }
        }

        Ok(())
    }

    fn load_module_program(
        &self,
        import_file: &Path,
        visiting: &mut HashSet<PathBuf>,
        cache: &mut HashMap<PathBuf, Program>,
    ) -> Result<Program, String> {
        let canonical = fs::canonicalize(import_file).map_err(|e| {
            format!(
                "Failed to canonicalize import file '{}': {}",
                import_file.display(),
                e
            )
        })?;

        if let Some(program) = cache.get(&canonical) {
            return Ok(program.clone());
        }

        if !visiting.insert(canonical.clone()) {
            return Err(format!(
                "Cyclic import detected at '{}'",
                canonical.display()
            ));
        }

        let result = (|| {
            let source = fs::read_to_string(&canonical).map_err(|e| {
                format!(
                    "Failed to read import file '{}': {}",
                    canonical.display(),
                    e
                )
            })?;

            let mut parser = Parser::new(&source);
            let parsed = parser.parse().map_err(|e| {
                format!(
                    "Parser error in imported file '{}': {}",
                    canonical.display(),
                    e
                )
            })?;

            let parent = canonical
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            let resolved = self.resolve_imports(parsed, &parent, visiting, cache)?;
            cache.insert(canonical.clone(), resolved.clone());
            Ok(resolved)
        })();

        visiting.remove(&canonical);
        result
    }

    fn resolve_imports(
        &self,
        program: Program,
        base_dir: &Path,
        visiting: &mut HashSet<PathBuf>,
        cache: &mut HashMap<PathBuf, Program>,
    ) -> Result<Program, String> {
        let mut statements = Vec::new();
        let mut imported_symbols = HashSet::new();

        for stmt in program.statements {
            match stmt {
                Stmt::Import { path, modules } => {
                    let import_file = self.resolve_import_path(base_dir, &path)?;
                    let imported_program =
                        self.load_module_program(&import_file, visiting, cache)?;
                    self.validate_imported_modules(&import_file, &modules, &imported_program)?;

                    for imported_stmt in imported_program.statements {
                        if let Some(name) = Self::top_level_symbol_name(&imported_stmt) {
                            if imported_symbols.insert(name.to_string()) {
                                statements.push(imported_stmt);
                            }
                        }
                    }
                }
                other => statements.push(other),
            }
        }

        Ok(Program::new(statements))
    }

    pub fn compile(&mut self, source: &str) -> Result<(), String> {
        // Step 1: Lexing
        println!("Step 1: Lexing");
        let mut lexer = Lexer::new(source);
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;
        println!("  ✓ Generated {} tokens", tokens.len());

        // Step 2: Parsing
        println!("\nStep 2: Parsing");
        let mut parser = Parser::new(source);
        let mut program = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
        let contains_imports = program
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Import { .. }));
        if contains_imports {
            let base_dir = self
                .source_path
                .as_ref()
                .and_then(|path| path.parent().map(Path::to_path_buf))
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            let mut visiting = HashSet::new();
            let mut cache = HashMap::new();
            program = self.resolve_imports(program, &base_dir, &mut visiting, &mut cache)?;
        }
        println!("  ✓ Parsed {} statements", program.statements.len());

        // Step 3: Semantic Analysis
        println!("\nStep 3: Semantic Analysis");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer
            .analyze(&program)
            .map_err(|e| format!("Semantic error: {}", e))?;
        println!("  ✓ Semantic analysis passed");

        // Step 4: IR Generation
        println!("\nStep 4: IR Generation");
        let builder = IRBuilder::new();
        let module = builder.build(&program);
        println!("  ✓ Generated IR with {} functions", module.functions.len());

        // Step 5: Code Generation
        println!("\nStep 5: Code Generation");

        let obj_file = format!("{}.o", self.output_name);

        match self.target {
            Target::Cranelift => {
                let gen = CraneliftGenerator::new()?;

                let obj_bytes = gen.compile_to_object(&module)?;

                fs::write(&obj_file, &obj_bytes)
                    .map_err(|e| format!("Failed to write object file: {}", e))?;

                println!("  ✓ Generated {} bytes of object code", obj_bytes.len());

                // Detect architecture and generate appropriate start helper
                // On macOS, the entry point is _main, so we create a wrapper
                // that calls our __user_main and returns to the C runtime.
                #[cfg(target_arch = "aarch64")]
                let start_asm = r#"
.text
.globl _main
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp
    bl __user_main
    ldp x29, x30, [sp], #16
    ret
"#;
                #[cfg(target_arch = "x86_64")]
                let start_asm = r#"
.text
.globl _main
_main:
    pushq %rbp
    movq %rsp, %rbp
    call __user_main
    popq %rbp
    ret
"#;

                let start_asm_file = format!("{}_start.s", self.output_name);
                fs::write(&start_asm_file, start_asm)
                    .map_err(|e| format!("Failed to write start assembly: {}", e))?;

                let start_obj_file = format!("{}_start.o", self.output_name);
                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                assembler.arg("-arch").arg("arm64");
                #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
                assembler.arg("-arch").arg("x86_64");
                let status = assembler
                    .arg("-o")
                    .arg(&start_obj_file)
                    .arg(&start_asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly of start helper failed".to_string());
                }

                // Build stdlib runtime object that provides callable built-ins.
                let runtime_c_file = format!("{}_stdlib_runtime.c", self.output_name);
                let runtime_obj_file = format!("{}_stdlib_runtime.o", self.output_name);
                let runtime_c = r#"
#include <stdint.h>
#include <stdio.h>

int64_t ziv_print_i64(int64_t value) {
    printf("%lld", (long long)value);
    return 0;
}

int64_t ziv_println_i64(int64_t value) {
    printf("%lld\n", (long long)value);
    return 0;
}

int64_t ziv_print_str(const char* value) {
    if (value != NULL) {
        fputs(value, stdout);
    }
    return 0;
}

int64_t ziv_println_str(const char* value) {
    if (value != NULL) {
        fputs(value, stdout);
    }
    fputc('\n', stdout);
    return 0;
}
"#;
                fs::write(&runtime_c_file, runtime_c)
                    .map_err(|e| format!("Failed to write stdlib runtime source: {}", e))?;

                let status = Command::new(&self.linker_cmd)
                    .arg("-c")
                    .arg(&runtime_c_file)
                    .arg("-o")
                    .arg(&runtime_obj_file)
                    .status()
                    .map_err(|e| format!("Failed to run linker: {}", e))?;
                if !status.success() {
                    return Err("Compilation of stdlib runtime failed".to_string());
                }

                // Link with both object files
                let status = Command::new(&self.linker_cmd)
                    .arg("-o")
                    .arg(&self.output_name)
                    .arg(&obj_file)
                    .arg(&start_obj_file)
                    .arg(&runtime_obj_file)
                    .status()
                    .map_err(|e| format!("Failed to run linker: {}", e))?;

                if !status.success() {
                    return Err("Linking failed".to_string());
                }
                println!("  ✓ Linked to executable {}", self.output_name);

                // Cleanup
                if !self.keep_asm {
                    fs::remove_file(&start_asm_file).ok();
                    fs::remove_file(&start_obj_file).ok();
                    fs::remove_file(&runtime_c_file).ok();
                    fs::remove_file(&runtime_obj_file).ok();
                    fs::remove_file(&obj_file).ok();
                    println!("  ✓ Cleaned up temporary files");
                } else {
                    fs::remove_file(&runtime_c_file).ok();
                }

                println!("\n✅ Compilation successful!");
                println!("   Run with: ./{}", self.output_name);

                return Ok(());
            }

            Target::X86_64 => {
                let mut gen = X86_64Generator::new();
                let asm = gen.generate(&module)?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(target_os = "macos")]
                assembler.arg("-arch").arg("x86_64");
                let status = assembler
                    .arg("-o")
                    .arg(&obj_file)
                    .arg(&asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly failed".to_string());
                }
                println!("  ✓ Assembled to {}", obj_file);

                if !self.keep_asm {
                    fs::remove_file(&asm_file).ok();
                }
            }

            Target::ARM64 => {
                let mut gen = ARM64Generator::new();
                let asm = gen.generate(&module)?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(target_os = "macos")]
                assembler.arg("-arch").arg("arm64");
                let status = assembler
                    .arg("-o")
                    .arg(&obj_file)
                    .arg(&asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly failed".to_string());
                }
                println!("  ✓ Assembled to {}", obj_file);

                if !self.keep_asm {
                    fs::remove_file(&asm_file).ok();
                }
            }
        }

        println!("  ✓ Object file written to {}", obj_file);

        // Step 6: Link to executable
        let mut linker = Command::new(&self.linker_cmd);
        #[cfg(target_os = "macos")]
        if matches!(self.target, Target::X86_64) {
            linker.arg("-arch").arg("x86_64");
        }
        let status = linker
            .arg("-o")
            .arg(&self.output_name)
            .arg(&obj_file)
            .status()
            .map_err(|e| format!("Failed to run linker: {}", e))?;

        if !status.success() {
            return Err("Linking failed".to_string());
        }
        println!("  ✓ Linked to executable {}", self.output_name);

        // Cleanup
        if !self.keep_asm {
            fs::remove_file(&obj_file).ok();
            println!("  ✓ Cleaned up temporary files");
        }

        println!("\n✅ Compilation successful!");
        println!("   Run with: ./{}", self.output_name);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Expr, Literal, Param};
    use tempfile::tempdir;

    fn make_function(name: &str) -> Stmt {
        Stmt::FunctionDecl {
            name: name.to_string(),
            params: vec![Param {
                name: "x".to_string(),
                type_annotation: None,
            }],
            return_type: None,
            body: vec![Stmt::Return(Some(Expr::Identifier("x".to_string())))],
        }
    }

    fn make_variable(name: &str, value: i64) -> Stmt {
        Stmt::VariableDecl {
            name: name.to_string(),
            type_annotation: None,
            init: Some(Expr::Literal(Literal::Number(value))),
            is_const: false,
        }
    }

    fn make_struct(name: &str) -> Stmt {
        Stmt::StructDecl {
            name: name.to_string(),
            fields: vec![crate::parser::ast::StructFieldDecl {
                name: "x".to_string(),
                ty: "int".to_string(),
            }],
        }
    }

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new();
        assert_eq!(compiler.output_name, "a.out");
    }

    #[test]
    fn test_compiler_builder_methods() {
        let compiler = Compiler::new()
            .output("out.bin")
            .keep_asm(true)
            .target(Target::ARM64)
            .assembler("my-as")
            .linker("my-linker");
        assert_eq!(compiler.output_name, "out.bin");
        assert!(compiler.keep_asm);
        assert_eq!(
            std::mem::discriminant(&compiler.target),
            std::mem::discriminant(&Target::ARM64)
        );
        assert_eq!(compiler.assembler_cmd, "my-as");
        assert_eq!(compiler.linker_cmd, "my-linker");
    }

    #[test]
    fn test_source_path_and_top_level_symbol_helpers() {
        let compiler = Compiler::new().source_path("main.ziv");
        assert_eq!(compiler.source_path, Some(PathBuf::from("main.ziv")));

        let func = make_function("f");
        let var = make_variable("v", 1);
        let strukt = make_struct("S");
        let expr = Stmt::Expression(Expr::Literal(Literal::Number(1)));

        assert_eq!(Compiler::top_level_symbol_name(&func), Some("f"));
        assert_eq!(Compiler::top_level_symbol_name(&var), Some("v"));
        assert_eq!(Compiler::top_level_symbol_name(&strukt), Some("S"));
        assert_eq!(Compiler::top_level_symbol_name(&expr), None);
    }

    #[test]
    fn test_resolve_import_path_relative_absolute_and_error() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("module.ziv");
        fs::write(&module, "function add(a, b) { return a + b; }").unwrap();

        let compiler = Compiler::new();
        let rel = compiler
            .resolve_import_path(dir.path(), "module.ziv")
            .unwrap();
        let abs = fs::canonicalize(&module).unwrap();
        assert_eq!(rel, abs);

        let abs2 = compiler
            .resolve_import_path(dir.path(), abs.to_string_lossy().as_ref())
            .unwrap();
        assert_eq!(abs2, abs);

        let err = compiler
            .resolve_import_path(dir.path(), "missing.ziv")
            .unwrap_err();
        assert!(err.contains("Failed to resolve import path"));
    }

    #[test]
    fn test_validate_imported_modules_success_and_failure() {
        let compiler = Compiler::new();
        let program = Program::new(vec![
            make_function("add"),
            make_variable("value", 7),
            Stmt::Expression(Expr::Literal(Literal::Number(1))),
        ]);

        assert!(
            compiler
                .validate_imported_modules(
                    Path::new("module.ziv"),
                    &["add".to_string(), "value".to_string()],
                    &program
                )
                .is_ok()
        );

        let err = compiler
            .validate_imported_modules(Path::new("module.ziv"), &["missing".to_string()], &program)
            .unwrap_err();
        assert!(err.contains("Module 'missing' not found"));
    }

    #[test]
    fn test_load_module_program_cache_and_cycle_paths() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("module.ziv");
        fs::write(&module, "function add(a, b) { return a + b; }").unwrap();
        let canonical = fs::canonicalize(&module).unwrap();

        let compiler = Compiler::new();
        let cached = Program::new(vec![make_variable("cached", 1)]);
        let mut cache = HashMap::new();
        cache.insert(canonical.clone(), cached.clone());
        let mut visiting = HashSet::new();

        let got = compiler
            .load_module_program(&module, &mut visiting, &mut cache)
            .unwrap();
        assert_eq!(got, cached);

        let mut visiting = HashSet::new();
        visiting.insert(canonical);
        let err = compiler
            .load_module_program(&module, &mut visiting, &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Cyclic import detected"));
    }

    #[test]
    fn test_load_module_program_read_and_parse_errors() {
        let dir = tempdir().unwrap();
        let compiler = Compiler::new();

        let missing = dir.path().join("missing.ziv");
        let err = compiler
            .load_module_program(&missing, &mut HashSet::new(), &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Failed to canonicalize import file"));

        let dir_as_file = dir.path().join("not_file.ziv");
        fs::create_dir(&dir_as_file).unwrap();
        let err = compiler
            .load_module_program(&dir_as_file, &mut HashSet::new(), &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Failed to read import file"));

        let bad = dir.path().join("bad.ziv");
        fs::write(&bad, "/").unwrap();
        let err = compiler
            .load_module_program(&bad, &mut HashSet::new(), &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Parser error in imported file"));
    }

    #[test]
    fn test_resolve_imports_dedup_and_skip_non_symbol_statements() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("module.ziv");
        fs::write(
            &module,
            r#"
            function add(a, b) { return a + b; }
            let value = 7;
            1;
            "#,
        )
        .unwrap();

        let program = Program::new(vec![
            Stmt::Import {
                path: "module.ziv".to_string(),
                modules: vec!["add".to_string(), "value".to_string()],
            },
            Stmt::Import {
                path: "module.ziv".to_string(),
                modules: vec!["add".to_string()],
            },
            Stmt::Expression(Expr::Call {
                callee: "add".to_string(),
                args: vec![
                    Expr::Literal(Literal::Number(1)),
                    Expr::Literal(Literal::Number(2)),
                ],
            }),
        ]);

        let compiler = Compiler::new();
        let resolved = compiler
            .resolve_imports(program, dir.path(), &mut HashSet::new(), &mut HashMap::new())
            .unwrap();

        let mut add_count = 0;
        let mut value_count = 0;
        for stmt in &resolved.statements {
            match stmt {
                Stmt::FunctionDecl { name, .. } if name == "add" => add_count += 1,
                Stmt::VariableDecl { name, .. } if name == "value" => value_count += 1,
                _ => {}
            }
        }

        assert_eq!(add_count, 1);
        assert_eq!(value_count, 1);
        assert!(matches!(
            resolved.statements.last(),
            Some(Stmt::Expression(Expr::Call { callee, .. })) if callee == "add"
        ));
    }

    #[test]
    fn test_resolve_imports_can_include_struct_symbols() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("types.ziv");
        fs::write(
            &module,
            r#"
            struct Person { age: int; score: int; }
            function mk(a, b) { return a + b; }
            "#,
        )
        .unwrap();

        let program = Program::new(vec![Stmt::Import {
            path: "types.ziv".to_string(),
            modules: vec!["Person".to_string()],
        }]);

        let compiler = Compiler::new();
        let resolved = compiler
            .resolve_imports(program, dir.path(), &mut HashSet::new(), &mut HashMap::new())
            .unwrap();

        assert!(resolved
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Stmt::StructDecl { name, .. } if name == "Person")));
    }

    #[test]
    fn test_compile_import_without_source_path_uses_current_dir_for_base() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("abs_import_module.ziv");
        fs::write(&module, "function add(a, b) { return a + b; }").unwrap();
        let module_abs = fs::canonicalize(&module).unwrap();

        let source = format!(
            "from \"{}\" import {{ add }}; println(add(1, 2));",
            module_abs.to_string_lossy()
        );
        let output = dir.path().join("abs_import_bin");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        compiler.compile(&source).unwrap();

        assert!(output.exists());
    }

    #[test]
    fn test_compile_lexer_error() {
        let huge = format!("let x = {};", "9".repeat(200));
        let mut compiler = Compiler::new().output("lexer_err_bin");
        let err = compiler.compile(&huge).unwrap_err();
        assert!(err.contains("Lexer error"));
        fs::remove_file("lexer_err_bin").ok();
    }

    #[test]
    fn test_compile_parser_and_semantic_errors() {
        let mut parser_err = Compiler::new().output("parser_err_bin");
        let err = parser_err.compile("/").unwrap_err();
        assert!(err.contains("Parser error"));

        let mut semantic_err = Compiler::new().output("semantic_err_bin");
        let err = semantic_err.compile("let y = x;").unwrap_err();
        assert!(err.contains("Semantic error"));
    }

    #[test]
    fn test_compile_cranelift_success_and_cleanup() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_ok");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        compiler.compile("let x = 1; let y = x + 2;").unwrap();

        assert!(output.exists());
        assert!(!dir.path().join("cranelift_ok.o").exists());
        assert!(!dir.path().join("cranelift_ok_start.s").exists());
        assert!(!dir.path().join("cranelift_ok_start.o").exists());
        assert!(!dir.path().join("cranelift_ok_stdlib_runtime.c").exists());
        assert!(!dir.path().join("cranelift_ok_stdlib_runtime.o").exists());
    }

    #[test]
    fn test_compile_cranelift_keep_asm() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_keep");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).keep_asm(true);
        compiler.compile("let x = 1;").unwrap();

        assert!(output.exists());
        assert!(dir.path().join("cranelift_keep.o").exists());
        assert!(dir.path().join("cranelift_keep_start.s").exists());
        assert!(dir.path().join("cranelift_keep_start.o").exists());
        assert!(dir.path().join("cranelift_keep_stdlib_runtime.o").exists());
        assert!(!dir.path().join("cranelift_keep_stdlib_runtime.c").exists());
    }

    #[test]
    fn test_compile_cranelift_write_object_failure() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing").join("out");
        let output_str = missing.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to write object file"));
    }

    #[test]
    fn test_compile_cranelift_link_failure_with_directory_output() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Linking failed"));

        let obj = format!("{}.o", output_str);
        let start_s = format!("{}_start.s", output_str);
        let start_o = format!("{}_start.o", output_str);
        let runtime_c = format!("{}_stdlib_runtime.c", output_str);
        let runtime_o = format!("{}_stdlib_runtime.o", output_str);
        fs::remove_file(obj).ok();
        fs::remove_file(start_s).ok();
        fs::remove_file(start_o).ok();
        fs::remove_file(runtime_c).ok();
        fs::remove_file(runtime_o).ok();
    }

    #[test]
    fn test_compile_cranelift_start_helper_assembly_failure() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_start_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).assembler("false");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Assembly of start helper failed"));
    }

    #[test]
    fn test_compile_cranelift_start_helper_spawn_error() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_start_spawn_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .assembler("__ziv_missing_assembler__");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_cranelift_linker_spawn_error() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_link_spawn_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .linker("__ziv_missing_linker__");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to run linker"));
    }

    #[test]
    fn test_compile_cranelift_runtime_compile_failure() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_runtime_compile_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).linker("false");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Compilation of stdlib runtime failed"));
    }

    #[test]
    fn test_compile_arm64_success() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("arm_ok");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).target(Target::ARM64);
        compiler.compile("function main() { return 0; }").unwrap();
        assert!(output.exists());
    }

    #[test]
    fn test_compile_arm64_success_keep_asm() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("arm_keep");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::ARM64)
            .keep_asm(true);
        compiler.compile("function main() { return 0; }").unwrap();

        assert!(output.exists());
        assert!(dir.path().join("arm_keep.o").exists());
        assert!(dir.path().join("arm_keep.s").exists());
    }

    #[test]
    fn test_compile_arm64_and_x86_assembly_failures() {
        let dir = tempdir().unwrap();
        let arm_out = dir.path().join("arm_fail");
        let arm_out_str = arm_out.to_string_lossy().to_string();
        let mut arm_compiler = Compiler::new().output(&arm_out_str).target(Target::ARM64);
        let arm_err = arm_compiler.compile("let x = 2 * 3;").unwrap_err();
        assert!(arm_err.contains("Assembly failed") | arm_err.contains("Failed to run assembler"));

        let x86_out = dir.path().join("x86_fail");
        let x86_out_str = x86_out.to_string_lossy().to_string();
        let mut x86_compiler = Compiler::new().output(&x86_out_str).target(Target::X86_64);
        let x86_err = x86_compiler
            .compile("while (1) { let y = 1; }")
            .unwrap_err();
        assert!(x86_err.contains("Assembly failed") | x86_err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_x86_assembly_success_then_link_failure_and_cleanup() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).target(Target::X86_64);
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Linking failed"));

        let asm_file = format!("{}.s", output_str);
        assert!(!std::path::Path::new(&asm_file).exists());
    }

    #[test]
    fn test_compile_x86_assembly_success_then_link_failure_keep_asm() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::X86_64)
            .keep_asm(true);
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Linking failed"));

        let asm_file = format!("{}.s", output_str);
        assert!(std::path::Path::new(&asm_file).exists());
    }

    #[test]
    fn test_compile_x86_and_arm64_assembler_spawn_errors() {
        let dir = tempdir().unwrap();

        let x86_out = dir.path().join("x86_spawn_fail");
        let x86_out_str = x86_out.to_string_lossy().to_string();
        let mut x86_compiler = Compiler::new()
            .output(&x86_out_str)
            .target(Target::X86_64)
            .assembler("__ziv_missing_assembler__");
        let x86_err = x86_compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(x86_err.contains("Failed to run assembler"));

        let arm_out = dir.path().join("arm_spawn_fail");
        let arm_out_str = arm_out.to_string_lossy().to_string();
        let mut arm_compiler = Compiler::new()
            .output(&arm_out_str)
            .target(Target::ARM64)
            .assembler("__ziv_missing_assembler__");
        let arm_err = arm_compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(arm_err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_arm64_linker_spawn_error() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("arm_link_spawn_fail");
        let out_str = out.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&out_str)
            .target(Target::ARM64)
            .linker("__ziv_missing_linker__");
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Failed to run linker"));
    }
}

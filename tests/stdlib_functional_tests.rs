use tempfile::tempdir;
use ziv::compiler::Compiler;
use ziv::ir::{IRBuilder, IRInstruction};
use ziv::parser::Parser;
use ziv::semantic::SemanticAnalyzer;
use ziv::stdlib::Stdlib;

fn parse(source: &str) -> ziv::parser::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

fn literal_for_param_type(ty: &str) -> &'static str {
    match ty {
        "string" => "\"x\"",
        "i64" | "f64" | "int" | "number" => "1",
        "bool" => "1",
        "array" | "function" | "any" | "char" => "0",
        _ => "0",
    }
}

fn build_stdlib_call_source() -> String {
    let stdlib = Stdlib::new();
    let mut funcs = stdlib.all_functions();
    funcs.sort_by(|a, b| a.name.cmp(&b.name));

    funcs
        .into_iter()
        .map(|func| {
            let args = func
                .params
                .iter()
                .map(|param| literal_for_param_type(&param.ty))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({});", func.name, args)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_stdlib_registry_surface_is_complete() {
    let stdlib = Stdlib::new();
    let expected = [
        "print",
        "println",
        "read",
        "eprint",
        "eprintln",
        "input",
        "readAll",
        "printf",
        "flush",
        "abs",
        "min",
        "max",
        "sqrt",
        "pow",
        "floor",
        "ceil",
        "round",
        "strlen",
        "concat",
        "substr",
        "char_at",
        "to_upper",
        "to_lower",
        "trim",
        "contains",
        "push",
        "pop",
        "arrlen",
        "get",
        "set",
        "first",
        "last",
        "reverse",
        "parseInt",
        "parseFloat",
        "isNaN",
        "isFinite",
        "Number",
        "String",
        "Boolean",
        "jsonParse",
        "jsonStringify",
        "includes",
        "indexOf",
        "startsWith",
        "endsWith",
        "split",
        "replace",
        "map",
        "filter",
        "reduce",
        "readFile",
        "writeFile",
        "appendFile",
        "exists",
        "mkdir",
        "readDir",
        "removeFile",
        "removeDir",
        "rename",
        "copyFile",
        "fileSize",
        "cwd",
        "fetch",
        "httpGet",
        "httpPost",
        "httpPut",
        "httpDelete",
        "download",
        "upload",
        "websocketConnect",
        "dnsLookup",
        "ping",
        "md5",
        "sha1",
        "sha256",
        "sha512",
        "hmacSha256",
        "pbkdf2",
        "encryptAES",
        "decryptAES",
        "sign",
        "verify",
        "randomBytes",
        "randomUUID",
        "base64Encode",
        "base64Decode",
        "hexEncode",
        "hexDecode",
        "urlEncode",
        "urlDecode",
        "utf8Encode",
        "utf8Decode",
        "csvEncode",
        "csvDecode",
        "queryStringify",
        "queryParse",
    ];

    for name in expected {
        assert!(stdlib.is_builtin(name), "missing builtin: {name}");
    }
    assert_eq!(expected.len(), stdlib.all_functions().len());
}

#[test]
fn test_semantic_accepts_cross_module_stdlib_calls() {
    let source = build_stdlib_call_source();
    let program = parse(&source);

    let mut analyzer = SemanticAnalyzer::new();
    assert!(analyzer.analyze(&program).is_ok());
}

#[test]
fn test_ir_builder_lowers_print_calls_and_skips_other_builtins() {
    let program = parse(
        r#"
        print(1);
        println("x");
        abs(2);
        strlen("abc");
        push(0, 1);
        "#,
    );

    let module = IRBuilder::new().build(&program);
    let main = module
        .functions
        .iter()
        .find(|func| func.name == "_user_main")
        .unwrap();

    let has_runtime_print = main.instructions.iter().any(|instr| match instr {
        IRInstruction::Call { function, .. } => function == "ziv_print_i64",
        _ => false,
    });
    let has_runtime_println_str = main.instructions.iter().any(|instr| match instr {
        IRInstruction::Call { function, .. } => function == "ziv_println_str",
        _ => false,
    });
    let has_skipped_builtin_call = main.instructions.iter().any(|instr| match instr {
        IRInstruction::Call { function, .. } => {
            matches!(function.as_str(), "abs" | "strlen" | "push")
        }
        _ => false,
    });
    assert!(has_runtime_print);
    assert!(has_runtime_println_str);
    assert!(!has_skipped_builtin_call);
}

#[test]
fn test_ir_builder_preserves_shadowed_builtin_calls() {
    let program = parse(
        r#"
        function print(x) { return x; }
        function abs(x) { return x; }
        print(1);
        abs(2);
        println(3);
        "#,
    );

    let module = IRBuilder::new().build(&program);
    let main = module
        .functions
        .iter()
        .find(|func| func.name == "_user_main")
        .unwrap();

    let has_user_print = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "print"
        )
    });
    let has_user_abs = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "abs"
        )
    });
    let has_builtin_println = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "println"
        )
    });
    let has_runtime_println = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "ziv_println_i64"
        )
    });

    assert!(has_user_print);
    assert!(has_user_abs);
    assert!(!has_builtin_println);
    assert!(has_runtime_println);
}

#[test]
fn test_compiler_can_compile_program_with_stdlib_calls() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("stdlib_ok");
    let output_str = output.to_string_lossy().to_string();

    let source = build_stdlib_call_source();

    let mut compiler = Compiler::new().output(&output_str);
    compiler.compile(&source).unwrap();

    assert!(output.exists());
}

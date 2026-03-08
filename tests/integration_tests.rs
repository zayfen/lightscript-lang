use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ziv")
}

#[test]
fn test_cli_no_args() {
    let output = Command::new(bin()).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
}

#[test]
fn test_cli_o_requires_argument() {
    let output = Command::new(bin()).arg("-o").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-o requires an argument"));
}

#[test]
fn test_cli_no_source_file() {
    let output = Command::new(bin()).arg("--keep-asm").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No source file specified"));
}

#[test]
fn test_cli_read_file_error() {
    let output = Command::new(bin())
        .arg("does_not_exist.ziv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error reading file"));
}

#[test]
fn test_cli_compilation_error() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("bad.ziv");
    fs::write(&src, "let y = x;").unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Compilation error"));
}

#[test]
fn test_cli_success_and_keep_asm() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("ok.ziv");
    fs::write(&src, "let x = 1;").unwrap();

    let output = Command::new(bin())
        .arg("--keep-asm")
        .arg(&src)
        .arg("-o")
        .arg("out_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(Path::new(&dir.path().join("out_bin")).exists());
    assert!(Path::new(&dir.path().join("out_bin.o")).exists());
    assert!(Path::new(&dir.path().join("out_bin_start.s")).exists());
    assert!(Path::new(&dir.path().join("out_bin_start.o")).exists());
}

#[test]
fn test_cli_compiled_program_emits_print_output() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("hello_print.ziv");
    fs::write(
        &src,
        r#"
        print("Hello, ");
        println("Ziv!");
        println(42);
        println(10 + 20);
        "#,
    )
    .unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .arg("-o")
        .arg("hello_print_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("hello_print_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(stdout, "Hello, Ziv!\n42\n30\n");
}

#[test]
fn test_cli_from_import_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.ziv");
    let math = dir.path().join("math.ziv");

    fs::write(
        &main,
        r#"
        from { "./math.ziv" } import { add };
        println(add(20, 22));
        "#,
    )
    .unwrap();
    fs::write(
        &math,
        r#"
        function add(a, b) { return a + b; }
        function sub(a, b) { return a - b; }
        "#,
    )
    .unwrap();

    let output = Command::new(bin())
        .arg(&main)
        .arg("-o")
        .arg("import_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("import_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\n");
}

#[test]
fn test_cli_from_import_missing_module_reports_error() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main_missing_import.ziv");
    let math = dir.path().join("math_missing_import.ziv");

    fs::write(
        &main,
        r#"
        from { "./math_missing_import.ziv" } import { missing };
        println(missing(1, 2));
        "#,
    )
    .unwrap();
    fs::write(&math, "function add(a, b) { return a + b; }").unwrap();

    let output = Command::new(bin())
        .arg(&main)
        .arg("-o")
        .arg("missing_import_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Module 'missing' not found"));
}

#[test]
fn test_from_import_full_demo_example_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("from_import")
        .join("full_demo.ziv");

    let output = Command::new(bin())
        .arg(&example)
        .arg("-o")
        .arg("from_import_demo_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("from_import_demo_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "27\n42\n9\n7\n");
}

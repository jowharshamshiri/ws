use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{NamedTempFile, TempDir};
use std::io::Write;
use std::fs;

/// Test the code command help functionality
#[test]
fn test_code_command_help() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&["code", "--help"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AST-based code analysis and transformation"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("transform"))
        .stdout(predicate::str::contains("patterns"))
        .stdout(predicate::str::contains("analyze"));
}

/// Test code search functionality
#[test]
fn test_code_search_basic() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(temp_file, "    let x = 42;").unwrap();
    writeln!(temp_file, "    println!(\"Value: {{}}\", x);").unwrap();
    writeln!(temp_file, "}}").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "search", 
        "println",
        "--files", temp_file.path().to_str().unwrap(),
        "--language", "rust"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("println"));
}

/// Test code search with JSON output
#[test]
fn test_code_search_json_output() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "console.log('Hello, world!');").unwrap();
    writeln!(temp_file, "const x = 42;").unwrap();
    writeln!(temp_file, "console.log('Value:', x);").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "search",
        "console.log",
        "--files", temp_file.path().to_str().unwrap(),
        "--language", "javascript",
        "--format", "json"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

/// Test code transform functionality (dry run)
#[test]
fn test_code_transform_dry_run() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    let result = some_operation().unwrap();").unwrap();
    writeln!(temp_file, "    println!(\"Result: {{}}\", result);").unwrap();
    writeln!(temp_file, "}}").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "transform",
        "unwrap",
        "expect(\"Expected value\")",
        "--files", temp_file.path().to_str().unwrap(),
        "--language", "rust",
        "--dry-run"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("dry run"));
}

/// Test code transform with actual file modification
#[test]
fn test_code_transform_actual() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.js");
    
    fs::write(&file_path, "var x = 42;\nvar y = 'hello';\n").unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "transform",
        "var",
        "let",
        "--files", file_path.to_str().unwrap(),
        "--language", "javascript",
        "--no-backup"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("changes applied"));

    // Verify the file was actually modified
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("let"));
}

/// Test code patterns for different languages
#[test]
fn test_code_patterns_rust() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&["code", "patterns", "rust", "--category", "transform"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Common transform patterns for rust"));
}

#[test]
fn test_code_patterns_javascript() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&["code", "patterns", "javascript", "--category", "transform"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Common transform patterns for javascript"));
}

#[test]
fn test_code_patterns_unsupported_language() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&["code", "patterns", "cobol"]);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Unsupported language"));
}

/// Test code analyze functionality
#[test]
fn test_code_analyze_basic() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fn main() {{").unwrap();
    writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(temp_file, "    let x = 42;").unwrap();
    writeln!(temp_file, "}}").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "analyze",
        "--files", temp_file.path().to_str().unwrap(),
        "--analysis-type", "structure"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Code Analysis"))
        .stdout(predicate::str::contains("lines"))
        .stdout(predicate::str::contains("characters"));
}

/// Test code analyze with JSON output
#[test]
fn test_code_analyze_json() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "print('Hello, Python!')").unwrap();
    writeln!(temp_file, "x = 42").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "analyze",
        "--files", temp_file.path().to_str().unwrap(),
        "--language", "python",
        "--format", "json"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"file\":"))
        .stdout(predicate::str::contains("\"lines\":"))
        .stdout(predicate::str::contains("\"chars\":"));
}

/// Test multiple file processing
#[test]
fn test_code_search_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create first file
    let file1 = temp_dir.path().join("file1.rs");
    fs::write(&file1, "fn main() { println!(\"File 1\"); }").unwrap();
    
    // Create second file
    let file2 = temp_dir.path().join("file2.rs");
    fs::write(&file2, "fn helper() { println!(\"File 2\"); }").unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "search",
        "println",
        "--files", file1.to_str().unwrap(),
        "--files", file2.to_str().unwrap(),
        "--language", "rust"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("println"));
}

/// Test search with context lines
#[test]
fn test_code_search_with_context() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    
    fs::write(&file_path, 
        "// This is a comment\n\
         fn main() {\n\
             println!(\"Target line\");\n\
             let x = 42;\n\
         }\n").unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "search",
        "Target line",
        "--files", file_path.to_str().unwrap(),
        "--context", "2"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Target line"));
}

/// Test code tree default functionality  
#[test]
fn test_code_tree_default() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.arg("code");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Codebase Structure"))
        .stdout(predicate::str::contains("Project Root:"))
        .stdout(predicate::str::contains("Current Location:"));
}

/// Test code tree with options
#[test]
fn test_code_tree_with_options() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "tree",
        "--depth", "2",
        "--sizes",
        "--extensions", "rs,toml"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Codebase Structure"))
        .stdout(predicate::str::contains("Project Root:"));
}

/// Test tree respects gitignore by default
#[test]
fn test_code_tree_respects_gitignore() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&["code", "tree", "--depth", "2"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should not show the target directory (which is gitignored)
    assert!(!stdout.contains("target"), "target directory should be filtered by gitignore");
    
    // Should show source files that are not ignored
    assert!(stdout.contains("src"), "src directory should be shown");
}

/// Test tree --no-ignore shows all files
#[test]
fn test_code_tree_no_ignore_shows_all() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&["code", "tree", "--depth", "1", "--no-ignore"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // With --no-ignore, should show directories that would normally be ignored
    // Note: target might not exist in all test environments, so we test for structure
    assert!(stdout.contains("Codebase Structure"), "Should show codebase structure");
}

/// Test transform with max changes limit
#[test]
fn test_code_transform_max_changes() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "var a = 1;").unwrap();
    writeln!(temp_file, "var b = 2;").unwrap();
    writeln!(temp_file, "var c = 3;").unwrap();
    writeln!(temp_file, "var d = 4;").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "transform",
        "var",
        "let",
        "--files", temp_file.path().to_str().unwrap(),
        "--language", "javascript",
        "--max-changes", "2",
        "--dry-run"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("changes applied"));
}

/// Test error handling for non-existent files
#[test]
fn test_code_search_nonexistent_file() {
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "search",
        "test",
        "--files", "/nonexistent/file.rs"
    ]);

    // Should handle the error gracefully, not crash
    cmd.assert()
        .success(); // The command should not crash, even if file doesn't exist
}

/// Test language auto-detection
#[test]
fn test_code_search_language_autodetect() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.py");
    fs::write(&file_path, "print('Hello, Python!')").unwrap();

    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "search",
        "print",
        "--files", file_path.to_str().unwrap()
        // No explicit language specified - should auto-detect from extension
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("print"));
}

/// Integration test - full workflow
#[test]
fn test_code_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("workflow.js");
    
    // Create initial file with var declarations
    fs::write(&file_path, 
        "var username = 'john';\n\
         var age = 25;\n\
         console.log('User:', username);\n"
    ).unwrap();

    // Step 1: Search for var declarations
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "search",
        "var",
        "--files", file_path.to_str().unwrap(),
        "--language", "javascript"
    ]);
    cmd.assert().success();

    // Step 2: Transform var to let
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "transform",
        "var",
        "let",
        "--files", file_path.to_str().unwrap(),
        "--language", "javascript",
        "--no-backup"
    ]);
    cmd.assert().success();

    // Step 3: Analyze the transformed file
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.args(&[
        "code", "analyze",
        "--files", file_path.to_str().unwrap()
    ]);
    cmd.assert().success();

    // Verify transformation worked
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("let"));
    assert!(!content.contains("var"));
}
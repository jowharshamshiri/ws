use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use assert_cmd::Command;

#[test]
fn test_completion_setup_detection() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    // Test with HOME set to temp directory
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    let output = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    
    // Should contain completion setup instructions
    assert!(stderr.contains("To enable ws completions"));
}

#[test]
fn test_completion_file_creation_zsh() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("SHELL", "/bin/zsh");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    cmd.assert().success();
    
    // Check that completion file was created
    let completion_file = PathBuf::from(temp_home)
        .join(".local/share/zsh/site-functions/_ws");
    assert!(completion_file.exists());
    
    // Check that file contains completion content
    let content = fs::read_to_string(&completion_file).unwrap();
    assert!(content.contains("_ws"));
    assert!(content.contains("#compdef"));
}

#[test]
fn test_completion_file_creation_bash() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("SHELL", "/bin/bash");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    cmd.assert().success();
    
    // Check that completion file was created
    let completion_file = PathBuf::from(temp_home)
        .join(".local/share/bash-completion/completions/ws");
    assert!(completion_file.exists());
    
    // Check that file contains completion content
    let content = fs::read_to_string(&completion_file).unwrap();
    assert!(content.contains("_ws"));
    assert!(content.contains("complete"));
}

#[test]
fn test_completion_file_creation_fish() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("SHELL", "/usr/bin/fish");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    cmd.assert().success();
    
    // Check that completion file was created
    let completion_file = PathBuf::from(temp_home)
        .join(".local/share/fish/completions/ws.fish");
    assert!(completion_file.exists());
    
    // Check that file contains completion content
    let content = fs::read_to_string(&completion_file).unwrap();
    assert!(content.contains("complete"));
    assert!(content.contains("ws"));
}

#[test]
fn test_no_repeated_completion_setup() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    // First run should set up completions
    let mut cmd1 = Command::cargo_bin("ws").unwrap();
    cmd1.env("HOME", temp_home);
    cmd1.env("SHELL", "/bin/zsh");
    cmd1.env_remove("WS_COMPLETIONS_LOADED");
    cmd1.arg("--help");
    
    let output1 = cmd1.assert().success();
    let stderr1 = String::from_utf8_lossy(&output1.get_output().stderr);
    assert!(stderr1.contains("To enable ws completions"));
    
    // Second run with WS_COMPLETIONS_LOADED should skip setup
    let mut cmd2 = Command::cargo_bin("ws").unwrap();
    cmd2.env("HOME", temp_home);
    cmd2.env("SHELL", "/bin/zsh");
    cmd2.env("WS_COMPLETIONS_LOADED", "1");
    cmd2.arg("--help");
    
    let output2 = cmd2.assert().success();
    let stderr2 = String::from_utf8_lossy(&output2.get_output().stderr);
    assert!(!stderr2.contains("To enable ws completions"));
}

#[test]
fn test_shell_detection() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    // Test ZSH detection via ZSH_VERSION
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("ZSH_VERSION", "5.8");
    cmd.env_remove("SHELL");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    let output = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    
    // Should detect zsh and create appropriate completion
    assert!(stderr.contains("fpath="));
}

#[test]
fn test_completion_content_includes_all_commands() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("SHELL", "/bin/bash");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    cmd.assert().success();
    
    let completion_file = PathBuf::from(temp_home)
        .join(".local/share/bash-completion/completions/ws");
    let content = fs::read_to_string(&completion_file).unwrap();
    
    // Should include all main commands
    assert!(content.contains("refactor"));
    assert!(content.contains("git"));
    assert!(content.contains("template"));
    assert!(content.contains("update"));
    assert!(content.contains("scrap"));
    assert!(content.contains("unscrap"));
    assert!(content.contains("ldiff"));
}

#[test]
fn test_xdg_data_home_respected() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    let xdg_data = temp_dir.path().join("xdg-data");
    
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("XDG_DATA_HOME", xdg_data.to_str().unwrap());
    cmd.env("SHELL", "/bin/zsh");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    cmd.assert().success();
    
    // Should use XDG_DATA_HOME location
    let completion_file = xdg_data.join("zsh/site-functions/_ws");
    assert!(completion_file.exists());
}

#[test]
fn test_no_completion_setup_during_git_hook() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    // Test with GIT_DIR environment variable (simulates git hook context)
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("GIT_DIR", temp_dir.path().join(".git"));
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    let output = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    
    // Should not have completion setup output in stderr when running as git hook
    assert!(!stderr.contains("To enable ws completions"));
    assert!(!stderr.contains("fpath="));
    
    // But should still have the help text in stdout
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Shell completions are automatically set up"));
}

#[test]
fn test_no_completion_setup_with_git_index_file() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    // Test with GIT_INDEX_FILE environment variable (another git hook indicator)
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env("GIT_INDEX_FILE", "/tmp/git_index_123");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    let output = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    
    // Should not have completion setup output when git hook environment detected
    assert!(!stderr.contains("To enable ws completions"));
    assert!(!stderr.contains("fpath="));
}

#[test]
fn test_completion_setup_works_normally() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_str().unwrap();
    
    // Test without git hook environment (normal execution)
    let mut cmd = Command::cargo_bin("ws").unwrap();
    cmd.env("HOME", temp_home);
    cmd.env_remove("GIT_DIR");
    cmd.env_remove("GIT_INDEX_FILE");
    cmd.env_remove("WS_COMPLETIONS_LOADED");
    cmd.arg("--help");
    
    let output = cmd.assert().success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    
    // Should have completion setup output in stderr during normal execution
    assert!(stderr.contains("To enable ws completions"));
    
    // Should have the help text in stdout
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Shell completions are automatically set up"));
}
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_git_repo(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize git repository
    std::process::Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output()?;
    
    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir)
        .output()?;
    
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()?;
    
    // Create initial commit
    fs::write(dir.join("README.md"), "# Test Repository")?;
    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(dir)
        .output()?;
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(dir)
        .output()?;
    
    Ok(())
}

fn create_test_commits(dir: &Path, count: u32) -> Result<(), Box<dyn std::error::Error>> {
    // Get current number of files to avoid conflicts
    let existing_files = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with("file") &&
            entry.file_name().to_string_lossy().ends_with(".txt")
        })
        .count() as u32;
    
    for i in 1..=count {
        let file_name = format!("file{}.txt", existing_files + i);
        fs::write(dir.join(&file_name), format!("Content {}", existing_files + i))?;
        
        std::process::Command::new("git")
            .args(["add", &file_name])
            .current_dir(dir)
            .output()?;
        
        std::process::Command::new("git")
            .args(["commit", "-m", &format!("Add {}", file_name)])
            .current_dir(dir)
            .output()?;
    }
    Ok(())
}

#[test]
fn test_st8_help() {
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Git integration commands"));
}

#[test]
fn test_st8_version() {
    Command::cargo_bin("ws")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ws"));
}

#[test]
fn test_st8_outside_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a git repository. Use --no-git to update anyway"));
}

#[test]
fn test_st8_show_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 3).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("show")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Current Version Information"))
        .stdout(predicate::str::contains("Major"))
        .stdout(predicate::str::contains("Minor"))
        .stdout(predicate::str::contains("Patch"))
        .stdout(predicate::str::contains("Full Version"));
}

#[test]
fn test_st8_status_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("status")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("St8 Status"))
        .stdout(predicate::str::contains("Git Repository"))
        .stdout(predicate::str::contains("Hook Installed"))
        .stdout(predicate::str::contains("Enabled"));
}

#[test]
fn test_st8_status_outside_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("status")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Not in a git repository"));
}

#[test]
fn test_st8_update_creates_version_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 2).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated version to"));
    
    // Check that version.txt was created
    let version_file = temp_dir.path().join("version.txt");
    assert!(version_file.exists());
    
    let version_content = fs::read_to_string(&version_file).unwrap();
    assert!(!version_content.trim().is_empty());
    
    // Should be in format X.Y.Z where Y >= 2 (we created 2 additional commits)
    let parts: Vec<&str> = version_content.trim().split('.').collect();
    assert_eq!(parts.len(), 3);
    
    let minor_version: u32 = parts[1].parse().unwrap();
    assert!(minor_version >= 2);
}

#[test]
fn test_ws_git_install_hook() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Git hook installed successfully"));
    
    // Check that pre-commit hook was created
    let hook_file = temp_dir.path().join(".git").join("hooks").join("pre-commit");
    assert!(hook_file.exists());
    
    let hook_content = fs::read_to_string(&hook_file).unwrap();
    assert!(hook_content.contains("=== WS BLOCK START ==="));
    assert!(hook_content.contains("=== WS BLOCK END ==="));
    assert!(hook_content.contains("ws update --git-add"));
    
    // Check that hook is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::metadata(&hook_file).unwrap().permissions();
        assert_ne!(perms.mode() & 0o111, 0); // At least one execute bit set
    }
}

#[test]
fn test_ws_git_install_hook_already_installed() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // First installation
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Second installation without force should inform already installed
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("already installed"));
    
    // Second installation should succeed (already installed is not an error)
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_st8_install_hook_with_existing_hook() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Create existing pre-commit hook
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_file = hooks_dir.join("pre-commit");
    
    let existing_content = "#!/bin/bash\necho \"Existing hook\"\n";
    fs::write(&hook_file, existing_content).unwrap();
    
    // Install st8 hook
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Check that both old and new content exist
    let updated_content = fs::read_to_string(&hook_file).unwrap();
    assert!(updated_content.contains("Existing hook"));
    assert!(updated_content.contains("=== WS BLOCK START ==="));
}

#[test]
fn test_st8_uninstall_hook() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Install hook first
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    let hook_file = temp_dir.path().join(".git").join("hooks").join("pre-commit");
    assert!(hook_file.exists());
    
    // Uninstall hook
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("uninstall")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Git hook uninstalled successfully"));
    
    // Hook file should be removed (since it only contained st8)
    assert!(!hook_file.exists());
}

#[test]
fn test_st8_uninstall_hook_with_other_content() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Create hook with existing content
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_file = hooks_dir.join("pre-commit");
    
    let existing_content = "#!/bin/bash\necho \"Other hook content\"\n";
    fs::write(&hook_file, existing_content).unwrap();
    
    // Install st8 hook
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Uninstall st8 hook
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("uninstall")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Hook file should still exist with original content
    assert!(hook_file.exists());
    let remaining_content = fs::read_to_string(&hook_file).unwrap();
    assert!(remaining_content.contains("Other hook content"));
    assert!(!remaining_content.contains("ST8 BLOCK"));
}

#[test]
fn test_st8_uninstall_no_hook() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Try to uninstall when no hook exists
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("uninstall")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No pre-commit hook found"));
}

#[test]
fn test_st8_default_behavior_install() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Default behavior should install hook if not installed
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Git hook installed successfully"));
    
    let hook_file = temp_dir.path().join(".git").join("hooks").join("pre-commit");
    assert!(hook_file.exists());
}

#[test]
fn test_st8_default_behavior_update() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Install hook first
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Default behavior should now update version
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated version to"));
    
    let version_file = temp_dir.path().join("version.txt");
    assert!(version_file.exists());
}

#[test]
fn test_st8_with_git_tag() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 2).unwrap();
    
    // Create a git tag
    std::process::Command::new("git")
        .args(["tag", "v1.0"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    // Create more commits after tag
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("show")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Major (tag): v1.0"))
        .stdout(predicate::str::contains("Minor (commits since tag): 1"));
}

#[test]
fn test_st8_config_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Create custom config
    let config_content = r#"{
  "version": 1,
  "enabled": true,
  "version_file": "custom_version.txt"
}"#;
    fs::write(temp_dir.path().join(".st8.json"), config_content).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Check that custom version file was created
    let custom_version_file = temp_dir.path().join("custom_version.txt");
    assert!(custom_version_file.exists());
    
    // Regular version.txt should not exist
    let default_version_file = temp_dir.path().join("version.txt");
    assert!(!default_version_file.exists());
}

#[test] 
fn test_st8_logging() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Check that log file was created in .ws/st8/logs/
    let log_file = temp_dir.path().join(".ws").join("st8").join("logs").join("st8.log");
    assert!(log_file.exists());
    
    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("Created new pre-commit hook") || 
           log_content.contains("Updated existing pre-commit hook"));
}

#[test]
fn test_st8_auto_detect_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Create a Cargo.toml file
    let cargo_content = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
    
    // Run st8 update
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated version to"))
        .stdout(predicate::str::contains("Updated project files"));
    
    // Check that Cargo.toml was updated
    let updated_cargo = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(updated_cargo.contains("name = \"test-project\""));
    assert!(!updated_cargo.contains("version = \"0.1.0\""));
    
    // Should contain new version format (something like "0.1.x.y")
    assert!(updated_cargo.contains("version = \"0."));
}

#[test]
fn test_st8_auto_detect_package_json() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 2).unwrap();
    
    // Create a package.json file
    let package_content = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "description": "A test package",
  "main": "index.js",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"
  }
}"#;
    fs::write(temp_dir.path().join("package.json"), package_content).unwrap();
    
    // Run st8 update
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated project files"));
    
    // Check that package.json was updated
    let updated_package = fs::read_to_string(temp_dir.path().join("package.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&updated_package).unwrap();
    
    assert_eq!(parsed["name"], "test-package");
    assert_ne!(parsed["version"], "1.0.0"); // Should be updated
    
    // Should be in format "0.X.Y" where X >= 2 (we created 2 additional commits)
    let version_str = parsed["version"].as_str().unwrap();
    let parts: Vec<&str> = version_str.split('.').collect();
    assert_eq!(parts.len(), 3);
    let minor_version: u32 = parts[1].parse().unwrap();
    assert!(minor_version >= 2);
}

#[test]
fn test_st8_auto_detect_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Create multiple project files
    let cargo_content = r#"[package]
name = "multi-test"
version = "0.5.0"
"#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
    
    let package_content = r#"{
  "name": "multi-test",
  "version": "0.5.0"
}"#;
    fs::write(temp_dir.path().join("package.json"), package_content).unwrap();
    
    let pyproject_content = r#"[tool.poetry]
name = "multi-test"
version = "0.5.0"

[project]
name = "multi-test"
version = "0.5.0"
"#;
    fs::write(temp_dir.path().join("pyproject.toml"), pyproject_content).unwrap();
    
    // Run st8 update
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated project files"));
    
    // Check that all files were updated with the same version
    let version_file_content = fs::read_to_string(temp_dir.path().join("version.txt")).unwrap();
    let new_version = version_file_content.trim();
    
    // Check Cargo.toml
    let updated_cargo = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(updated_cargo.contains(&format!("version = \"{}\"", new_version)));
    
    // Check package.json
    let updated_package = fs::read_to_string(temp_dir.path().join("package.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&updated_package).unwrap();
    assert_eq!(parsed["version"], new_version);
    
    // Check pyproject.toml
    let updated_pyproject = fs::read_to_string(temp_dir.path().join("pyproject.toml")).unwrap();
    assert!(updated_pyproject.contains(&format!("version = \"{}\"", new_version)));
    // Should appear twice (in tool.poetry and project sections)
    assert_eq!(updated_pyproject.matches(&format!("version = \"{}\"", new_version)).count(), 2);
}

#[test]
fn test_st8_status_shows_detected_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Create project files
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();
    fs::write(temp_dir.path().join("package.json"), "{\"name\": \"test\", \"version\": \"1.0.0\"}").unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("status")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Auto-detect Project Files: ✓"))
        .stdout(predicate::str::contains("Detected Project Files:"))
        .stdout(predicate::str::contains("Cargo.toml"))
        .stdout(predicate::str::contains("package.json"));
}

#[test]
fn test_st8_config_disable_auto_detect() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Create config with auto-detect disabled
    let config_content = r#"{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt",
  "auto_detect_project_files": false
}"#;
    fs::write(temp_dir.path().join(".st8.json"), config_content).unwrap();
    
    // Create a Cargo.toml file
    let cargo_content = r#"[package]
name = "no-auto-detect"
version = "0.1.0"
"#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
    
    // Run st8 update
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated version to"))
        .stdout(predicate::str::contains("Updated project files").not());
    
    // Check that Cargo.toml was NOT updated (still has old version)
    let updated_cargo = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(updated_cargo.contains("version = \"0.1.0\""));
    
    // But version.txt should be updated
    let version_file = temp_dir.path().join("version.txt");
    assert!(version_file.exists());
    let version_content = fs::read_to_string(&version_file).unwrap();
    assert_ne!(version_content.trim(), "0.1.0");
}

#[test]
fn test_st8_manual_project_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Create config with manual project files
    let config_content = r#"{
  "version": 1,
  "enabled": true,
  "version_file": "version.txt",
  "auto_detect_project_files": false,
  "project_files": ["custom.json", "Cargo.toml"]
}"#;
    fs::write(temp_dir.path().join(".st8.json"), config_content).unwrap();
    
    // Create the specified files
    fs::write(temp_dir.path().join("custom.json"), "{\"version\": \"1.0.0\"}").unwrap();
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"manual\"\nversion = \"1.0.0\"").unwrap();
    
    // Run st8 update
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated configured project files"));
    
    // Check that both files were updated
    let updated_custom = fs::read_to_string(temp_dir.path().join("custom.json")).unwrap();
    let custom_parsed: serde_json::Value = serde_json::from_str(&updated_custom).unwrap();
    assert_ne!(custom_parsed["version"], "1.0.0");
    
    let updated_cargo = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(!updated_cargo.contains("version = \"1.0.0\""));
}

#[test]
fn test_st8_no_update_when_version_unchanged() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Create a Cargo.toml file
    let cargo_content = r#"[package]
name = "test-project"
version = "0.1.0"
"#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
    
    // Run st8 update first time
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated version to"));
    
    // Get the version that was set
    let version_content = fs::read_to_string(temp_dir.path().join("version.txt")).unwrap();
    let version = version_content.trim();
    
    // Run st8 update second time (no git changes)
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&format!("Version {} is already up to date", version)))
        .stdout(predicate::str::contains("Updated project files").not());
    
    // Cargo.toml should not have been modified again
    let cargo_modified_time = fs::metadata(temp_dir.path().join("Cargo.toml"))
        .unwrap()
        .modified()
        .unwrap();
    
    // Sleep a bit and run again to make sure file timestamp would change if modified
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("already up to date"));
    
    // File should not have been touched
    let cargo_modified_time_after = fs::metadata(temp_dir.path().join("Cargo.toml"))
        .unwrap()
        .modified()
        .unwrap();
    
    assert_eq!(cargo_modified_time, cargo_modified_time_after);
}

#[test]
fn test_st8_logging_in_workspace_directory() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    // Install st8 to trigger logging
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Check that log file was created in .ws/st8/logs/ directory
    let log_file = temp_dir.path().join(".ws").join("st8").join("logs").join("st8.log");
    assert!(log_file.exists(), "st8.log should be created in .ws/st8/logs/");
    
    // Check log content
    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("pre-commit hook"), "Log should contain hook installation message");
    assert!(log_content.contains("["), "Log should have timestamp format");
    
    // Trigger another logging action (uninstall)
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("uninstall")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Check that log was appended
    let log_content_after = fs::read_to_string(&log_file).unwrap();
    assert!(log_content_after.len() > log_content.len(), "Log should be appended to");
    assert!(log_content_after.contains("Removed"), "Log should contain uninstall message");
}

#[test]
fn test_st8_update_with_git_add() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Create a version file
    let version_file = temp_dir.path().join("VERSION");
    fs::write(&version_file, "1.0.0").unwrap();
    
    // Add version file to git and commit it
    std::process::Command::new("git")
        .args(["add", "VERSION"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Add version file"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    // Run st8 update with --git-add flag
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .arg("--git-add")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated version to:"));
    
    // Check that the version file is in git staging area
    let git_status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    let status_output = String::from_utf8(git_status.stdout).unwrap();
    
    // The version file should be staged (shows as "M " in git status --porcelain)
    // or already committed if the version didn't change
    assert!(
        status_output.contains("M  VERSION") || 
        status_output.is_empty() || 
        !status_output.contains("VERSION"),
        "Version file should be staged or not modified, got: '{}'", 
        status_output
    );
}

#[test]
fn test_st8_update_without_git_add() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    create_test_commits(temp_dir.path(), 1).unwrap();
    
    // Create a version file
    let version_file = temp_dir.path().join("VERSION");
    fs::write(&version_file, "1.0.0").unwrap();
    
    // Add version file to git and commit it
    std::process::Command::new("git")
        .args(["add", "VERSION"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Add version file"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    // Run st8 update without --git-add flag
    Command::cargo_bin("ws")
        .unwrap()
        .arg("update")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated version to:"));
    
    // Check that the version file might be modified but not staged
    let git_status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    let status_output = String::from_utf8(git_status.stdout).unwrap();
    
    // Without --git-add, the file should either not be modified (if version didn't change)
    // or be modified but not staged (shows as " M" in git status --porcelain)
    assert!(
        status_output.contains(" M VERSION") || 
        status_output.is_empty() || 
        !status_output.contains("VERSION"),
        "Version file should be unstaged if modified, got: '{}'", 
        status_output
    );
}

#[test]
fn test_st8_install_hook_includes_git_add() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(temp_dir.path()).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("git")
        .arg("install")
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Check that the hook includes --git-add flag
    let hook_file = temp_dir.path().join(".git").join("hooks").join("pre-commit");
    assert!(hook_file.exists());
    
    let hook_content = fs::read_to_string(&hook_file).unwrap();
    assert!(hook_content.contains("update --git-add"), 
           "Hook should include --git-add flag, got: {}", hook_content);
}
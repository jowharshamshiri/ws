use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_git_repo(temp_dir: &TempDir) {
    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Configure git
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Create version file
    fs::write(temp_dir.path().join("version.txt"), "1.0.0").unwrap();
    
    // Create Cargo.toml for project name detection
    let cargo_content = r#"[package]
name = "test-project"
version = "1.0.0"
edition = "2021"
"#;
    fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
}

#[test]
fn test_template_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "list"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No templates configured"));
}

#[test]
fn test_template_add_and_list() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add a template
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "changelog",
            "--template", "# {{ project.name }} v{{ project.version }}\n\nRelease notes",
            "--output", "CHANGELOG.md",
            "--description", "Changelog template"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added template 'changelog' → CHANGELOG.md"));
    
    // List templates
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "list"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("changelog → CHANGELOG.md (enabled)"))
        .stdout(predicate::str::contains("Changelog template"));
}

#[test]
fn test_template_show() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add a template
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "release",
            "--template", "Version {{ project.version }} released",
            "--output", "RELEASE.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Show template details
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "show", "release"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Template: release"))
        .stdout(predicate::str::contains("Output: RELEASE.txt"))
        .stdout(predicate::str::contains("Enabled: Yes"))
        .stdout(predicate::str::contains("Version {{ project.version }} released"));
}

#[test]
fn test_template_show_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "show", "nonexistent"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Template 'nonexistent' not found"));
}

#[test]
fn test_template_enable_disable() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add a template
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "notes",
            "--template", "Notes for {{ project.version }}",
            "--output", "notes.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Disable template
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "enable", "notes", "--disable"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Template 'notes' disabled"));
    
    // Verify disabled in list
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "list"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("notes → notes.txt (disabled)"));
    
    // Enable template
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "enable", "notes"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Template 'notes' enabled"));
}

#[test]
fn test_template_enable_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "enable", "nonexistent"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Template 'nonexistent' not found"));
}

#[test]
fn test_template_render() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add templates
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "version",
            "--template", "Version: {{ project.version }}",
            "--output", "VERSION.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "info",
            "--template", "Project: {{ project.name | default(value='Unknown') }}\nVersion: {{ project.version }}\nDate: {{ datetime.date }}",
            "--output", "INFO.md"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Render templates
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "render"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Rendered 2 template(s)"))
        .stdout(predicate::str::contains("VERSION.txt"))
        .stdout(predicate::str::contains("INFO.md"));
    
    // Verify rendered files exist and have correct content
    let version_content = fs::read_to_string(temp_dir.path().join("VERSION.txt")).unwrap();
    assert!(version_content.contains("Version: 0."));
    
    let info_content = fs::read_to_string(temp_dir.path().join("INFO.md")).unwrap();
    assert!(info_content.contains("Project: test-project"));
    assert!(info_content.contains("Version: 0."));
    assert!(info_content.contains("Date: "));
}

#[test]
fn test_template_render_no_enabled() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add a disabled template
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "disabled",
            "--template", "{{ project.version }}",
            "--output", "disabled.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "enable", "disabled", "--disable"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Render should report no enabled templates
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "render"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No enabled templates to render"));
}

#[test]
fn test_template_remove() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add a template
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "temp",
            "--template", "temporary",
            "--output", "temp.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Verify it exists
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "list"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("temp → temp.txt"));
    
    // Remove template
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "remove", "temp"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed template 'temp'"));
    
    // Verify it's gone
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "list"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No templates configured"));
}

#[test]
fn test_template_remove_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "remove", "nonexistent"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Template 'nonexistent' not found"));
}

#[test]
fn test_template_add_from_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Create a template file
    let template_content = "# Release {{ project.version }}\n\n## Changes\n- Version bump to {{ project.version }}";
    let template_file = temp_dir.path().join("release_template.md");
    fs::write(&template_file, template_content).unwrap();
    
    // Add template from file
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "release",
            "--template", template_file.to_str().unwrap(),
            "--output", "RELEASE.md",
            "--description", "Release notes from file"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added template 'release' → RELEASE.md"));
    
    // Show template to verify content was read from file
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "show", "release"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Version bump to {{ project.version }}"));
}

#[test]
fn test_template_add_from_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "test",
            "--template", "nonexistent_file.txt",
            "--output", "output.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Template file not found"));
}

#[test]
fn test_template_integration_with_update() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add a template
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "version_info",
            "--template", "Current version: {{ project.version }}\nProject: {{ project.name }}",
            "--output", "VERSION_INFO.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Update version (which should render templates automatically)
    Command::cargo_bin("st8")
        .unwrap()
        .args(["update", "--no-git"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Rendered 1 template(s)"))
        .stdout(predicate::str::contains("VERSION_INFO.txt"));
    
    // Verify the template was rendered
    let version_info_content = fs::read_to_string(temp_dir.path().join("VERSION_INFO.txt")).unwrap();
    assert!(version_info_content.contains("Current version: 0."));
    assert!(version_info_content.contains("Project: test-project"));
}

#[test]
fn test_template_status_integration() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add templates
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "enabled_template",
            "--template", "enabled",
            "--output", "enabled.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "disabled_template",
            "--template", "disabled",
            "--output", "disabled.txt"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "enable", "disabled_template", "--disable"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Check status shows templates
    Command::cargo_bin("st8")
        .unwrap()
        .args(["status"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Templates: 2"))
        .stdout(predicate::str::contains("enabled_template → enabled.txt (✓)"))
        .stdout(predicate::str::contains("disabled_template → disabled.txt (✗)"));
}

#[test] 
fn test_template_with_complex_variables() {
    let temp_dir = TempDir::new().unwrap();
    setup_git_repo(&temp_dir);
    
    // Add a template using various template variables
    let complex_template = r#"# {{ project.name | title }} Release

**Version:** {{ project.version }}
**Major:** {{ project.major_version }}
**Minor:** {{ project.minor_version }}
**Patch:** {{ project.patch_version }}

**Release Date:** {{ datetime.date }}
**Release Time:** {{ datetime.time }}
**Year:** {{ datetime.year }}

---
Generated on {{ datetime.iso }}
"#;
    
    Command::cargo_bin("st8")
        .unwrap()
        .args([
            "template", "add", "complex",
            "--template", complex_template,
            "--output", "COMPLEX_RELEASE.md"
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Render template
    Command::cargo_bin("st8")
        .unwrap()
        .args(["template", "render"])
        .current_dir(temp_dir.path())
        .assert()
        .success();
    
    // Verify complex template rendered correctly
    let content = fs::read_to_string(temp_dir.path().join("COMPLEX_RELEASE.md")).unwrap();
    assert!(content.contains("# Test-Project Release") || content.contains("# Test-project Release"));
    assert!(content.contains("**Version:** 0."));
    assert!(content.contains("**Major:** v0") || content.contains("**Major:** 0"));
    assert!(content.contains("**Minor:** "));
    assert!(content.contains("**Patch:** "));
    assert!(content.contains("**Release Date:** "));
    assert!(content.contains("**Year:** 20"));
    assert!(content.contains("Generated on "));
}
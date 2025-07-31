use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_scrap_with_items(temp_dir: &Path) {
    // Create test files in the temp directory first
    fs::write(temp_dir.join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.join("file2.log"), "log content").unwrap();
    
    // Create a test directory
    let test_dir = temp_dir.join("testdir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("nested.txt"), "nested content").unwrap();
    
    // Now scrap them using the ws command
    let _ = Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("file1.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_dir)
        .output();
        
    let _ = Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("file2.log")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_dir)
        .output();
        
    let _ = Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("testdir")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_dir)
        .output();
}

#[test]
fn test_scrap_list_default() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // Test default list behavior (no arguments)
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scrapped files:"));
}

#[test]
fn test_scrap_list_explicit() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // Test explicit list command
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("list")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scrapped files:"));
}

#[test]
fn test_scrap_list_sort_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("list")
        .arg("--sort")
        .arg("name")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scrapped files:"));    
}

#[test]
fn test_scrap_list_sort_size() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("list")
        .arg("--sort")
        .arg("size")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scrapped files:"));    
}

#[test]
fn test_scrap_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create empty .scrap folder
    fs::create_dir(temp_path.join(".scrap")).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scrap folder is empty"));
}

#[test]
fn test_scrap_clean_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("clean")
        .arg("--days")
        .arg("0")  // Clean everything
        .arg("--dry-run")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Would remove 3 items older than 0 days"));
}

#[test]
fn test_scrap_clean_actual() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // First check items exist
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("clean")
        .arg("--days")
        .arg("0")  // Clean everything
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 3 items older than 0 days"));
    
    // Check items were removed
    assert!(!temp_path.join(".scrap").join("file1.txt").exists());
}

#[test]
fn test_scrap_purge_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    // Check items exist
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("purge")
        .arg("--force")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Purged 3 items from scrap folder"));
    
    // Check all items were removed
    assert!(!temp_path.join(".scrap").join("file1.txt").exists());
    assert!(!temp_path.join(".scrap").join("file2.log").exists());
    assert!(!temp_path.join(".scrap").join("testdir").exists());
}

#[test]
fn test_scrap_purge_empty_folder() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create empty .scrap folder
    fs::create_dir(temp_path.join(".scrap")).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("purge")
        .arg("--force")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Purged 0 items from scrap folder"));
}

#[test]
fn test_scrap_find_filename() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("find")
        .arg("file.*txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching files found"));
}

#[test]
fn test_scrap_find_no_matches() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("find")
        .arg("nonexistent")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching files found"));
}

#[test]
fn test_scrap_find_content() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("find")
        .arg("content1")
        .arg("--content")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching files found"));
}

#[test]
fn test_scrap_archive() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    let archive_name = "test-archive.tar.gz";
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("archive")
        .arg("--output")
        .arg(archive_name)
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created archive: test-archive.tar.gz"));
    
    // Check archive was created
    assert!(temp_path.join(archive_name).exists());
    
    // Check original files still exist
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
}

#[test]
fn test_scrap_archive_with_remove() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    let archive_name = "test-archive-remove.tar.gz";
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("archive")
        .arg("--output")
        .arg(archive_name)
        .arg("--remove")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created archive: test-archive-remove.tar.gz"));
    
    // Check archive was created
    assert!(temp_path.join(archive_name).exists());
    
    // Check original files were removed
    assert!(!temp_path.join(".scrap").join("file1.txt").exists());
    assert!(!temp_path.join(".scrap").join("file2.log").exists());
    assert!(!temp_path.join(".scrap").join("testdir").exists());
}

#[test]
fn test_scrap_archive_empty_folder() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create empty .scrap folder
    fs::create_dir(temp_path.join(".scrap")).unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("archive")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created archive: scrap-archive.tar.gz"));
}

#[test]
fn test_scrap_archive_default_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    setup_scrap_with_items(temp_path);
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("archive")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created archive"));
}

#[test]
fn test_scrap_with_metadata_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a test file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    // Scrap the file
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("test.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Check metadata was created
    let metadata_file = temp_path.join(".scrap").join(".metadata.json");
    assert!(metadata_file.exists());
    
    // Check metadata content
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(metadata_content.contains("test.txt"));
    assert!(metadata_content.contains("original_path"));
    assert!(metadata_content.contains("scrapped_at"));
}

#[test]
fn test_scrap_list_with_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap a file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("test.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // List should show metadata info
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"))
        .stdout(predicate::str::contains("(from test.txt)"));
}

#[test]
fn test_unscrap_integration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap a file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("test.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Verify file was moved
    assert!(!test_file.exists());
    assert!(temp_path.join(".scrap").join("test.txt").exists());
    
    // Restore the file
    Command::cargo_bin("ws")
        .unwrap()
        .arg("unscrap")
        .arg("test.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored test.txt to test.txt"));
    
    // Verify file was restored
    assert!(test_file.exists());
    assert!(!temp_path.join(".scrap").join("test.txt").exists());
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "test content");
}

#[test]
fn test_unscrap_undo_last() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap two files
    let test_file1 = temp_path.join("file1.txt");
    let test_file2 = temp_path.join("file2.txt");
    fs::write(&test_file1, "content1").unwrap();
    fs::write(&test_file2, "content2").unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("file1.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Wait a bit to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("file2.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Undo last (should restore file2.txt)
    Command::cargo_bin("ws")
        .unwrap()
        .arg("unscrap")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored file2.txt to file2.txt"));
    
    // Verify correct file was restored
    assert!(test_file2.exists());
    assert!(!test_file1.exists());
    assert!(temp_path.join(".scrap").join("file1.txt").exists());
    assert!(!temp_path.join(".scrap").join("file2.txt").exists());
}

#[test]
fn test_unscrap_custom_destination() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create and scrap a file
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    Command::cargo_bin("ws")
        .unwrap()
        .arg("scrap")
        .arg("test.txt")
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Create custom destination directory
    let custom_dir = temp_path.join("custom");
    fs::create_dir(&custom_dir).unwrap();
    let custom_file = custom_dir.join("test.txt");
    
    // Restore to custom location
    Command::cargo_bin("ws")
        .unwrap()
        .arg("unscrap")
        .arg("test.txt")
        .arg("--to")
        .arg(&custom_file)
        .env("WS_COMPLETIONS_LOADED", "1")
        .current_dir(temp_path)
        .assert()
        .success();
    
    // Verify file was restored to custom location
    assert!(custom_file.exists());
    assert!(!test_file.exists());
    assert!(!temp_path.join(".scrap").join("test.txt").exists());
}
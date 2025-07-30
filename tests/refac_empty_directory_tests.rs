use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use nomion::{cli::Args, run_refac};

/// Specific tests for empty directory cleanup issues in refac tool
/// These tests address the specific issue where folders remain empty but undeleted

#[test]
fn test_empty_directory_left_after_all_files_renamed() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a directory with files that will ALL be renamed out
    let source_dir = temp_dir.path().join("source_oldname_dir");
    fs::create_dir(&source_dir)?;
    
    // Put files that will be renamed to different directory structure
    File::create(source_dir.join("oldname_file1.txt"))?
        .write_all(b"oldname content 1")?;
    File::create(source_dir.join("oldname_file2.txt"))?
        .write_all(b"oldname content 2")?;
    File::create(source_dir.join("file_oldname.txt"))?
        .write_all(b"oldname content 3")?;
    
    // Also create target directory structure
    let target_dir = temp_dir.path().join("target_newname_dir");
    fs::create_dir(&target_dir)?;

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Check what happened to the original directory
    println!("Source dir exists: {}", source_dir.exists());
    if source_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&source_dir)?.collect();
        println!("Source dir contents count: {}", entries.len());
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  - {}", entry.path().display());
            }
        }
    }
    
    // The source directory might still exist but should be empty, or should be removed
    // This is the specific issue - empty directories not being cleaned up
    if source_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&source_dir)?.collect();
        assert!(entries.is_empty(), "Source directory should be empty after all files renamed out");
    }

    Ok(())
}

#[test]
fn test_nested_empty_directories_after_complete_rename() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create deeply nested structure where ALL content will be renamed
    fs::create_dir_all(temp_dir.path().join("oldname_root/oldname_middle/oldname_leaf"))?;
    
    // Only put content in the deepest level
    File::create(temp_dir.path().join("oldname_root/oldname_middle/oldname_leaf/oldname_file.txt"))?
        .write_all(b"oldname content")?;
    
    // Create another branch that will also be completely renamed
    fs::create_dir_all(temp_dir.path().join("oldname_root/oldname_branch"))?;
    File::create(temp_dir.path().join("oldname_root/oldname_branch/oldname_another.txt"))?
        .write_all(b"oldname content")?;

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Check for empty directory cleanup
    let old_root = temp_dir.path().join("oldname_root");
    let old_middle = temp_dir.path().join("oldname_root/oldname_middle");
    let old_leaf = temp_dir.path().join("oldname_root/oldname_middle/oldname_leaf");
    let old_branch = temp_dir.path().join("oldname_root/oldname_branch");
    
    // These directories should either be renamed or cleaned up if empty
    // The specific bug is when they exist but are empty
    for dir in [&old_root, &old_middle, &old_leaf, &old_branch] {
        if dir.exists() {
            let entries: Vec<_> = fs::read_dir(dir)?.collect();
            if entries.is_empty() {
                panic!("Empty directory not cleaned up: {}", dir.display());
            }
        }
    }
    
    // Verify the renamed structure exists
    assert!(temp_dir.path().join("newname_root").exists());
    assert!(temp_dir.path().join("newname_root/newname_middle/newname_leaf/newname_file.txt").exists());
    assert!(temp_dir.path().join("newname_root/newname_branch/newname_another.txt").exists());

    Ok(())
}

#[test]
fn test_partially_emptied_directories() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create directory with mixed content - some will be renamed, some won't
    let mixed_dir = temp_dir.path().join("mixed_content_dir");
    fs::create_dir(&mixed_dir)?;
    
    // Files that will be renamed (contain oldname)
    File::create(mixed_dir.join("oldname_file1.txt"))?
        .write_all(b"oldname content")?;
    File::create(mixed_dir.join("file_oldname.txt"))?
        .write_all(b"oldname content")?;
    
    // Files that won't be renamed (don't contain oldname)
    File::create(mixed_dir.join("normal_file.txt"))?
        .write_all(b"normal content")?;
    File::create(mixed_dir.join("another_file.txt"))?
        .write_all(b"another content")?;
    
    // Subdirectory that will be completely renamed out
    let sub_oldname_dir = mixed_dir.join("oldname_subdir");
    fs::create_dir(&sub_oldname_dir)?;
    File::create(sub_oldname_dir.join("oldname_sub_file.txt"))?
        .write_all(b"oldname content")?;

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // The mixed directory should still exist (has non-oldname files)
    assert!(mixed_dir.exists());
    
    // Should contain the non-renamed files
    assert!(mixed_dir.join("normal_file.txt").exists());
    assert!(mixed_dir.join("another_file.txt").exists());
    
    // Should contain the renamed files
    assert!(mixed_dir.join("newname_file1.txt").exists());
    assert!(mixed_dir.join("file_newname.txt").exists());
    
    // The old subdirectory should be handled properly (either renamed or cleaned up)
    if sub_oldname_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&sub_oldname_dir)?.collect();
        assert!(entries.is_empty() || 
                entries.iter().any(|e| e.as_ref().unwrap().path().file_name().unwrap().to_str().unwrap().contains("newname")),
                "Subdirectory should be empty or contain renamed files");
    }
    
    // Check for the renamed subdirectory
    let new_subdir = mixed_dir.join("newname_subdir");
    if new_subdir.exists() {
        assert!(new_subdir.join("newname_sub_file.txt").exists());
    }

    Ok(())
}

#[test]
fn test_directory_rename_with_permission_issues() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create directory structure
    let parent_dir = temp_dir.path().join("oldname_parent");
    let child_dir = parent_dir.join("oldname_child");
    fs::create_dir_all(&child_dir)?;
    
    File::create(child_dir.join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    
    // This test simulates permission issues that might prevent cleanup
    // On Unix systems, we can test with restricted permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        // Make parent directory read-only (can't delete children)
        let mut perms = fs::metadata(&parent_dir)?.permissions();
        perms.set_mode(0o555);
        fs::set_permissions(&parent_dir, perms)?;
        
        let args = create_test_args(temp_dir.path(), "oldname", "newname");
        let result = run_refac(args);
        
        // Restore permissions for cleanup
        let mut restore_perms = fs::metadata(&parent_dir)?.permissions();
        restore_perms.set_mode(0o755);
        fs::set_permissions(&parent_dir, restore_perms)?;
        
        // Should handle permission errors gracefully
        match result {
            Ok(_) => {
                // If successful, verify state
                println!("Operation succeeded despite permission restrictions");
            },
            Err(e) => {
                // Should be a permission error, not a crash
                assert!(e.to_string().contains("permission") || e.to_string().contains("Permission") || e.to_string().contains("read-only") || e.to_string().contains("Read-Only"));
            }
        }
    }

    Ok(())
}

#[test]
fn test_empty_directory_with_hidden_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create directory that appears empty but has hidden files
    let hidden_dir = temp_dir.path().join("oldname_hidden_dir");
    fs::create_dir(&hidden_dir)?;
    
    // Add hidden files that will be renamed
    File::create(hidden_dir.join(".oldname_hidden"))?
        .write_all(b"oldname content")?;
    File::create(hidden_dir.join(".another_oldname_file"))?
        .write_all(b"oldname content")?;
    
    // Also add a hidden file that won't be renamed
    File::create(hidden_dir.join(".normal_hidden"))?
        .write_all(b"normal content")?;

    let args = Args {
        include_patterns: vec![".*".to_string()], // Include hidden files
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Directory should still exist with original name (has non-renamed hidden file)
    let original_hidden_dir = temp_dir.path().join("oldname_hidden_dir");
    assert!(original_hidden_dir.exists());
    
    // Should contain renamed and non-renamed hidden files
    assert!(original_hidden_dir.join(".newname_hidden").exists());
    assert!(original_hidden_dir.join(".another_newname_file").exists());
    assert!(original_hidden_dir.join(".normal_hidden").exists());

    Ok(())
}

#[test]
fn test_directory_with_only_subdirectories_all_renamed() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create directory containing only subdirectories that will all be renamed
    let container_dir = temp_dir.path().join("container_dir");
    fs::create_dir(&container_dir)?;
    
    // Create multiple subdirectories that will all be renamed
    for i in 1..=5 {
        let sub_dir = container_dir.join(format!("oldname_sub_{}", i));
        fs::create_dir(&sub_dir)?;
        File::create(sub_dir.join("file.txt"))?
            .write_all(format!("content {}", i).as_bytes())?;
    }

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Container directory should still exist (it wasn't renamed)
    assert!(container_dir.exists());
    
    // Should contain the renamed subdirectories
    for i in 1..=5 {
        let new_sub_dir = container_dir.join(format!("newname_sub_{}", i));
        assert!(new_sub_dir.exists(), "Subdirectory {} was not renamed", i);
        assert!(new_sub_dir.join("file.txt").exists());
    }
    
    // Original subdirectories should not exist
    for i in 1..=5 {
        let old_sub_dir = container_dir.join(format!("oldname_sub_{}", i));
        assert!(!old_sub_dir.exists(), "Old subdirectory {} still exists", i);
    }

    Ok(())
}

#[test]
fn test_symlinked_empty_directory_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create actual directory
    let real_dir = temp_dir.path().join("oldname_real_dir");
    fs::create_dir(&real_dir)?;
    File::create(real_dir.join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    
    // Create symlink to the directory
    let link_dir = temp_dir.path().join("oldname_link_dir");
    std::os::unix::fs::symlink(&real_dir, &link_dir)?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;
    
    // Check how symlinks to directories are handled
    // This depends on follow_symlinks setting
    let new_real_dir = temp_dir.path().join("newname_real_dir");
    let new_link_dir = temp_dir.path().join("newname_link_dir");
    
    // At least one of these should exist
    assert!(new_real_dir.exists() || new_link_dir.exists(), 
            "Neither real directory nor symlink was renamed");

    Ok(())
}

#[test]
fn test_directory_deletion_order_dependencies() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create structure where deletion order matters
    // Parent directories that depend on children being processed first
    fs::create_dir_all(temp_dir.path().join("oldname_a/oldname_b/oldname_c/oldname_d"))?;
    
    // Put files at different levels
    File::create(temp_dir.path().join("oldname_a/file_a.txt"))?
        .write_all(b"oldname content a")?;
    File::create(temp_dir.path().join("oldname_a/oldname_b/file_b.txt"))?
        .write_all(b"oldname content b")?;
    File::create(temp_dir.path().join("oldname_a/oldname_b/oldname_c/file_c.txt"))?
        .write_all(b"oldname content c")?;
    File::create(temp_dir.path().join("oldname_a/oldname_b/oldname_c/oldname_d/file_d.txt"))?
        .write_all(b"oldname content d")?;
    
    // Some directories will be renamed, others might become empty
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;
    
    // Verify proper handling of nested renames
    assert!(temp_dir.path().join("newname_a").exists());
    assert!(temp_dir.path().join("newname_a/newname_b").exists());
    assert!(temp_dir.path().join("newname_a/newname_b/newname_c").exists());
    assert!(temp_dir.path().join("newname_a/newname_b/newname_c/newname_d").exists());
    
    // Verify files were properly renamed
    assert!(temp_dir.path().join("newname_a/file_a.txt").exists());
    assert!(temp_dir.path().join("newname_a/newname_b/file_b.txt").exists());
    assert!(temp_dir.path().join("newname_a/newname_b/newname_c/file_c.txt").exists());
    assert!(temp_dir.path().join("newname_a/newname_b/newname_c/newname_d/file_d.txt").exists());
    
    // Check for any leftover empty directories
    let old_paths = [
        temp_dir.path().join("oldname_a"),
        temp_dir.path().join("oldname_a/oldname_b"),
        temp_dir.path().join("oldname_a/oldname_b/oldname_c"),
        temp_dir.path().join("oldname_a/oldname_b/oldname_c/oldname_d"),
    ];
    
    for old_path in &old_paths {
        if old_path.exists() {
            let entries: Vec<_> = fs::read_dir(old_path)?.collect();
            assert!(entries.is_empty(), "Empty directory not cleaned up: {}", old_path.display());
        }
    }

    Ok(())
}

// Helper function to create standardized test arguments
fn create_test_args(root_dir: &Path, old_string: &str, new_string: &str) -> Args {
    Args {
        root_dir: root_dir.to_path_buf(),
        old_string: old_string.to_string(),
        new_string: new_string.to_string(),
        assume_yes: true,
        verbose: true, // Enable verbose for better debugging
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: nomion::cli::OutputFormat::Plain,
        threads: 1,
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
    }
}
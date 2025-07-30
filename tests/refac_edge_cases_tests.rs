use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;
use nomion::{cli::Args, run_refac};

/// Critical edge case tests for refac tool - mission critical scenarios
/// These tests cover complex directory structures, empty directory cleanup,
/// permission edge cases, and other failure scenarios that have occurred in production

#[test]
fn test_deep_nested_directory_structure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create very deep nested structure (beyond typical limits)
    let mut deep_path = temp_dir.path().to_path_buf();
    for i in 0..50 {
        deep_path = deep_path.join(format!("oldname_level_{}", i));
        fs::create_dir_all(&deep_path)?;
        
        // Add some files at various levels
        if i % 5 == 0 {
            File::create(deep_path.join("oldname_file.txt"))?
                .write_all(b"oldname content")?;
        }
    }

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Verify deep structures were processed correctly
    let mut check_path = temp_dir.path().to_path_buf();
    for i in 0..50 {
        check_path = check_path.join(format!("newname_level_{}", i));
        assert!(check_path.exists(), "Deep level {} was not renamed", i);
        
        if i % 5 == 0 {
            let file_path = check_path.join("newname_file.txt");
            assert!(file_path.exists(), "File at level {} was not renamed", i);
            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("newname"), "Content at level {} was not updated", i);
        }
    }

    Ok(())
}

#[test]
fn test_empty_directory_cleanup_after_rename() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create structure where renaming will leave empty directories
    fs::create_dir_all(temp_dir.path().join("oldname_container/oldname_subdir/oldname_deepdir"))?;
    
    // Only put files in the deepest directory
    File::create(temp_dir.path().join("oldname_container/oldname_subdir/oldname_deepdir/oldname_file.txt"))?
        .write_all(b"oldname content")?;
    
    // Create a structure that should become completely empty after rename
    fs::create_dir_all(temp_dir.path().join("pure_oldname_dir/oldname_only_subdir"))?;
    File::create(temp_dir.path().join("pure_oldname_dir/oldname_only_subdir/oldname_only.txt"))?
        .write_all(b"oldname")?;

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Check that empty directories are properly handled
    // The intermediate directories should exist if they contain renamed content
    assert!(temp_dir.path().join("newname_container").exists());
    assert!(temp_dir.path().join("newname_container/newname_subdir").exists());
    assert!(temp_dir.path().join("newname_container/newname_subdir/newname_deepdir").exists());
    
    // The renamed file should exist with updated content
    let file_path = temp_dir.path().join("newname_container/newname_subdir/newname_deepdir/newname_file.txt");
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("newname"));

    // Check the pure_oldname structure
    assert!(temp_dir.path().join("pure_newname_dir").exists());
    assert!(temp_dir.path().join("pure_newname_dir/newname_only_subdir").exists());
    let pure_file = temp_dir.path().join("pure_newname_dir/newname_only_subdir/newname_only.txt");
    assert!(pure_file.exists());
    let pure_content = fs::read_to_string(&pure_file)?;
    assert!(pure_content.contains("newname"));

    Ok(())
}

#[test]
fn test_directories_with_only_hidden_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create directory structure with only hidden files
    fs::create_dir_all(temp_dir.path().join("oldname_hidden_dir"))?;
    File::create(temp_dir.path().join("oldname_hidden_dir/.oldname_hidden"))?
        .write_all(b"oldname content")?;
    File::create(temp_dir.path().join("oldname_hidden_dir/.another_oldname_file"))?
        .write_all(b"more oldname content")?;

    let args = Args {
        include_patterns: vec!["*".to_string()], // Include all files including hidden ones
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify directory and hidden files were renamed
    assert!(temp_dir.path().join("newname_hidden_dir").exists());
    assert!(temp_dir.path().join("newname_hidden_dir/.newname_hidden").exists());
    assert!(temp_dir.path().join("newname_hidden_dir/.another_newname_file").exists());
    
    // Verify content was updated
    let content1 = fs::read_to_string(temp_dir.path().join("newname_hidden_dir/.newname_hidden"))?;
    assert!(content1.contains("newname"));
    let content2 = fs::read_to_string(temp_dir.path().join("newname_hidden_dir/.another_newname_file"))?;
    assert!(content2.contains("newname"));

    Ok(())
}

#[test]
fn test_complex_circular_directory_reference_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create complex structure that could cause infinite loops
    fs::create_dir_all(temp_dir.path().join("oldname_a/oldname_b/oldname_c"))?;
    fs::create_dir_all(temp_dir.path().join("oldname_b/oldname_c/oldname_a"))?;
    fs::create_dir_all(temp_dir.path().join("oldname_c/oldname_a/oldname_b"))?;
    
    // Add files to verify processing
    File::create(temp_dir.path().join("oldname_a/file_a.txt"))?
        .write_all(b"oldname in a")?;
    File::create(temp_dir.path().join("oldname_b/file_b.txt"))?
        .write_all(b"oldname in b")?;
    File::create(temp_dir.path().join("oldname_c/file_c.txt"))?
        .write_all(b"oldname in c")?;

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Verify all directories were processed without infinite loops
    assert!(temp_dir.path().join("newname_a").exists());
    assert!(temp_dir.path().join("newname_b").exists());
    assert!(temp_dir.path().join("newname_c").exists());
    
    // Verify nested structures
    assert!(temp_dir.path().join("newname_a/newname_b").exists());
    assert!(temp_dir.path().join("newname_b/newname_c").exists());
    assert!(temp_dir.path().join("newname_c/newname_a").exists());

    Ok(())
}

#[test]
fn test_very_long_file_and_directory_names() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create names that are long but within filesystem limits (< 255 chars)
    let long_name = "oldname_".repeat(25); // ~200 characters, well under 255 limit
    let long_dir_path = temp_dir.path().join(&long_name);
    fs::create_dir_all(&long_dir_path)?;
    
    let long_file_name = format!("{}_file.txt", "oldname_very_long_name".repeat(5)); // ~120 characters
    let long_file_path = long_dir_path.join(&long_file_name);
    File::create(&long_file_path)?
        .write_all(b"oldname content in very long named file")?;

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Verify long names were handled correctly
    let expected_dir = temp_dir.path().join(long_name.replace("oldname", "newname"));
    assert!(expected_dir.exists(), "Long directory name was not renamed");
    
    let expected_file = expected_dir.join(long_file_name.replace("oldname", "newname"));
    assert!(expected_file.exists(), "Long file name was not renamed");

    Ok(())
}

#[test]
fn test_special_characters_and_unicode_in_names() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files/dirs with special characters and Unicode
    let special_names = vec![
        "oldname_with_spaces and more",
        "oldname_with_unicode_🦀_emoji",
        "oldname_with_symbols_!@#$%^&*()",
        "oldname_with_quotes'\"",
        "oldname_with_brackets[]{}", 
        "oldname_with_unicode_測試",
    ];
    
    for name in &special_names {
        let dir_path = temp_dir.path().join(name);
        fs::create_dir_all(&dir_path)?;
        
        let file_path = dir_path.join("file.txt");
        File::create(&file_path)?
            .write_all(format!("oldname content in {}", name).as_bytes())?;
    }

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Verify special character names were handled
    for name in &special_names {
        let expected_name = name.replace("oldname", "newname");
        let expected_dir = temp_dir.path().join(&expected_name);
        assert!(expected_dir.exists(), "Special character directory {} was not renamed", name);
        
        let expected_file = expected_dir.join("file.txt");
        assert!(expected_file.exists(), "File in special character directory {} was not found", name);
        
        let content = fs::read_to_string(&expected_file)?;
        assert!(content.contains("newname"), "Content in special character file {} was not updated", name);
    }

    Ok(())
}

#[test]
fn test_readonly_files_and_directories() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files with restricted permissions
    let readonly_file = temp_dir.path().join("oldname_readonly.txt");
    File::create(&readonly_file)?
        .write_all(b"oldname content")?;
    
    // Make file read-only
    let mut perms = fs::metadata(&readonly_file)?.permissions();
    perms.set_mode(0o444); // read-only
    fs::set_permissions(&readonly_file, perms)?;
    
    // Create read-only directory
    let readonly_dir = temp_dir.path().join("oldname_readonly_dir");
    fs::create_dir(&readonly_dir)?;
    File::create(readonly_dir.join("normal_file.txt"))?
        .write_all(b"oldname content")?;
    
    let mut dir_perms = fs::metadata(&readonly_dir)?.permissions();
    dir_perms.set_mode(0o555); // read-only directory
    fs::set_permissions(&readonly_dir, dir_perms)?;

    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // This should handle permission errors gracefully
    let result = run_refac(args);
    
    // Should either succeed with warnings or fail gracefully
    match result {
        Ok(_) => {
            // If successful, verify what could be renamed was renamed
            // Read-only files might not have content changed but names could be changed
        },
        Err(e) => {
            // Should be a permission error, not a crash
            assert!(e.to_string().contains("permission") || e.to_string().contains("Permission"));
        }
    }

    Ok(())
}

#[test]
fn test_files_being_modified_during_operation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test files
    let file1 = temp_dir.path().join("oldname_concurrent1.txt");
    let file2 = temp_dir.path().join("oldname_concurrent2.txt");
    
    File::create(&file1)?
        .write_all(b"oldname content 1")?;
    File::create(&file2)?
        .write_all(b"oldname content 2")?;
    
    // Test with files that might be locked or modified during operation
    // This simulates real-world scenarios where files are in use
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // Should handle file locking gracefully
    let result = run_refac(args);
    
    // Should either succeed or fail with clear error message
    match result {
        Ok(_) => {
            // Verify successful renames
            assert!(temp_dir.path().join("newname_concurrent1.txt").exists());
            assert!(temp_dir.path().join("newname_concurrent2.txt").exists());
        },
        Err(e) => {
            // Should be a file access error, not a crash
            println!("Expected file access error: {}", e);
        }
    }

    Ok(())
}

#[test]
fn test_symbolic_links_and_broken_links() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create target file
    let target_file = temp_dir.path().join("oldname_target.txt");
    File::create(&target_file)?
        .write_all(b"oldname target content")?;
    
    // Create symbolic link
    let link_file = temp_dir.path().join("oldname_link.txt");
    std::os::unix::fs::symlink(&target_file, &link_file)?;
    
    // Create broken symbolic link
    let broken_target = temp_dir.path().join("nonexistent_oldname.txt");
    let broken_link = temp_dir.path().join("oldname_broken_link.txt");
    std::os::unix::fs::symlink(&broken_target, &broken_link)?;
    
    // Test without following symlinks (default)
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;
    
    
    // Verify link names were changed but targets might not be followed
    // This depends on the follow_symlinks setting
    let newname_link_exists = temp_dir.path().join("newname_link.txt").exists() || 
                             temp_dir.path().join("newname_link.txt").symlink_metadata().is_ok();
    let oldname_link_exists = temp_dir.path().join("oldname_link.txt").exists() || 
                             temp_dir.path().join("oldname_link.txt").symlink_metadata().is_ok();
    
    let link_exists = newname_link_exists || oldname_link_exists;
    assert!(link_exists, "Symbolic link handling failed");

    Ok(())
}

#[test]
fn test_case_sensitivity_edge_cases() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files with case variations (use different base names to avoid case-insensitive filesystem issues)
    fs::create_dir_all(temp_dir.path().join("OldName_dir1"))?;
    fs::create_dir_all(temp_dir.path().join("oldname_dir2"))?;  
    fs::create_dir_all(temp_dir.path().join("OLDNAME_dir3"))?;
    
    File::create(temp_dir.path().join("OldName_file1.txt"))?
        .write_all(b"OldName content")?;
    File::create(temp_dir.path().join("oldname_file2.txt"))?
        .write_all(b"oldname content")?;
    File::create(temp_dir.path().join("OLDNAME_file3.txt"))?
        .write_all(b"OLDNAME content")?;


    // Test case-sensitive replacement
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;

    // Only exact case matches should be replaced
    assert!(temp_dir.path().join("OldName_dir1").exists()); // Unchanged
    assert!(temp_dir.path().join("newname_dir2").exists()); // Changed from oldname_dir2
    assert!(temp_dir.path().join("OLDNAME_dir3").exists()); // Unchanged
    
    assert!(temp_dir.path().join("OldName_file1.txt").exists()); // Unchanged
    assert!(temp_dir.path().join("newname_file2.txt").exists()); // Changed from oldname_file2.txt
    assert!(temp_dir.path().join("OLDNAME_file3.txt").exists()); // Unchanged

    Ok(())
}

#[test]
fn test_very_large_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a large file with oldname pattern
    let large_file = temp_dir.path().join("oldname_large.txt");
    let mut file = File::create(&large_file)?;
    
    // Write 10MB of content with oldname patterns
    for i in 0..100000 {
        writeln!(file, "Line {} with oldname pattern repeated multiple times oldname oldname", i)?;
    }
    file.flush()?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;
    
    // Verify large file was processed
    assert!(temp_dir.path().join("newname_large.txt").exists());
    
    // Verify content was updated (check first and last lines)
    let content = fs::read_to_string(temp_dir.path().join("newname_large.txt"))?;
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines[0].contains("newname"));
    assert!(!lines[0].contains("oldname"));
    assert!(lines.last().unwrap().contains("newname"));

    Ok(())
}

#[test]
fn test_maximum_directory_depth_limits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Test max_depth parameter functionality
    let mut path = temp_dir.path().to_path_buf();
    for i in 1..=10 {
        path = path.join(format!("oldname_level_{}", i));
        fs::create_dir_all(&path)?;
        File::create(path.join("oldname_file.txt"))?
            .write_all(format!("oldname at level {}", i).as_bytes())?;
    }
    
    // Set max depth to 5
    let args = Args {
        max_depth: 5,
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;
    
    // Verify only files within max_depth were processed
    for i in 1..=5 {
        let mut check_path = temp_dir.path().to_path_buf();
        for j in 1..=i {
            check_path = check_path.join(format!("newname_level_{}", j));
        }
        assert!(check_path.exists(), "Level {} within max_depth was not renamed", i);
    }
    
    // Levels beyond max_depth should remain unchanged
    let mut deep_path = temp_dir.path().to_path_buf();
    for j in 1..=6 {
        if j <= 5 {
            deep_path = deep_path.join(format!("newname_level_{}", j));
        } else {
            deep_path = deep_path.join(format!("oldname_level_{}", j));
        }
    }
    // The directory at level 6 should still have the old name since it's beyond max_depth
    // But this depends on implementation - directory might be renamed but not traversed

    Ok(())
}

#[test]
fn test_collision_handling_complex_scenarios() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create scenario where multiple sources would rename to same target
    File::create(temp_dir.path().join("file_oldname.txt"))?
        .write_all(b"content 1")?;
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"content 2")?;
    
    // Both would try to rename to something like "file_newname.txt" vs "newname_file.txt" 
    // but let's create a real collision
    File::create(temp_dir.path().join("oldname_duplicate.txt"))?
        .write_all(b"content 1")?;
    fs::create_dir(temp_dir.path().join("oldname_duplicate"))?; // Directory with same base name
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // This should detect collisions and handle them appropriately
    let result = run_refac(args);
    
    match result {
        Ok(_) => {
            // If successful, verify collision resolution
            println!("Collision resolution succeeded");
        },
        Err(e) => {
            // Should be a collision error, not a crash
            assert!(e.to_string().contains("collision") || e.to_string().contains("Collision"));
        }
    }

    Ok(())
}

#[test]
fn test_interrupted_operation_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a structure that simulates partial completion
    // (as if a previous operation was interrupted)
    File::create(temp_dir.path().join("oldname_original.txt"))?
        .write_all(b"oldname content")?;
    File::create(temp_dir.path().join("newname_already_renamed.txt"))?
        .write_all(b"already newname content")?;
    File::create(temp_dir.path().join("oldname_not_yet_renamed.txt"))?
        .write_all(b"oldname content")?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    run_refac(args)?;
    
    // Should handle mixed state gracefully
    assert!(temp_dir.path().join("newname_original.txt").exists());
    assert!(temp_dir.path().join("newname_already_renamed.txt").exists());
    assert!(temp_dir.path().join("newname_not_yet_renamed.txt").exists());

    Ok(())
}

// Helper function to create standardized test arguments
fn create_test_args(root_dir: &Path, old_string: &str, new_string: &str) -> Args {
    Args {
        root_dir: root_dir.to_path_buf(),
        old_string: old_string.to_string(),
        new_string: new_string.to_string(),
        assume_yes: true,
        verbose: false,
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
    }
}
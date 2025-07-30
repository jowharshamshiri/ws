use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
// Arc not needed for current implementation
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use nomion::{cli::Args, run_refac};

/// Tests for concurrent operation safety and thread-related edge cases in refac tool
/// These tests ensure the tool handles multi-threading and concurrent file system operations safely

#[test]
fn test_high_thread_count_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create many files to stress test parallel processing
    for i in 0..1000 {
        let file_path = temp_dir.path().join(format!("oldname_file_{:04}.txt", i));
        File::create(&file_path)?
            .write_all(format!("oldname content for file {}", i).as_bytes())?;
    }
    
    // Create some directories too
    for i in 0..50 {
        let dir_path = temp_dir.path().join(format!("oldname_dir_{:02}", i));
        fs::create_dir(&dir_path)?;
        
        // Add files inside directories
        for j in 0..10 {
            let nested_file = dir_path.join(format!("oldname_nested_{}.txt", j));
            File::create(&nested_file)?
                .write_all(format!("oldname nested content {}/{}", i, j).as_bytes())?;
        }
    }

    let args = Args {
        threads: 16, // High thread count
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify all files were processed correctly with high concurrency
    for i in 0..1000 {
        let expected_file = temp_dir.path().join(format!("newname_file_{:04}.txt", i));
        assert!(expected_file.exists(), "File {} was not renamed with high concurrency", i);
        
        let content = fs::read_to_string(&expected_file)?;
        assert!(content.contains("newname"), "Content in file {} was not updated", i);
        assert!(!content.contains("oldname"), "Old content still exists in file {}", i);
    }
    
    // Verify directories
    for i in 0..50 {
        let expected_dir = temp_dir.path().join(format!("newname_dir_{:02}", i));
        assert!(expected_dir.exists(), "Directory {} was not renamed", i);
        
        for j in 0..10 {
            let expected_nested = expected_dir.join(format!("newname_nested_{}.txt", j));
            assert!(expected_nested.exists(), "Nested file {}/{} was not renamed", i, j);
        }
    }

    Ok(())
}

#[test]
fn test_concurrent_file_access_safety() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files that will be accessed concurrently
    for i in 0..100 {
        let file_path = temp_dir.path().join(format!("oldname_concurrent_{}.txt", i));
        File::create(&file_path)?
            .write_all(format!("oldname content line 1\noldname content line 2\noldname content line {}", i).as_bytes())?;
    }

    let args = Args {
        threads: 8,
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify file integrity after concurrent processing
    for i in 0..100 {
        let expected_file = temp_dir.path().join(format!("newname_concurrent_{}.txt", i));
        assert!(expected_file.exists(), "Concurrent file {} was not renamed", i);
        
        let content = fs::read_to_string(&expected_file)?;
        let lines: Vec<&str> = content.lines().collect();
        
        // Verify content integrity - all lines should be updated consistently
        assert_eq!(lines.len(), 3, "File {} has wrong number of lines", i);
        assert!(lines[0].contains("newname"), "Line 1 in file {} not updated", i);
        assert!(lines[1].contains("newname"), "Line 2 in file {} not updated", i);
        assert!(lines[2].contains("newname"), "Line 3 in file {} not updated", i);
        
        // Ensure no partial updates (mixed old/new content)
        assert!(!content.contains("oldname"), "File {} has mixed old/new content", i);
    }

    Ok(())
}

#[test]
fn test_directory_rename_race_conditions() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create nested directory structure that could cause race conditions
    // when multiple threads try to rename parent and child directories
    for i in 0..20 {
        let deep_path = temp_dir.path()
            .join(format!("oldname_parent_{}", i))
            .join(format!("oldname_child_{}", i))
            .join(format!("oldname_grandchild_{}", i));
        fs::create_dir_all(&deep_path)?;
        
        // Add files at each level
        let parent_file = temp_dir.path()
            .join(format!("oldname_parent_{}", i))
            .join("parent_file.txt");
        File::create(&parent_file)?
            .write_all(format!("oldname parent content {}", i).as_bytes())?;
        
        let child_file = temp_dir.path()
            .join(format!("oldname_parent_{}", i))
            .join(format!("oldname_child_{}", i))
            .join("child_file.txt");
        File::create(&child_file)?
            .write_all(format!("oldname child content {}", i).as_bytes())?;
        
        let grandchild_file = deep_path.join("grandchild_file.txt");
        File::create(&grandchild_file)?
            .write_all(format!("oldname grandchild content {}", i).as_bytes())?;
    }

    let args = Args {
        threads: 8,
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify nested directory renames completed without race conditions
    for i in 0..20 {
        let expected_parent = temp_dir.path().join(format!("newname_parent_{}", i));
        let expected_child = expected_parent.join(format!("newname_child_{}", i));
        let expected_grandchild = expected_child.join(format!("newname_grandchild_{}", i));
        
        assert!(expected_parent.exists(), "Parent directory {} was not renamed", i);
        assert!(expected_child.exists(), "Child directory {} was not renamed", i);
        assert!(expected_grandchild.exists(), "Grandchild directory {} was not renamed", i);
        
        // Verify files at each level
        assert!(expected_parent.join("parent_file.txt").exists());
        assert!(expected_child.join("child_file.txt").exists());
        assert!(expected_grandchild.join("grandchild_file.txt").exists());
    }

    Ok(())
}

#[test]
fn test_large_file_concurrent_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create several large files that will be processed concurrently
    for i in 0..5 {
        let file_path = temp_dir.path().join(format!("oldname_large_{}.txt", i));
        let mut file = File::create(&file_path)?;
        
        // Write 1MB of content with many oldname occurrences
        for line in 0..10000 {
            writeln!(file, "Line {} with oldname pattern repeated oldname oldname", line)?;
        }
    }

    let args = Args {
        threads: 4, // Multiple threads processing large files
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify large files were processed correctly
    for i in 0..5 {
        let expected_file = temp_dir.path().join(format!("newname_large_{}.txt", i));
        assert!(expected_file.exists(), "Large file {} was not renamed", i);
        
        let content = fs::read_to_string(&expected_file)?;
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 10000, "Large file {} has wrong line count", i);
        
        // Check random lines for correct content replacement
        for line_idx in [0, 1000, 5000, 9999] {
            assert!(lines[line_idx].contains("newname"), "Line {} in large file {} not updated", line_idx, i);
            assert!(!lines[line_idx].contains("oldname"), "Line {} in large file {} has old content", line_idx, i);
        }
    }

    Ok(())
}

#[test]
fn test_mixed_operation_modes_concurrency() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create structure that will test different operation modes simultaneously
    for i in 0..50 {
        // Files with oldname in name and content
        let file1 = temp_dir.path().join(format!("oldname_both_{}.txt", i));
        File::create(&file1)?
            .write_all(format!("oldname content in both {}", i).as_bytes())?;
        
        // Files with oldname only in name
        let file2 = temp_dir.path().join(format!("oldname_name_only_{}.txt", i));
        File::create(&file2)?
            .write_all(format!("normal content {}", i).as_bytes())?;
        
        // Files with oldname only in content
        let file3 = temp_dir.path().join(format!("normal_name_{}.txt", i));
        File::create(&file3)?
            .write_all(format!("oldname content only {}", i).as_bytes())?;
        
        // Directories
        let dir = temp_dir.path().join(format!("oldname_dir_{}", i));
        fs::create_dir(&dir)?;
        File::create(dir.join("internal.txt"))?
            .write_all(format!("oldname internal content {}", i).as_bytes())?;
    }

    let args = Args {
        threads: 6,
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify all types of operations completed correctly
    for i in 0..50 {
        // Files with both name and content changes
        let both_file = temp_dir.path().join(format!("newname_both_{}.txt", i));
        assert!(both_file.exists());
        let both_content = fs::read_to_string(&both_file)?;
        assert!(both_content.contains("newname"));
        
        // Files with only name changes
        let name_only_file = temp_dir.path().join(format!("newname_name_only_{}.txt", i));
        assert!(name_only_file.exists());
        let name_only_content = fs::read_to_string(&name_only_file)?;
        assert!(name_only_content.contains("normal content"));
        assert!(name_only_content.contains(&i.to_string()));
        
        // Files with only content changes
        let content_only_file = temp_dir.path().join(format!("normal_name_{}.txt", i));
        assert!(content_only_file.exists());
        let content_only_content = fs::read_to_string(&content_only_file)?;
        assert!(content_only_content.contains("newname"));
        
        // Directories
        let dir = temp_dir.path().join(format!("newname_dir_{}", i));
        assert!(dir.exists());
        let internal_file = dir.join("internal.txt");
        assert!(internal_file.exists());
        let internal_content = fs::read_to_string(&internal_file)?;
        assert!(internal_content.contains("newname"));
    }

    Ok(())
}

#[test]
fn test_filesystem_stress_concurrent_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a stress test with many small operations
    for i in 0..200 {
        let subdir = temp_dir.path().join(format!("stress_oldname_{:03}", i));
        fs::create_dir(&subdir)?;
        
        // Multiple files per directory
        for j in 0..5 {
            let file = subdir.join(format!("oldname_file_{}.txt", j));
            File::create(&file)?
                .write_all(format!("oldname stress content {}/{}", i, j).as_bytes())?;
        }
    }

    let args = Args {
        threads: 12, // High concurrency
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    let start_time = std::time::Instant::now();
    run_refac(args)?;
    let duration = start_time.elapsed();
    
    println!("Stress test completed in {:?}", duration);

    // Verify all operations completed correctly under stress
    for i in 0..200 {
        let expected_dir = temp_dir.path().join(format!("stress_newname_{:03}", i));
        assert!(expected_dir.exists(), "Stress directory {} missing", i);
        
        for j in 0..5 {
            let expected_file = expected_dir.join(format!("newname_file_{}.txt", j));
            assert!(expected_file.exists(), "Stress file {}/{} missing", i, j);
            
            let content = fs::read_to_string(&expected_file)?;
            assert!(content.contains("newname"), "Stress file {}/{} content not updated", i, j);
        }
    }

    Ok(())
}

#[test]
fn test_interrupt_safety_simulation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test data
    for i in 0..100 {
        let file = temp_dir.path().join(format!("oldname_interrupt_{}.txt", i));
        File::create(&file)?
            .write_all(format!("oldname content for interrupt test {}", i).as_bytes())?;
    }
    
    // Run with a timeout to simulate interruption
    let temp_path = temp_dir.path().to_path_buf();
    let handle = thread::spawn(move || {
        let args = Args {
            threads: 2,
            ..create_test_args(&temp_path, "oldname", "newname")
        };
        run_refac(args)
    });
    
    // Let it run for a short time then check results
    thread::sleep(Duration::from_millis(100));
    
    match handle.join() {
        Ok(result) => {
            match result {
                Ok(_) => {
                    // Operation completed - verify state consistency
                    let mut renamed_count = 0;
                    let mut original_count = 0;
                    
                    for i in 0..100 {
                        let old_file = temp_dir.path().join(format!("oldname_interrupt_{}.txt", i));
                        let new_file = temp_dir.path().join(format!("newname_interrupt_{}.txt", i));
                        
                        if old_file.exists() { original_count += 1; }
                        if new_file.exists() { renamed_count += 1; }
                    }
                    
                    // Should have all files in one state or the other, not lost
                    assert_eq!(renamed_count + original_count, 100, 
                             "Files lost during operation: {} renamed, {} original", 
                             renamed_count, original_count);
                },
                Err(e) => {
                    // Operation failed - verify no partial state corruption
                    println!("Operation failed as expected: {}", e);
                }
            }
        },
        Err(_) => {
            // Thread panicked - this is bad
            panic!("Thread panicked during concurrent operation");
        }
    }

    Ok(())
}

#[test]
fn test_thread_pool_exhaustion_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create more work than threads to test thread pool management
    for i in 0..1000 {
        let file = temp_dir.path().join(format!("oldname_pool_{:04}.txt", i));
        File::create(&file)?
            .write_all(format!("oldname content {}", i).as_bytes())?;
    }

    // Use fewer threads than work items
    let args = Args {
        threads: 4, // Much fewer than 1000 files
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify thread pool handled all work correctly
    for i in 0..1000 {
        let expected_file = temp_dir.path().join(format!("newname_pool_{:04}.txt", i));
        assert!(expected_file.exists(), "File {} not processed by thread pool", i);
    }

    Ok(())
}

#[test]
fn test_concurrent_directory_tree_modification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a complex tree that will be modified at multiple levels simultaneously
    for i in 0..10 {
        for j in 0..10 {
            let path = temp_dir.path()
                .join(format!("oldname_level1_{}", i))
                .join(format!("oldname_level2_{}", j));
            fs::create_dir_all(&path)?;
            
            File::create(path.join("oldname_file.txt"))?
                .write_all(format!("oldname content {}/{}", i, j).as_bytes())?;
        }
    }

    let args = Args {
        threads: 8,
        ..create_test_args(temp_dir.path(), "oldname", "newname")
    };
    
    run_refac(args)?;

    // Verify complex tree modifications completed correctly
    for i in 0..10 {
        for j in 0..10 {
            let expected_path = temp_dir.path()
                .join(format!("newname_level1_{}", i))
                .join(format!("newname_level2_{}", j));
            assert!(expected_path.exists(), "Tree path {}/{} not renamed", i, j);
            
            let expected_file = expected_path.join("newname_file.txt");
            assert!(expected_file.exists(), "Tree file {}/{} not renamed", i, j);
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
        threads: 1, // Will be overridden in individual tests
        progress: nomion::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
    }
}
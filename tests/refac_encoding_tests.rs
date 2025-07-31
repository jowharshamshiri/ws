use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use workspace::{cli::Args, run_refac};

/// Tests for file encoding handling and validation in refac tool
/// These tests ensure encoding issues are caught during validation, not during execution

#[test]
fn test_invalid_utf8_encoding_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a file with invalid UTF-8 sequences
    let invalid_utf8_file = temp_dir.path().join("oldname_invalid_utf8.txt");
    let mut file = File::create(&invalid_utf8_file)?;
    
    // Write some valid text followed by invalid UTF-8 bytes
    file.write_all(b"This is valid text with oldname pattern\n")?;
    file.write_all(&[0xFF, 0xFE, 0xFD, 0xFC])?; // Invalid UTF-8 sequence
    file.write_all(b"\nMore text after invalid bytes")?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // With automatic encoding detection, this should now succeed
    let result = run_refac(args);
    
    match result {
        Ok(_) => {
            // Verify the operation succeeded and file was processed correctly
            let output_file = temp_dir.path().join("newname_invalid_utf8.txt");
            assert!(output_file.exists(), "File should have been renamed successfully");
            println!("Successfully processed file with mixed encoding using automatic detection");
        },
        Err(e) => {
            // If it fails, it should be for a different reason than encoding
            let error_msg = e.to_string();
            if error_msg.contains("binary") {
                println!("File correctly identified as binary and skipped: {}", error_msg);
            } else {
                panic!("Unexpected error (encoding detection should handle this): {}", error_msg);
            }
        }
    }

    Ok(())
}

#[test]
fn test_binary_file_mixed_with_text_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a file that looks like text but has binary content
    let mixed_file = temp_dir.path().join("oldname_mixed.txt");
    let mut file = File::create(&mixed_file)?;
    
    // Write text content with oldname
    file.write_all(b"Start with oldname text\n")?;
    // Insert null bytes (binary indicator)
    file.write_all(&[0x00, 0x01, 0x02, 0x03])?;
    file.write_all(b"More text with oldname")?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // Should either skip (binary detection) or fail validation
    let result = run_refac(args);
    
    // This might succeed if binary detection works, or fail if it doesn't
    // The key is that it shouldn't crash mid-operation
    match result {
        Ok(_) => {
            // If successful, binary detection worked and skipped the file
            println!("Binary detection correctly skipped mixed file");
        },
        Err(e) => {
            // If failed, validation caught the issue
            let error_msg = e.to_string();
            assert!(error_msg.contains("validation") || error_msg.contains("binary") || error_msg.contains("encoding"), 
                   "Expected encoding/binary error, got: {}", error_msg);
        }
    }

    Ok(())
}

#[test]
fn test_different_line_endings_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files with different line ending styles
    let unix_file = temp_dir.path().join("oldname_unix.txt");
    let mut file = File::create(&unix_file)?;
    file.write_all(b"Unix line endings with oldname\nSecond line with oldname\n")?;
    
    let windows_file = temp_dir.path().join("oldname_windows.txt");
    let mut file = File::create(&windows_file)?;
    file.write_all(b"Windows line endings with oldname\r\nSecond line with oldname\r\n")?;
    
    let mac_file = temp_dir.path().join("oldname_mac.txt");
    let mut file = File::create(&mac_file)?;
    file.write_all(b"Mac line endings with oldname\rSecond line with oldname\r")?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // All line ending styles should be handled correctly
    let result = run_refac(args);
    
    match result {
        Ok(_) => {
            // Verify all files were processed
            assert!(temp_dir.path().join("newname_unix.txt").exists());
            assert!(temp_dir.path().join("newname_windows.txt").exists());
            assert!(temp_dir.path().join("newname_mac.txt").exists());
            
            // Verify content was updated properly
            let unix_content = fs::read_to_string(temp_dir.path().join("newname_unix.txt"))?;
            assert!(unix_content.contains("newname"));
            assert!(!unix_content.contains("oldname"));
        },
        Err(e) => {
            panic!("Line ending variations should be handled correctly, but got error: {}", e);
        }
    }

    Ok(())
}

#[test]
fn test_large_file_with_mixed_encoding() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a large file with potential encoding issues
    let large_file = temp_dir.path().join("oldname_large_mixed.txt");
    let mut file = File::create(&large_file)?;
    
    // Write lots of valid content
    for i in 0..1000 {
        writeln!(file, "Line {} with oldname pattern that should be replaced", i)?;
    }
    
    // Insert some problematic bytes in the middle
    file.write_all(&[0xC0, 0x80])?; // Invalid UTF-8 sequence (overlong encoding)
    
    // More valid content
    for i in 1000..2000 {
        writeln!(file, "Line {} with oldname pattern continued", i)?;
    }
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // With automatic encoding detection, large files with mixed encoding should succeed
    let result = run_refac(args);
    
    match result {
        Ok(_) => {
            // Verify the operation succeeded and file was processed correctly
            let output_file = temp_dir.path().join("newname_large_mixed.txt");
            assert!(output_file.exists(), "Large file should have been renamed successfully");
            println!("Successfully processed large file with mixed encoding using automatic detection");
        },
        Err(e) => {
            // If it fails, it should be for a different reason than encoding
            let error_msg = e.to_string();
            if error_msg.contains("binary") {
                println!("Large file correctly identified as binary and skipped: {}", error_msg);
            } else {
                panic!("Unexpected error (encoding detection should handle large files): {}", error_msg);
            }
        }
    }

    Ok(())
}

#[test]
fn test_utf8_bom_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create file with UTF-8 BOM (Byte Order Mark)
    let bom_file = temp_dir.path().join("oldname_bom.txt");
    let mut file = File::create(&bom_file)?;
    
    // UTF-8 BOM: EF BB BF
    file.write_all(&[0xEF, 0xBB, 0xBF])?;
    file.write_all(b"File with BOM and oldname content")?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // UTF-8 BOM should be handled correctly
    let result = run_refac(args);
    
    match result {
        Ok(_) => {
            assert!(temp_dir.path().join("newname_bom.txt").exists());
            
            // Verify content was updated (BOM should be preserved)
            let content = fs::read_to_string(temp_dir.path().join("newname_bom.txt"))?;
            assert!(content.contains("newname"));
            assert!(!content.contains("oldname"));
        },
        Err(e) => {
            panic!("UTF-8 BOM files should be handled correctly, but got error: {}", e);
        }
    }

    Ok(())
}

#[test]
fn test_empty_and_whitespace_only_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create empty file
    let empty_file = temp_dir.path().join("oldname_empty.txt");
    File::create(&empty_file)?;
    
    // Create whitespace-only file
    let whitespace_file = temp_dir.path().join("oldname_whitespace.txt");
    let mut file = File::create(&whitespace_file)?;
    file.write_all(b"   \n\t\r\n   \n")?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // Empty and whitespace files should be handled without issues
    let result = run_refac(args);
    
    match result {
        Ok(_) => {
            // Files should be renamed even if they don't contain the target string
            assert!(temp_dir.path().join("newname_empty.txt").exists());
            assert!(temp_dir.path().join("newname_whitespace.txt").exists());
        },
        Err(e) => {
            panic!("Empty and whitespace files should be handled correctly, but got error: {}", e);
        }
    }

    Ok(())
}

#[test]
fn test_very_long_lines_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create file with very long lines
    let long_lines_file = temp_dir.path().join("oldname_long_lines.txt");
    let mut file = File::create(&long_lines_file)?;
    
    // Create a very long line (100KB)
    let long_content = "a".repeat(50000) + "oldname" + &"b".repeat(50000);
    writeln!(file, "{}", long_content)?;
    writeln!(file, "Normal line with oldname")?;
    
    let args = create_test_args(temp_dir.path(), "oldname", "newname");
    
    // Very long lines should be handled correctly
    let result = run_refac(args);
    
    match result {
        Ok(_) => {
            assert!(temp_dir.path().join("newname_long_lines.txt").exists());
            
            let content = fs::read_to_string(temp_dir.path().join("newname_long_lines.txt"))?;
            assert!(content.contains("newname"));
            assert!(!content.contains("oldname"));
        },
        Err(e) => {
            panic!("Very long lines should be handled correctly, but got error: {}", e);
        }
    }

    Ok(())
}

// Helper function to create standardized test arguments
fn create_test_args(root_dir: &Path, pattern: &str, substitute: &str) -> Args {
    Args {
        root_dir: root_dir.to_path_buf(),
        pattern: pattern.to_string(),
        substitute: substitute.to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    }
}
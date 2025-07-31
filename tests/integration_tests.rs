use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use workspace::{cli::Args, run_refac};

/// Test utilities
mod test_utils {
    use super::*;

    pub fn create_test_structure(root: &Path) -> Result<()> {
        // Create directory structure
        fs::create_dir_all(root.join("oldname_dir1/oldname_subdir"))?;
        fs::create_dir_all(root.join("oldname_dir2/normal_subdir"))?;
        fs::create_dir_all(root.join("normal_dir/oldname_nested"))?;
        fs::create_dir_all(root.join("edge cases"))?;
        fs::create_dir_all(root.join("special-chars"))?;

        // Create files with oldname in names
        File::create(root.join("oldname_file1.txt"))?
            .write_all(b"Content with oldname inside")?;
        File::create(root.join("oldname_dir1/oldname_file2.txt"))?
            .write_all(b"Another oldname content")?;
        File::create(root.join("normal_dir/normal_file.txt"))?
            .write_all(b"No pattern content")?;
        File::create(root.join("normal_dir/content_only.txt"))?
            .write_all(b"File with oldname in content only")?;
        File::create(root.join("oldname_dir2/end_oldname.txt"))?
            .write_all(b"end with oldname")?;

        // Edge case files
        File::create(root.join("edge cases/oldname multiple.txt"))?
            .write_all(b"oldname oldname oldname")?;
        File::create(root.join("special-chars/oldname.config"))?
            .write_all(b"Complex oldname content")?;

        // Files with multiple occurrences
        File::create(root.join("oldnameoldnameoldname.txt"))?
            .write_all(b"oldname content with oldname")?;

        // Hidden files
        File::create(root.join(".oldname_hidden"))?
            .write_all(b"Hidden oldname content")?;
        fs::create_dir(root.join(".oldname_hidden_dir"))?;

        // Binary file (for testing binary detection)
        File::create(root.join("oldname_binary.bin"))?
            .write_all(&[0x00, 0x01, 0x02, b'o', b'l', b'd', b'n', b'a', b'm', b'e', 0x03, 0x04])?;

        Ok(())
    }

    pub fn count_matches(root: &Path, pattern: &str) -> usize {
        walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        return name_str.contains(pattern);
                    }
                }
                false
            })
            .count()
    }

    pub fn count_content_matches(root: &Path, pattern: &str) -> Result<usize> {
        let mut count = 0;
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry?;
            if entry.path().is_file() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.contains(pattern) {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }
}

#[test]
fn test_basic_replacement() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create simple structure
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    // Create args for renaming
    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    // Run refac
    run_refac(args)?;

    // Verify changes
    assert!(temp_dir.path().join("newname_file.txt").exists());
    assert!(temp_dir.path().join("newname_dir").exists());
    assert!(!temp_dir.path().join("oldname_file.txt").exists());
    assert!(!temp_dir.path().join("oldname_dir").exists());

    // Check content was modified
    let content = fs::read_to_string(temp_dir.path().join("newname_file.txt"))?;
    assert!(content.contains("newname"));
    assert!(!content.contains("oldname"));

    Ok(())
}

#[test]
fn test_mandatory_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test structure that will validate and execute successfully
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    // Run operation (validation is now mandatory and automatic)
    run_refac(args)?;

    // Verify changes were actually made (validation passed, so execution happened)
    assert!(!temp_dir.path().join("oldname_file.txt").exists());
    assert!(!temp_dir.path().join("oldname_dir").exists());
    assert!(temp_dir.path().join("newname_file.txt").exists());
    assert!(temp_dir.path().join("newname_dir").exists());

    // Check content was changed
    let content = fs::read_to_string(temp_dir.path().join("newname_file.txt"))?;
    assert!(content.contains("newname"));
    assert!(!content.contains("oldname"));

    Ok(())
}

#[test]
fn test_case_sensitivity() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files with different cases
    File::create(temp_dir.path().join("OldName_file.txt"))?
        .write_all(b"OldName OLDNAME oldname")?;
    fs::create_dir(temp_dir.path().join("OLDNAME_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "OldName".to_string(),
        substitute: "NewName".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args)?;

    // Check that only exact case matches were replaced
    assert!(temp_dir.path().join("NewName_file.txt").exists());
    assert!(temp_dir.path().join("OLDNAME_dir").exists()); // Should not be renamed

    let content = fs::read_to_string(temp_dir.path().join("NewName_file.txt"))?;
    assert!(content.contains("NewName")); // Should be replaced
    assert!(content.contains("OLDNAME")); // Should not be replaced
    assert!(content.contains("oldname")); // Should not be replaced

    Ok(())
}

#[test]
fn test_complex_nested_structure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    test_utils::create_test_structure(temp_dir.path())?;

    let _initial_oldname_count = test_utils::count_matches(temp_dir.path(), "oldname");
    let _initial_content_count = test_utils::count_content_matches(temp_dir.path(), "oldname")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args)?;

    // Check that most oldname occurrences in names were replaced (some may fail due to path conflicts)
    let final_oldname_count = test_utils::count_matches(temp_dir.path(), "oldname");
    let final_newname_count = test_utils::count_matches(temp_dir.path(), "newname");
    let final_content_count = test_utils::count_content_matches(temp_dir.path(), "oldname")?;

    assert!(final_oldname_count < 10, "Most file/directory names should be renamed");
    assert!(final_newname_count > 0, "No items were renamed to contain 'newname'");
    assert!(final_content_count < 5, "Most file contents should be updated");

    // Verify specific files exist
    assert!(temp_dir.path().join("newname_file1.txt").exists());
    assert!(temp_dir.path().join("newname_dir1").exists());
    assert!(temp_dir.path().join("newname_dir1/newname_subdir").exists());

    Ok(())
}

#[test]
fn test_files_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create mixed structure
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: true,
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
    };

    run_refac(args)?;

    // File should be renamed, directory should not
    assert!(temp_dir.path().join("newname_file.txt").exists());
    assert!(temp_dir.path().join("oldname_dir").exists()); // Directory unchanged
    assert!(!temp_dir.path().join("oldname_file.txt").exists());

    Ok(())
}

#[test]
fn test_dirs_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create mixed structure
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join("oldname_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: true,
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
    };

    run_refac(args)?;

    // Directory should be renamed, file content should be updated but name unchanged
    assert!(temp_dir.path().join("oldname_file.txt").exists()); // File name unchanged
    assert!(temp_dir.path().join("newname_dir").exists());
    assert!(!temp_dir.path().join("oldname_dir").exists());

    // Content should NOT be updated in dirs-only mode
    let content = fs::read_to_string(temp_dir.path().join("oldname_file.txt"))?;
    assert!(content.contains("oldname")); // Content should be unchanged

    Ok(())
}

#[test]
fn test_names_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: true,
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
    };

    run_refac(args)?;

    // File should be renamed but content unchanged
    assert!(temp_dir.path().join("newname_file.txt").exists());
    assert!(!temp_dir.path().join("oldname_file.txt").exists());

    let content = fs::read_to_string(temp_dir.path().join("newname_file.txt"))?;
    assert!(content.contains("oldname")); // Content should be unchanged

    Ok(())
}

#[test]
fn test_content_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: true,
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
    };

    run_refac(args)?;

    // File name should be unchanged but content updated
    assert!(temp_dir.path().join("oldname_file.txt").exists());
    assert!(!temp_dir.path().join("newname_file.txt").exists());

    let content = fs::read_to_string(temp_dir.path().join("oldname_file.txt"))?;
    assert!(content.contains("newname")); // Content should be updated
    assert!(!content.contains("oldname")); // Old content should be replaced

    Ok(())
}

#[test]
fn test_binary_file_handling_default() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create binary file with oldname in filename
    File::create(temp_dir.path().join("oldname_binary.bin"))?
        .write_all(&[0x00, 0x01, 0x02, b'o', b'l', b'd', b'n', b'a', b'm', b'e', 0x03, 0x04])?;
    
    // Create text file for comparison
    File::create(temp_dir.path().join("oldname_text.txt"))?
        .write_all(b"oldname text content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false, // Default: binary files are NOT renamed
    };

    run_refac(args)?;

    // Binary file should NOT be renamed (default behavior)
    assert!(!temp_dir.path().join("newname_binary.bin").exists());
    assert!(temp_dir.path().join("oldname_binary.bin").exists());
    
    // Text file should be renamed and content modified
    assert!(temp_dir.path().join("newname_text.txt").exists());
    assert!(!temp_dir.path().join("oldname_text.txt").exists());

    let text_content = fs::read_to_string(temp_dir.path().join("newname_text.txt"))?;
    assert!(text_content.contains("newname"));
    assert!(!text_content.contains("oldname"));

    Ok(())
}

#[test]
fn test_binary_file_handling_with_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create binary file with oldname in filename
    File::create(temp_dir.path().join("oldname_binary.bin"))?
        .write_all(&[0x00, 0x01, 0x02, b'o', b'l', b'd', b'n', b'a', b'm', b'e', 0x03, 0x04])?;
    
    // Create text file for comparison
    File::create(temp_dir.path().join("oldname_text.txt"))?
        .write_all(b"oldname text content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: true, // Enable binary file renaming
    };

    run_refac(args)?;

    // Both files should be renamed when binary_names flag is set
    assert!(temp_dir.path().join("newname_binary.bin").exists());
    assert!(!temp_dir.path().join("oldname_binary.bin").exists());
    assert!(temp_dir.path().join("newname_text.txt").exists());
    assert!(!temp_dir.path().join("oldname_text.txt").exists());

    // Text file content should be modified
    let text_content = fs::read_to_string(temp_dir.path().join("newname_text.txt"))?;
    assert!(text_content.contains("newname"));
    assert!(!text_content.contains("oldname"));

    // Binary file content should be unchanged
    let binary_content = fs::read(temp_dir.path().join("newname_binary.bin"))?;
    assert_eq!(binary_content, vec![0x00, 0x01, 0x02, b'o', b'l', b'd', b'n', b'a', b'm', b'e', 0x03, 0x04]);

    Ok(())
}

#[test]
fn test_binary_flag_cli_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create mixed files with pattern in names
    File::create(temp_dir.path().join("test_old.txt"))?
        .write_all(b"old content in text file")?;
    
    File::create(temp_dir.path().join("old_image.png"))?
        .write_all(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, b'o', b'l', b'd'])?; // PNG header + "old"
    
    File::create(temp_dir.path().join("old_binary.bin"))?
        .write_all(&[0x00, 0x01, 0x02, b'o', b'l', b'd', 0x03, 0x04])?;

    // Test default behavior (binary files ignored)
    let args_default = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args_default)?;

    // Only text file should be renamed and have content changed
    assert!(temp_dir.path().join("test_new.txt").exists());
    assert!(!temp_dir.path().join("test_old.txt").exists());
    
    // Binary files should remain unchanged
    assert!(temp_dir.path().join("old_image.png").exists());
    assert!(temp_dir.path().join("old_binary.bin").exists());
    assert!(!temp_dir.path().join("new_image.png").exists());
    assert!(!temp_dir.path().join("new_binary.bin").exists());

    // Verify text content was changed
    let text_content = fs::read_to_string(temp_dir.path().join("test_new.txt"))?;
    assert!(text_content.contains("new content in text file"));
    assert!(!text_content.contains("old content in text file"));

    Ok(())
}

#[test]
fn test_binary_names_flag_enables_renaming() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create binary files with pattern in names
    File::create(temp_dir.path().join("old_document.pdf"))?
        .write_all(&[0x25, 0x50, 0x44, 0x46, b'o', b'l', b'd'])?; // PDF header + "old"
        
    File::create(temp_dir.path().join("old_executable"))?
        .write_all(&[0x7f, 0x45, 0x4c, 0x46, b'o', b'l', b'd'])?; // ELF header + "old"

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: true, // Enable binary renaming
    };

    run_refac(args)?;

    // Binary files should be renamed
    assert!(temp_dir.path().join("new_document.pdf").exists());
    assert!(temp_dir.path().join("new_executable").exists());
    assert!(!temp_dir.path().join("old_document.pdf").exists());
    assert!(!temp_dir.path().join("old_executable").exists());

    // Binary content should be unchanged
    let pdf_content = fs::read(temp_dir.path().join("new_document.pdf"))?;
    assert_eq!(pdf_content, vec![0x25, 0x50, 0x44, 0x46, b'o', b'l', b'd']);
    
    let elf_content = fs::read(temp_dir.path().join("new_executable"))?;
    assert_eq!(elf_content, vec![0x7f, 0x45, 0x4c, 0x46, b'o', b'l', b'd']);

    Ok(())
}

#[test]
fn test_mixed_binary_text_with_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create mixed files
    File::create(temp_dir.path().join("project_old.txt"))?
        .write_all(b"old project configuration")?;
        
    File::create(temp_dir.path().join("project_old.json"))?
        .write_all(br#"{"name": "old_project", "version": "1.0"}"#)?;
        
    File::create(temp_dir.path().join("project_old.zip"))?
        .write_all(&[0x50, 0x4b, 0x03, 0x04, b'o', b'l', b'd'])?; // ZIP header + "old"

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: true,
    };

    run_refac(args)?;

    // All files should be renamed
    assert!(temp_dir.path().join("project_new.txt").exists());
    assert!(temp_dir.path().join("project_new.json").exists());
    assert!(temp_dir.path().join("project_new.zip").exists());

    // Text files should have content modified
    let txt_content = fs::read_to_string(temp_dir.path().join("project_new.txt"))?;
    assert!(txt_content.contains("new project configuration"));
    
    let json_content = fs::read_to_string(temp_dir.path().join("project_new.json"))?;
    assert!(json_content.contains("new_project"));

    // Binary file should have unchanged content
    let zip_content = fs::read(temp_dir.path().join("project_new.zip"))?;
    assert_eq!(zip_content, vec![0x50, 0x4b, 0x03, 0x04, b'o', b'l', b'd']); // Still contains "old" bytes

    Ok(())
}

#[test]
fn test_binary_flag_with_names_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files
    File::create(temp_dir.path().join("test_old.txt"))?
        .write_all(b"old content that should not change")?;
        
    File::create(temp_dir.path().join("old_binary.bin"))?
        .write_all(&[0x00, b'o', b'l', b'd', 0xff])?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: true, // Only rename, don't change content
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
        binary_names: true,
    };

    run_refac(args)?;

    // Both files should be renamed
    assert!(temp_dir.path().join("test_new.txt").exists());
    assert!(temp_dir.path().join("new_binary.bin").exists());

    // Text content should be unchanged (names_only mode)
    let txt_content = fs::read_to_string(temp_dir.path().join("test_new.txt"))?;
    assert!(txt_content.contains("old content that should not change"));
    assert!(!txt_content.contains("new content"));

    // Binary content should be unchanged
    let bin_content = fs::read(temp_dir.path().join("new_binary.bin"))?;
    assert_eq!(bin_content, vec![0x00, b'o', b'l', b'd', 0xff]);

    Ok(())
}

#[test]
fn test_binary_flag_with_content_only_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files - note: binary files won't have content processed anyway
    File::create(temp_dir.path().join("document_old.txt"))?
        .write_all(b"This contains old text")?;
        
    File::create(temp_dir.path().join("image_old.png"))?
        .write_all(&[0x89, 0x50, 0x4e, 0x47, b'o', b'l', b'd'])?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: true, // Only change content, don't rename
        max_depth: 0,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: true, // This should have no effect in content_only mode
    };

    run_refac(args)?;

    // No files should be renamed (content_only mode)
    assert!(temp_dir.path().join("document_old.txt").exists());
    assert!(temp_dir.path().join("image_old.png").exists());
    assert!(!temp_dir.path().join("document_new.txt").exists());
    assert!(!temp_dir.path().join("image_new.png").exists());

    // Text content should be modified
    let txt_content = fs::read_to_string(temp_dir.path().join("document_old.txt"))?;
    assert!(txt_content.contains("This contains new text"));
    assert!(!txt_content.contains("This contains old text"));

    // Binary content should be unchanged
    let png_content = fs::read(temp_dir.path().join("image_old.png"))?;
    assert_eq!(png_content, vec![0x89, 0x50, 0x4e, 0x47, b'o', b'l', b'd']);

    Ok(())
}

#[test]
fn test_binary_flag_edge_cases() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files that might be tricky to detect as binary
    File::create(temp_dir.path().join("empty_old.bin"))?
        .write_all(&[])?; // Empty file
        
    File::create(temp_dir.path().join("mostly_text_old.data"))?
        .write_all(b"This looks like text but has binary \x00\x01\x02 bytes")?;
        
    File::create(temp_dir.path().join("old_script.sh"))?
        .write_all(b"#!/bin/bash\necho old script")?; // Text file with executable extension

    // Test default behavior - only text files should be processed
    let args_default = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args_default)?;

    // Only the shell script (text file) should be renamed and modified
    assert!(temp_dir.path().join("new_script.sh").exists());
    assert!(!temp_dir.path().join("old_script.sh").exists());
    
    // Binary files should remain unchanged
    assert!(temp_dir.path().join("empty_old.bin").exists());
    assert!(temp_dir.path().join("mostly_text_old.data").exists());
    assert!(!temp_dir.path().join("empty_new.bin").exists());
    assert!(!temp_dir.path().join("mostly_text_new.data").exists());

    // Verify script content was modified
    let script_content = fs::read_to_string(temp_dir.path().join("new_script.sh"))?;
    assert!(script_content.contains("echo new script"));
    assert!(!script_content.contains("echo old script"));
    
    // Now test with binary_names flag - all should be renamed
    // First reset the files
    fs::remove_file(temp_dir.path().join("new_script.sh"))?;
    File::create(temp_dir.path().join("old_script.sh"))?
        .write_all(b"#!/bin/bash\necho old script")?;

    let args_with_flag = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: true,
    };

    run_refac(args_with_flag)?;

    // All files should now be renamed
    assert!(temp_dir.path().join("empty_new.bin").exists());
    assert!(temp_dir.path().join("mostly_text_new.data").exists());
    assert!(temp_dir.path().join("new_script.sh").exists());
    
    // Binary content should be unchanged
    let empty_content = fs::read(temp_dir.path().join("empty_new.bin"))?;
    assert_eq!(empty_content, Vec::<u8>::new());
    
    let data_content = fs::read(temp_dir.path().join("mostly_text_new.data"))?;
    assert_eq!(data_content, b"This looks like text but has binary \x00\x01\x02 bytes");

    Ok(())
}

#[test]
fn test_max_depth() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create nested structure
    fs::create_dir_all(temp_dir.path().join("level1/level2/level3"))?;
    File::create(temp_dir.path().join("level1/oldname1.txt"))?
        .write_all(b"content")?;
    File::create(temp_dir.path().join("level1/level2/oldname2.txt"))?
        .write_all(b"content")?;
    File::create(temp_dir.path().join("level1/level2/level3/oldname3.txt"))?
        .write_all(b"content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 4,
        exclude_patterns: vec![],
        include_patterns: vec![],
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args)?;

    // Files at depth 1, 2, and 3 should be renamed with max_depth 3
    assert!(temp_dir.path().join("level1/newname1.txt").exists());
    assert!(temp_dir.path().join("level1/level2/newname2.txt").exists());
    assert!(temp_dir.path().join("level1/level2/level3/newname3.txt").exists());

    Ok(())
}

#[test]
fn test_backup_functionality() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file
    let original_content = "oldname content here";
    File::create(temp_dir.path().join("oldname_file.txt"))?
        .write_all(original_content.as_bytes())?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: true,
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
    };

    run_refac(args)?;

    // Check that backup was created and contains original content
    let backup_files: Vec<_> = fs::read_dir(temp_dir.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.contains(".bak"))
                .unwrap_or(false)
        })
        .collect();

    assert!(!backup_files.is_empty(), "No backup file was created");

    // Check that one of the backup files contains the original content
    let mut found_original_content = false;
    for backup_file in backup_files {
        if let Ok(backup_content) = fs::read_to_string(backup_file.path()) {
            if backup_content.contains("oldname") {
                found_original_content = true;
                break;
            }
        }
    }
    assert!(found_original_content, "Backup file doesn't contain original content");

    // Check that main file was updated
    let updated_content = fs::read_to_string(temp_dir.path().join("newname_file.txt"))?;
    assert!(updated_content.contains("newname"));
    assert!(!updated_content.contains("oldname"));

    Ok(())
}

#[test]
fn test_multiple_occurrences() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create file with multiple occurrences in name and content
    File::create(temp_dir.path().join("oldname_oldname_oldname.txt"))?
        .write_all(b"oldname oldname oldname in content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args)?;

    // All occurrences should be replaced
    assert!(temp_dir.path().join("newname_newname_newname.txt").exists());
    
    let content = fs::read_to_string(temp_dir.path().join("newname_newname_newname.txt"))?;
    assert!(content.contains("newname newname newname"));
    assert!(!content.contains("oldname"));

    Ok(())
}

#[test]
fn test_hidden_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create hidden files and directories
    File::create(temp_dir.path().join(".oldname_hidden"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join(".oldname_hidden_dir"))?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        include_patterns: vec![".*".to_string()], // Include hidden files
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args)?;

    // Hidden files should be renamed
    assert!(temp_dir.path().join(".newname_hidden").exists());
    assert!(temp_dir.path().join(".newname_hidden_dir").exists());
    
    let content = fs::read_to_string(temp_dir.path().join(".newname_hidden"))?;
    assert!(content.contains("newname"));

    Ok(())
}

#[test]
fn test_exclude_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create files, some of which should be excluded
    File::create(temp_dir.path().join("oldname_include.txt"))?
        .write_all(b"oldname content")?;
    File::create(temp_dir.path().join("oldname_exclude.log"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec!["*.log".to_string()],
        include_patterns: vec![],
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args)?;

    // .txt file should be renamed, .log file should not
    assert!(temp_dir.path().join("newname_include.txt").exists());
    assert!(temp_dir.path().join("oldname_exclude.log").exists()); // Should not be renamed

    Ok(())
}

#[test]
fn test_parallel_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create many files to test parallel processing
    for i in 0..100 {
        File::create(temp_dir.path().join(format!("oldname_file_{}.txt", i)))?
            .write_all(format!("oldname content {}", i).as_bytes())?;
    }

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 4, // Use multiple threads
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };

    run_refac(args)?;

    // All files should be processed correctly
    for i in 0..100 {
        let new_file = temp_dir.path().join(format!("newname_file_{}.txt", i));
        assert!(new_file.exists(), "File {} was not renamed", i);
        
        let content = fs::read_to_string(&new_file)?;
        assert!(content.contains("newname"), "Content in file {} was not updated", i);
        assert!(!content.contains("oldname"), "Old content still exists in file {}", i);
    }

    Ok(())
}

#[test]
fn test_include_hidden_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create hidden files and directories
    File::create(temp_dir.path().join(".hidden_oldname.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join(".hidden_oldname_dir"))?;
    File::create(temp_dir.path().join(".hidden_oldname_dir/file.txt"))?
        .write_all(b"oldname content")?;
    
    // Create regular files for comparison
    File::create(temp_dir.path().join("regular_oldname.txt"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: true, // Enable hidden file processing
        binary_names: false,
    };

    run_refac(args)?;

    // Hidden files should be renamed
    assert!(temp_dir.path().join(".hidden_newname.txt").exists());
    assert!(temp_dir.path().join(".hidden_newname_dir").exists());
    assert!(temp_dir.path().join(".hidden_newname_dir/file.txt").exists());
    
    // Regular files should also be renamed
    assert!(temp_dir.path().join("regular_newname.txt").exists());
    
    // Check content was updated in hidden files
    let hidden_content = fs::read_to_string(temp_dir.path().join(".hidden_newname.txt"))?;
    assert!(hidden_content.contains("newname"));
    assert!(!hidden_content.contains("oldname"));
    
    let nested_content = fs::read_to_string(temp_dir.path().join(".hidden_newname_dir/file.txt"))?;
    assert!(nested_content.contains("newname"));
    assert!(!nested_content.contains("oldname"));

    Ok(())
}

#[test]
fn test_include_hidden_flag_disabled() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create hidden files and directories
    File::create(temp_dir.path().join(".hidden_oldname.txt"))?
        .write_all(b"oldname content")?;
    fs::create_dir(temp_dir.path().join(".hidden_oldname_dir"))?;
    
    // Create regular files for comparison
    File::create(temp_dir.path().join("regular_oldname.txt"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false, // Disable hidden file processing
        binary_names: false,
    };

    run_refac(args)?;

    // Hidden files should NOT be renamed
    assert!(temp_dir.path().join(".hidden_oldname.txt").exists());
    assert!(temp_dir.path().join(".hidden_oldname_dir").exists());
    assert!(!temp_dir.path().join(".hidden_newname.txt").exists());
    assert!(!temp_dir.path().join(".hidden_newname_dir").exists());
    
    // Regular files should be renamed
    assert!(temp_dir.path().join("regular_newname.txt").exists());
    assert!(!temp_dir.path().join("regular_oldname.txt").exists());

    Ok(())
}

#[test]
fn test_include_hidden_with_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create hidden files with different patterns
    File::create(temp_dir.path().join(".hidden_oldname.txt"))?
        .write_all(b"oldname content")?;
    File::create(temp_dir.path().join(".other_oldname.log"))?
        .write_all(b"oldname content")?;

    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "oldname".to_string(),
        substitute: "newname".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: false,
        max_depth: 0,
        exclude_patterns: vec!["*.log".to_string()], // Exclude .log files
        include_patterns: vec![],
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: true, // Enable hidden file processing
        binary_names: false,
    };

    run_refac(args)?;

    // Hidden .txt file should be renamed
    assert!(temp_dir.path().join(".hidden_newname.txt").exists());
    assert!(!temp_dir.path().join(".hidden_oldname.txt").exists());
    
    // Hidden .log file should NOT be renamed due to exclude pattern
    assert!(temp_dir.path().join(".other_oldname.log").exists());
    assert!(!temp_dir.path().join(".other_newname.log").exists());

    Ok(())
}

#[test]
fn test_content_only_allows_path_separators() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file with content
    let test_file = temp_dir.path().join("test.txt");
    File::create(&test_file)?
        .write_all(b"old content here\nmore old content")?;
    
    // Test that forward slash is allowed in content-only mode
    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new/path".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: false,
        content_only: true,
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
    };
    
    run_refac(args)?;
    
    // Check that content was replaced correctly
    let content = fs::read_to_string(&test_file)?;
    assert!(content.contains("new/path content here"));
    assert!(content.contains("more new/path content"));
    assert!(!content.contains("old content"));
    
    // Verify file name wasn't changed
    assert!(test_file.exists());
    
    Ok(())
}

#[test]
fn test_typescript_file_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create TypeScript file
    let ts_file = temp_dir.path().join("app.ts");
    File::create(&ts_file)?
        .write_all(b"const message: string = 'Hello World';\nexport function oldFunction() { return message; }")?;
    
    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "old".to_string(),
        substitute: "new".to_string(),
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
        format: workspace::cli::OutputFormat::Plain,
        threads: 1,
        progress: workspace::cli::ProgressMode::Never,
        ignore_case: false,
        use_regex: false,
        include_hidden: false,
        binary_names: false,
    };
    
    run_refac(args)?;
    
    // Check that TypeScript content was processed
    let content = fs::read_to_string(&ts_file)?;
    assert!(content.contains("newFunction"));
    assert!(!content.contains("oldFunction"));
    assert!(content.contains("const message: string = 'Hello World';"));
    
    Ok(())
}

#[test]
fn test_mutual_exclusion_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test file 
    let test_file = temp_dir.path().join("test.txt");
    File::create(&test_file)?
        .write_all(b"content")?;
    
    // Test that content-only and names-only are mutually exclusive
    let args = Args {
        root_dir: temp_dir.path().to_path_buf(),
        pattern: "test".to_string(),
        substitute: "new".to_string(),
        assume_yes: true,
        verbose: false,
        follow_symlinks: false,
        backup: false,
        files_only: false,
        dirs_only: false,
        names_only: true,    // Both flags set
        content_only: true,  // Both flags set
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
    };
    
    // Should fail during validation
    let result = run_refac(args);
    assert!(result.is_err());
    
    Ok(())
}
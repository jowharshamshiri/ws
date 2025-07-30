use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::{DirEntry, WalkDir};

use crate::{
    ItemType, RenameConfig, RenameItem, RenameStats, utils,
};
use super::{
    cli::{Args, Mode, OutputFormat},
    collision_detector::{CollisionDetector, CollisionType},
    file_ops::FileOperations,
    progress::{ProgressTracker, SimpleOutput},
};

/// Detailed information about changes to a specific file/directory
#[derive(Debug, Clone)]
pub struct FileChangeReport {
    pub path: PathBuf,
    pub content_changes: Option<usize>, // Number of content occurrences to replace
    pub rename_target: Option<PathBuf>,  // New path if being renamed
    pub item_type: ItemType,
}

/// Organized summary of all changes, grouped and sorted by location
#[derive(Debug, Clone)]
pub struct DetailedChangeReport {
    pub file_changes: Vec<FileChangeReport>,
    pub total_stats: RenameStats,
}

/// Structured validation error with location and context information
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub location: PathBuf,
    pub error_type: ValidationErrorType,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationErrorType {
    FileNotFound,
    PermissionDenied,
    EncodingError,
    NotAFile,
    NotADirectory,
    ReadOnlyFile,
    TargetExists,
    ParentDirectoryError,
    ContentNotFound,
    EmptyDirectoryIssue,
}

/// Main engine for executing rename operations
pub struct RenameEngine {
    config: RenameConfig,
    mode: Mode,
    file_ops: FileOperations,
    progress: Option<ProgressTracker>,
    simple_output: Option<SimpleOutput>,
    thread_count: usize,
    output_format: OutputFormat,
    max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    ignore_case: bool,
    use_regex: bool,
}

impl RenameEngine {
    pub fn new(args: Args) -> Result<Self> {
        // Validate arguments
        args.validate().map_err(|e| anyhow::anyhow!(e))?;

        // Create configuration
        let config = RenameConfig::new(&args.root_dir, args.old_string.clone(), args.new_string.clone())?
            .with_assume_yes(args.assume_yes)
            .with_verbose(args.verbose)
            .with_follow_symlinks(args.follow_symlinks)
            .with_backup(args.backup);

        // Setup progress tracking
        let show_progress = match args.progress {
            super::cli::ProgressMode::Always => true,
            super::cli::ProgressMode::Never => false,
            super::cli::ProgressMode::Auto => atty::is(atty::Stream::Stdout),
        };

        let (progress, simple_output) = if show_progress && args.format == OutputFormat::Human {
            (Some(ProgressTracker::new(true, args.verbose)), None)
        } else {
            (None, Some(SimpleOutput::new(args.verbose)))
        };

        Ok(Self {
            config,
            mode: args.get_mode(),
            file_ops: FileOperations::new().with_backup(args.backup),
            progress,
            simple_output,
            thread_count: args.get_thread_count(),
            output_format: args.format,
            max_depth: if args.max_depth > 0 { Some(args.max_depth) } else { None },
            include_patterns: args.include_patterns,
            exclude_patterns: args.exclude_patterns,
            ignore_case: args.ignore_case,
            use_regex: args.use_regex,
        })
    }

    /// Execute the rename operation
    pub fn execute(&self) -> Result<()> {
        self.print_header()?;

        // Phase 1: Discovery
        self.print_info("Phase 1: Discovering files and directories...")?;
        let (content_files, rename_items) = self.discover_items()?;

        // Phase 2: Collision Detection
        self.print_info("Phase 2: Checking for naming collisions...")?;
        self.check_collisions(&rename_items)?;

        // Phase 3: Mandatory Validation (Dry-Run)
        self.print_info("Phase 3: Validating all operations...")?;
        self.validate_all_operations(&content_files, &rename_items)?;

        // Phase 4: Summary and Confirmation
        let stats = self.show_summary(&content_files, &rename_items)?;
        if stats.total_changes() == 0 {
            self.print_success("No changes needed.")?;
            return Ok(());
        }

        if !self.confirm_changes()? {
            self.print_info("Operation cancelled by user.")?;
            return Ok(());
        }

        // Phase 5: Execute Changes
        self.execute_changes(&content_files, &rename_items)?;

        // Phase 5: Final Report
        self.show_final_report(&stats)?;

        Ok(())
    }

    /// Discover files for content replacement and items for renaming
    fn discover_items(&self) -> Result<(Vec<PathBuf>, Vec<RenameItem>)> {
        let mut content_files = Vec::new();
        let mut rename_items = Vec::new();

        // Setup progress
        if let Some(progress) = &self.progress {
            progress.init_main_progress(0, "Scanning files and directories...");
        }

        // Walk the directory tree
        let walker = WalkDir::new(&self.config.root_dir)
            .follow_links(self.config.follow_symlinks)
            .max_depth(self.max_depth.unwrap_or(usize::MAX))
            .into_iter()
            .filter_entry(|e| self.should_process_entry(e));

        for entry in walker {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();

            // Skip the root directory itself
            if path == self.config.root_dir {
                continue;
            }

            // Apply include/exclude patterns
            if !self.matches_patterns(path)? {
                continue;
            }

            // Check for content replacement in files
            if self.should_process_content() && 
               self.should_process_files() && 
               path.is_file() {
                if self.file_needs_content_replacement(path)? {
                    content_files.push(path.to_path_buf());
                }
            }

            // Check for renaming
            if self.should_process_names() {
                if let Some(rename_item) = self.create_rename_item(path)? {
                    rename_items.push(rename_item);
                }
            }

            // Update progress
            if let Some(progress) = &self.progress {
                progress.update_main(&format!("Scanned: {}", path.display()));
            }
        }

        // Sort rename items to prevent race conditions:
        // 1. Files first (deepest first), then directories (deepest first)
        // 2. This ensures files are renamed before their containing directories
        rename_items.sort_by(|a, b| {
            match (&a.item_type, &b.item_type) {
                // Files come before directories to prevent path invalidation
                (ItemType::File, ItemType::Directory) => std::cmp::Ordering::Less,
                (ItemType::Directory, ItemType::File) => std::cmp::Ordering::Greater,
                // Among files: process deepest first (children before parents)
                (ItemType::File, ItemType::File) => b.depth.cmp(&a.depth),
                // Among directories: process deepest first (children before parents)
                (ItemType::Directory, ItemType::Directory) => b.depth.cmp(&a.depth),
            }
        });

        if let Some(progress) = &self.progress {
            progress.finish_main("Discovery complete");
        }

        Ok((content_files, rename_items))
    }

    /// Check if an entry should be processed
    fn should_process_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        
        // Don't skip the root directory itself, even if it's hidden
        if path == self.config.root_dir {
            return true;
        }
        
        // Skip hidden files unless explicitly included
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with('.') {
                    let should_include = self.include_patterns.iter().any(|p| p == ".*" || p.contains("*"));
                    if !should_include {
                        return false;
                    }
                }
            }
        }

        // Check file type restrictions
        match self.mode {
            Mode::FilesOnly => path.is_file(),
            Mode::DirsOnly => path.is_dir(),
            _ => true,
        }
    }

    /// Check if a path matches include/exclude patterns
    fn matches_patterns(&self, path: &Path) -> Result<bool> {
        // If there are include patterns, the file must match at least one
        if !self.include_patterns.is_empty() {
            let matches = self.include_patterns.iter().any(|pattern| {
                self.path_matches_pattern(path, pattern)
            });
            if !matches {
                return Ok(false);
            }
        }

        // If there are exclude patterns, the file must not match any
        if !self.exclude_patterns.is_empty() {
            let excluded = self.exclude_patterns.iter().any(|pattern| {
                self.path_matches_pattern(path, pattern)
            });
            if excluded {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a path matches a glob pattern
    fn path_matches_pattern(&self, path: &Path, pattern: &str) -> bool {
        // Simple glob matching - could be enhanced with a proper glob library
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let pattern_str = if self.ignore_case {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };
        let compare_str = if self.ignore_case {
            file_name.to_lowercase()
        } else {
            file_name.to_string()
        };


        if self.use_regex {
            // Use regex matching
            if let Ok(regex) = regex::Regex::new(&pattern_str) {
                return regex.is_match(&compare_str);
            }
        }

        // Special case for ".*" pattern to match hidden files
        if pattern_str == ".*" {
            return compare_str.starts_with('.');
        }

        // Simple glob-style matching
        if pattern_str.contains('*') {
            let parts: Vec<&str> = pattern_str.split('*').collect();
            if parts.len() == 2 {
                return compare_str.starts_with(parts[0]) && compare_str.ends_with(parts[1]);
            }
        }

        compare_str.contains(&pattern_str)
    }

    /// Check if a file needs content replacement
    fn file_needs_content_replacement(&self, path: &Path) -> Result<bool> {
        if !self.file_ops.is_text_file(path)? {
            return Ok(false);
        }

        let search_string = if self.ignore_case {
            // For case-insensitive search, we'd need to read the file content
            // This is simplified - a full implementation would use regex
            &self.config.old_string.to_lowercase()
        } else {
            &self.config.old_string
        };

        self.file_ops.file_contains_string(path, search_string)
    }

    /// Create a rename item if the path needs renaming
    fn create_rename_item(&self, path: &Path) -> Result<Option<RenameItem>> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name: {}", path.display()))?;

        let contains_pattern = if self.ignore_case {
            file_name.to_lowercase().contains(&self.config.old_string.to_lowercase())
        } else {
            file_name.contains(&self.config.old_string)
        };


        if !contains_pattern {
            return Ok(None);
        }

        // Apply type restrictions
        let item_type = if path.is_file() {
            if !self.should_process_files() {
                return Ok(None);
            }
            ItemType::File
        } else {
            if !self.should_process_dirs() {
                return Ok(None);
            }
            ItemType::Directory
        };

        // Calculate new name
        let new_name = if self.ignore_case {
            // Case-insensitive replacement
            file_name.to_lowercase().replace(
                &self.config.old_string.to_lowercase(),
                &self.config.new_string
            )
        } else {
            utils::replace_all(file_name, &self.config.old_string, &self.config.new_string)
        };

        let new_path = path.with_file_name(new_name);
        let depth = utils::calculate_depth(path, &self.config.root_dir);

        Ok(Some(RenameItem {
            original_path: path.to_path_buf(),
            new_path,
            item_type,
            depth,
        }))
    }

    /// Check for collisions in the rename operations
    fn check_collisions(&self, rename_items: &[RenameItem]) -> Result<()> {
        if rename_items.is_empty() {
            return Ok(());
        }

        let mut detector = CollisionDetector::new();
        
        // Scan existing paths
        detector.scan_existing_paths(&self.config.root_dir)?;
        
        // Add rename operations
        detector.add_renames(rename_items);
        
        // Detect collisions
        let collisions = detector.detect_collisions()?;
        
        if !collisions.is_empty() {
            self.print_error("Naming collisions detected!")?;
            
            for collision in &collisions {
                match collision.collision_type {
                    CollisionType::SourceEqualsTarget => {
                        // Skip no-op renames
                        continue;
                    }
                    _ => {
                        self.print_error(&collision.description)?;
                    }
                }
            }
            
            let serious_collisions: Vec<_> = collisions.iter()
                .filter(|c| c.collision_type != CollisionType::SourceEqualsTarget)
                .collect();
                
            if !serious_collisions.is_empty() {
                anyhow::bail!("Cannot proceed due to {} naming collision(s)", serious_collisions.len());
            }
        }

        Ok(())
    }

    /// Generate detailed report of all changes organized by file/directory
    fn generate_detailed_report(&self, content_files: &[PathBuf], rename_items: &[RenameItem]) -> Result<DetailedChangeReport> {
        use std::collections::HashMap;
        
        let mut file_changes_map: HashMap<PathBuf, FileChangeReport> = HashMap::new();
        let mut stats = RenameStats::default();
        
        // Process content changes
        for file_path in content_files {
            // Count occurrences of old string in this file
            let content_count = match std::fs::read_to_string(file_path) {
                Ok(content) => content.matches(&self.config.old_string).count(),
                Err(_) => 0, // Already validated during validation phase
            };
            
            file_changes_map.insert(file_path.clone(), FileChangeReport {
                path: file_path.clone(),
                content_changes: Some(content_count),
                rename_target: None,
                item_type: ItemType::File,
            });
            stats.files_with_content_changes += 1;
        }
        
        // Process rename operations
        for item in rename_items {
            let entry = file_changes_map.entry(item.original_path.clone()).or_insert_with(|| {
                FileChangeReport {
                    path: item.original_path.clone(),
                    content_changes: None,
                    rename_target: None,
                    item_type: item.item_type.clone(),
                }
            });
            
            entry.rename_target = Some(item.new_path.clone());
            entry.item_type = item.item_type.clone();
            
            match item.item_type {
                ItemType::File => stats.files_renamed += 1,
                ItemType::Directory => stats.directories_renamed += 1,
            }
        }
        
        // Convert to sorted vector (by path for consistent ordering)
        let mut file_changes: Vec<FileChangeReport> = file_changes_map.into_values().collect();
        file_changes.sort_by(|a, b| a.path.cmp(&b.path));
        
        Ok(DetailedChangeReport {
            file_changes,
            total_stats: stats,
        })
    }

    /// Show detailed summary of changes organized by file/directory
    fn show_summary(&self, content_files: &[PathBuf], rename_items: &[RenameItem]) -> Result<RenameStats> {
        let report = self.generate_detailed_report(content_files, rename_items)?;

        match self.output_format {
            OutputFormat::Json => {
                let json_report = serde_json::json!({
                    "summary": {
                        "content_changes": report.total_stats.files_with_content_changes,
                        "file_renames": report.total_stats.files_renamed,
                        "directory_renames": report.total_stats.directories_renamed,
                        "total_changes": report.total_stats.total_changes()
                    },
                    "file_changes": report.file_changes.iter().map(|fc| {
                        serde_json::json!({
                            "path": fc.path,
                            "content_changes": fc.content_changes,
                            "rename_target": fc.rename_target,
                            "item_type": format!("{:?}", fc.item_type)
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&json_report)?);
            }
            OutputFormat::Plain => {
                println!("Content changes: {}", report.total_stats.files_with_content_changes);
                println!("File renames: {}", report.total_stats.files_renamed);
                println!("Directory renames: {}", report.total_stats.directories_renamed);
                println!("Total changes: {}", report.total_stats.total_changes());
            }
            OutputFormat::Human => {
                self.print_info("=== PLANNED CHANGES ===")?;
                self.print_info(&format!("Total files/directories affected: {}", report.file_changes.len()))?;
                self.print_info(&format!("Content modifications: {} file(s)", report.total_stats.files_with_content_changes))?;
                self.print_info(&format!("File renames:         {} file(s)", report.total_stats.files_renamed))?;
                self.print_info(&format!("Directory renames:    {} directory(ies)", report.total_stats.directories_renamed))?;
                self.print_info("")?;

                if !report.file_changes.is_empty() {
                    self.print_info("=== DETAILED CHANGES BY LOCATION ===")?;
                    
                    for change in &report.file_changes {
                        let relative_path = change.path.strip_prefix(&self.config.root_dir)
                            .unwrap_or(&change.path);
                        
                        self.print_info(&format!("📁 {}", relative_path.display()))?;
                        
                        // Show content changes
                        if let Some(count) = change.content_changes {
                            self.print_verbose(&format!("   Content: {} occurrence(s) of '{}' → '{}'", 
                                count, self.config.old_string, self.config.new_string))?;
                        }
                        
                        // Show rename operation
                        if let Some(target) = &change.rename_target {
                            let relative_target = target.strip_prefix(&self.config.root_dir)
                                .unwrap_or(target);
                            self.print_verbose(&format!("   Rename:  {} → {}", 
                                relative_path.display(), relative_target.display()))?;
                        }
                        
                        self.print_info("")?; // Empty line for readability
                    }
                }
            }
        }

        Ok(report.total_stats)
    }

    /// Confirm changes with the user
    fn confirm_changes(&self) -> Result<bool> {
        if self.config.assume_yes {
            return Ok(true);
        }

        match self.output_format {
            OutputFormat::Json => Ok(true), // No confirmation in JSON mode
            OutputFormat::Plain | OutputFormat::Human => {
                self.print_warning("This operation will modify your files and directories.")?;
                
                let confirmation = if let Some(progress) = &self.progress {
                    progress.suspend(|| {
                        dialoguer::Confirm::new()
                            .with_prompt("Do you want to proceed?")
                            .default(false)
                            .interact()
                    })
                } else {
                    dialoguer::Confirm::new()
                        .with_prompt("Do you want to proceed?")
                        .default(false)
                        .interact()
                };

                confirmation.with_context(|| "Failed to get user confirmation")
            }
        }
    }

    /// Execute the actual changes
    fn execute_changes(&self, content_files: &[PathBuf], rename_items: &[RenameItem]) -> Result<()> {
        // Phase 1: Content replacement
        if !content_files.is_empty() && self.should_process_content() {
            self.execute_content_changes(content_files)?;
        }

        // Phase 2: Rename items (directories first, then files)
        if !rename_items.is_empty() && self.should_process_names() {
            self.execute_renames(rename_items)?;
        }

        Ok(())
    }

    /// Execute content changes
    fn execute_content_changes(&self, content_files: &[PathBuf]) -> Result<()> {
        self.print_info("Replacing content in files...")?;

        if let Some(progress) = &self.progress {
            progress.init_content_progress(content_files.len() as u64);
        }

        let errors = Arc::new(Mutex::new(Vec::new()));
        let _progress_ref = &self.progress;
        let config_ref = &self.config;
        let file_ops_ref = &self.file_ops;
        let errors_ref = Arc::clone(&errors);

        if self.thread_count > 1 {
            // Parallel processing with improved error handling
            content_files.par_iter().for_each(|file_path| {
                // Validate file still exists before processing
                if !file_path.exists() {
                    errors_ref.lock().unwrap().push(format!("File no longer exists: {}", file_path.display()));
                    return;
                }

                let result = file_ops_ref.replace_content(
                    file_path,
                    &config_ref.old_string,
                    &config_ref.new_string,
                );

                match result {
                    Ok(modified) => {
                        if modified && config_ref.verbose {
                            // Note: Can't use self.print_verbose in parallel context
                            // Verbose output is handled in sequential mode
                        }
                    }
                    Err(e) => {
                        errors_ref.lock().unwrap().push(format!("Failed to modify {}: {}", file_path.display(), e));
                    }
                }
            });
        } else {
            // Sequential processing with enhanced error handling
            for file_path in content_files {
                // Validate file still exists before processing
                if !file_path.exists() {
                    self.print_error(&format!("File no longer exists: {}", file_path.display()))?;
                    if let Some(progress) = &self.progress {
                        progress.update_content(&file_path.display().to_string());
                    }
                    continue;
                }

                let result = file_ops_ref.replace_content(
                    file_path,
                    &config_ref.old_string,
                    &config_ref.new_string,
                );

                match result {
                    Ok(modified) => {
                        if modified && config_ref.verbose {
                            self.print_verbose(&format!("Modified: {}", file_path.display()))?;
                        }
                    }
                    Err(e) => {
                        self.print_error(&format!("Failed to modify {}: {}", file_path.display(), e))?;
                    }
                }

                if let Some(progress) = &self.progress {
                    progress.update_content(&file_path.display().to_string());
                }
            }
        }

        // Report any errors from parallel processing
        let errors = errors.lock().unwrap();
        for error in errors.iter() {
            self.print_error(error)?;
        }

        if let Some(progress) = &self.progress {
            progress.finish_content(&format!("Content replacement complete ({} files)", content_files.len()));
        }

        Ok(())
    }

    /// Execute rename operations with proper ordering and error handling
    fn execute_renames(&self, rename_items: &[RenameItem]) -> Result<()> {
        self.print_info("Renaming files and directories...")?;

        if let Some(progress) = &self.progress {
            progress.init_rename_progress(rename_items.len() as u64);
        }

        let mut errors = Vec::new();
        let mut successful_renames = Vec::new();

        // Process renames sequentially to maintain ordering (files before directories)
        for item in rename_items {
            // Skip no-op renames
            if item.original_path == item.new_path {
                if let Some(progress) = &self.progress {
                    progress.update_rename(&item.original_path.display().to_string());
                }
                continue;
            }

            // Validate that source still exists (in case of race conditions, including broken symlinks)
            let source_exists = item.original_path.exists() || item.original_path.symlink_metadata().is_ok();
            if !source_exists {
                errors.push(format!("Source path no longer exists: {}", item.original_path.display()));
                if let Some(progress) = &self.progress {
                    progress.update_rename(&item.original_path.display().to_string());
                }
                continue;
            }

            // Ensure target directory exists
            if let Some(parent) = item.new_path.parent() {
                if let Err(e) = self.file_ops.create_dir_all(parent) {
                    errors.push(format!("Failed to create parent directory for {}: {}", 
                                      item.new_path.display(), e));
                    if let Some(progress) = &self.progress {
                        progress.update_rename(&item.original_path.display().to_string());
                    }
                    continue;
                }
            }

            let result = self.file_ops.move_item(&item.original_path, &item.new_path);

            match result {
                Ok(()) => {
                    successful_renames.push((item.original_path.clone(), item.new_path.clone()));
                    if self.config.verbose {
                        self.print_verbose(&format!("Renamed: {} → {}", 
                            item.original_path.display(), 
                            item.new_path.display()))?;
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to rename {} to {}: {}", 
                        item.original_path.display(), 
                        item.new_path.display(),
                        e));
                }
            }

            if let Some(progress) = &self.progress {
                progress.update_rename(&item.original_path.display().to_string());
            }
        }

        // Report errors
        for error in &errors {
            self.print_error(error)?;
        }

        if !errors.is_empty() {
            self.print_warning(&format!("{} rename operation(s) failed out of {}", 
                                      errors.len(), rename_items.len()))?;
        }

        if let Some(progress) = &self.progress {
            progress.finish_rename(&format!("Rename complete ({} successful, {} failed)", 
                                          successful_renames.len(), errors.len()));
        }

        Ok(())
    }

    /// Validate all operations before execution (mandatory validation phase)
    /// This catches all potential issues before making any changes
    fn validate_all_operations(&self, content_files: &[PathBuf], rename_items: &[RenameItem]) -> Result<()> {
        let mut validation_errors = Vec::new();

        // Validate content replacement operations
        for file_path in content_files {
            self.validate_content_file(file_path, &mut validation_errors);
        }

        // Validate rename operations
        for item in rename_items {
            self.validate_rename_item(item, &mut validation_errors);
        }

        // Validate that operation will not leave empty directories
        self.validate_no_empty_directories_remain(rename_items, &mut validation_errors)?;

        // Report all validation errors with enhanced formatting
        if !validation_errors.is_empty() {
            self.report_validation_errors(&validation_errors)?;
            
            // Create a more descriptive error message that includes error types
            let error_types: std::collections::HashSet<&str> = validation_errors.iter().map(|e| {
                match e.error_type {
                    ValidationErrorType::FileNotFound => "missing files",
                    ValidationErrorType::PermissionDenied => "permission issues", 
                    ValidationErrorType::EncodingError => "encoding issues",
                    ValidationErrorType::ReadOnlyFile => "read-only files",
                    ValidationErrorType::TargetExists => "target conflicts",
                    ValidationErrorType::ParentDirectoryError => "directory issues",
                    ValidationErrorType::ContentNotFound => "content issues",
                    ValidationErrorType::EmptyDirectoryIssue => "directory structure issues",
                    _ => "other issues",
                }
            }).collect();
            
            let error_summary = error_types.into_iter().collect::<Vec<_>>().join(", ");
            anyhow::bail!("Operation cannot proceed due to {} validation error(s): {}", validation_errors.len(), error_summary);
        }

        self.print_info("Validation passed: All operations can be performed safely.")?;
        Ok(())
    }

    /// Validate a single content file for replacement operations
    fn validate_content_file(&self, file_path: &PathBuf, validation_errors: &mut Vec<ValidationError>) {
        let relative_path = file_path.strip_prefix(&self.config.root_dir)
            .unwrap_or(file_path);

        if !file_path.exists() {
            validation_errors.push(ValidationError {
                location: file_path.clone(),
                error_type: ValidationErrorType::FileNotFound,
                message: format!("Content file does not exist: {}", relative_path.display()),
                suggestion: Some("Check if the file was moved or deleted".to_string()),
            });
            return;
        }

        if !file_path.is_file() {
            validation_errors.push(ValidationError {
                location: file_path.clone(),
                error_type: ValidationErrorType::NotAFile,
                message: format!("Content target is not a file: {}", relative_path.display()),
                suggestion: Some("Content replacement only works on files, not directories".to_string()),
            });
            return;
        }

        // Check if file is readable and writable
        if let Err(e) = std::fs::OpenOptions::new().read(true).write(true).open(file_path) {
            let error_type = if e.kind() == std::io::ErrorKind::PermissionDenied {
                ValidationErrorType::PermissionDenied
            } else {
                ValidationErrorType::PermissionDenied
            };
            
            validation_errors.push(ValidationError {
                location: file_path.clone(),
                error_type,
                message: format!("Cannot access file {}: {}", relative_path.display(), e),
                suggestion: Some("Check file permissions or if file is locked by another process".to_string()),
            });
            return;
        }

        // Validate that file can be read and contains the target string
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                if !content.contains(&self.config.old_string) {
                    validation_errors.push(ValidationError {
                        location: file_path.clone(),
                        error_type: ValidationErrorType::ContentNotFound,
                        message: format!("File {} does not contain target string '{}'", 
                                       relative_path.display(), self.config.old_string),
                        suggestion: Some("File may have been modified since discovery phase".to_string()),
                    });
                }
            },
            Err(e) => {
                validation_errors.push(ValidationError {
                    location: file_path.clone(),
                    error_type: ValidationErrorType::EncodingError,
                    message: format!("Cannot read file {} as text: {}. This may be due to encoding issues or the file may be binary.", 
                                   relative_path.display(), e),
                    suggestion: Some("Use --files-only mode or exclude this file with --exclude patterns".to_string()),
                });
            }
        }
    }

    /// Validate a single rename operation
    fn validate_rename_item(&self, item: &RenameItem, validation_errors: &mut Vec<ValidationError>) {
        let relative_source = item.original_path.strip_prefix(&self.config.root_dir)
            .unwrap_or(&item.original_path);
        let relative_target = item.new_path.strip_prefix(&self.config.root_dir)
            .unwrap_or(&item.new_path);

        // Skip no-op renames
        if item.original_path == item.new_path {
            return;
        }

        // Check source exists (including broken symlinks)
        let source_exists = item.original_path.exists() || item.original_path.symlink_metadata().is_ok();
        if !source_exists {
            validation_errors.push(ValidationError {
                location: item.original_path.clone(),
                error_type: ValidationErrorType::FileNotFound,
                message: format!("Rename source does not exist: {}", relative_source.display()),
                suggestion: Some("Source may have been moved or deleted since discovery".to_string()),
            });
            return;
        }

        // Check target doesn't already exist (unless it's the same as source)
        if item.new_path.exists() && item.new_path != item.original_path {
            validation_errors.push(ValidationError {
                location: item.new_path.clone(),
                error_type: ValidationErrorType::TargetExists,
                message: format!("Rename target already exists: {}", relative_target.display()),
                suggestion: Some("Target was created since discovery phase, or there's a naming collision".to_string()),
            });
            return;
        }

        // Check parent directory exists or can be created
        if let Some(parent) = item.new_path.parent() {
            if !parent.exists() {
                match std::fs::create_dir_all(parent) {
                    Ok(_) => {
                        // Remove the directory we just created for validation
                        let _ = std::fs::remove_dir_all(parent);
                    },
                    Err(e) => {
                        validation_errors.push(ValidationError {
                            location: parent.to_path_buf(),
                            error_type: ValidationErrorType::ParentDirectoryError,
                            message: format!("Cannot create parent directory for {}: {}", 
                                           relative_target.display(), e),
                            suggestion: Some("Check permissions on parent directories".to_string()),
                        });
                        return;
                    }
                }
            }
        }

        // Check permissions on source (use symlink_metadata to handle broken symlinks)
        match item.original_path.symlink_metadata() {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    validation_errors.push(ValidationError {
                        location: item.original_path.clone(),
                        error_type: ValidationErrorType::ReadOnlyFile,
                        message: format!("Source is read-only: {}", relative_source.display()),
                        suggestion: Some("Change file permissions or exclude read-only files".to_string()),
                    });
                }
            },
            Err(e) => {
                validation_errors.push(ValidationError {
                    location: item.original_path.clone(),
                    error_type: ValidationErrorType::PermissionDenied,
                    message: format!("Cannot read metadata for {}: {}", relative_source.display(), e),
                    suggestion: Some("Check file permissions and access rights".to_string()),
                });
            }
        }
    }

    /// Report validation errors with enhanced formatting and organization
    fn report_validation_errors(&self, validation_errors: &[ValidationError]) -> Result<()> {
        self.print_error("=== VALIDATION FAILED ===")?;
        self.print_error(&format!("Found {} issue(s) that prevent safe execution:", validation_errors.len()))?;
        self.print_error("")?;

        // Group errors by type for better organization
        use std::collections::HashMap;
        let mut errors_by_type: HashMap<String, Vec<&ValidationError>> = HashMap::new();
        
        for error in validation_errors {
            let type_name = match error.error_type {
                ValidationErrorType::FileNotFound => "Missing Files/Directories",
                ValidationErrorType::PermissionDenied => "Permission Issues", 
                ValidationErrorType::EncodingError => "Encoding/Binary File Issues",
                ValidationErrorType::NotAFile => "File Type Issues",
                ValidationErrorType::ReadOnlyFile => "Read-Only Files",
                ValidationErrorType::TargetExists => "Target Conflicts",
                ValidationErrorType::ParentDirectoryError => "Directory Creation Issues",
                ValidationErrorType::ContentNotFound => "Content Issues",
                ValidationErrorType::EmptyDirectoryIssue => "Directory Structure Issues",
                _ => "Other Issues",
            };
            
            errors_by_type.entry(type_name.to_string())
                .or_insert_with(Vec::new)
                .push(error);
        }

        // Report errors grouped by type
        for (error_type, errors) in errors_by_type {
            self.print_error(&format!("📋 {}: ({} issue(s))", error_type, errors.len()))?;
            
            for error in errors {
                let relative_path = error.location.strip_prefix(&self.config.root_dir)
                    .unwrap_or(&error.location);
                
                self.print_error(&format!("   📍 Location: {}", relative_path.display()))?;
                self.print_error(&format!("      Problem: {}", error.message))?;
                
                if let Some(suggestion) = &error.suggestion {
                    self.print_error(&format!("      Suggestion: {}", suggestion))?;
                }
                self.print_error("")?;
            }
        }

        self.print_error("❌ Cannot proceed until these issues are resolved.")?;
        Ok(())
    }

    /// Validate that the rename operations will not leave empty directories
    /// This is a critical test - if it fails, our operation ordering is wrong
    fn validate_no_empty_directories_remain(&self, rename_items: &[RenameItem], validation_errors: &mut Vec<ValidationError>) -> Result<()> {
        use std::collections::{HashMap, HashSet};

        // Build a map of all directories and what they contain
        let mut directory_contents: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
        
        // Scan the existing directory structure
        for entry in walkdir::WalkDir::new(&self.config.root_dir) {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(parent) = path.parent() {
                directory_contents.entry(parent.to_path_buf())
                    .or_insert_with(HashSet::new)
                    .insert(path.to_path_buf());
            }
        }

        // Simulate the rename operations
        for item in rename_items {
            if item.original_path == item.new_path {
                continue; // No-op
            }

            // Remove from old parent
            if let Some(old_parent) = item.original_path.parent() {
                if let Some(contents) = directory_contents.get_mut(old_parent) {
                    contents.remove(&item.original_path);
                }
            }

            // Add to new parent
            if let Some(new_parent) = item.new_path.parent() {
                directory_contents.entry(new_parent.to_path_buf())
                    .or_insert_with(HashSet::new)
                    .insert(item.new_path.clone());
            }
        }

        // Check for directories that would become empty (excluding directories being renamed themselves)
        let renamed_dirs: HashSet<PathBuf> = rename_items.iter()
            .filter(|item| item.item_type == crate::ItemType::Directory)
            .map(|item| item.original_path.clone())
            .collect();

        for (dir_path, contents) in &directory_contents {
            // Skip root directory
            if dir_path == &self.config.root_dir {
                continue;
            }

            // Skip directories that are being renamed (they should become empty)
            if renamed_dirs.contains(dir_path) {
                continue;
            }

            // Check if directory would become empty
            if contents.is_empty() {
                let relative_path = dir_path.strip_prefix(&self.config.root_dir)
                    .unwrap_or(dir_path);
                    
                validation_errors.push(ValidationError {
                    location: dir_path.clone(),
                    error_type: ValidationErrorType::EmptyDirectoryIssue,
                    message: format!("CRITICAL: Directory would become empty after operation: {}. This indicates incorrect operation ordering.", 
                                   relative_path.display()),
                    suggestion: Some("This is a logic error in operation ordering - files should be processed before directories".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Show final report
    fn show_final_report(&self, stats: &RenameStats) -> Result<()> {
        match self.output_format {
            OutputFormat::Json => {
                let report = serde_json::json!({
                    "result": "success",
                    "stats": {
                        "content_changes": stats.files_with_content_changes,
                        "file_renames": stats.files_renamed,
                        "directory_renames": stats.directories_renamed,
                        "total_changes": stats.total_changes(),
                        "errors": stats.errors.len()
                    },
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            OutputFormat::Plain => {
                println!("Operation completed successfully.");
                println!("Total changes: {}", stats.total_changes());
            }
            OutputFormat::Human => {
                self.print_success("=== OPERATION COMPLETE ===")?;
                self.print_success("Operation completed successfully!")?;
                self.print_info(&format!("Total changes applied: {}", stats.total_changes()))?;

                if !stats.errors.is_empty() {
                    self.print_warning(&format!("{} error(s) occurred:", stats.errors.len()))?;
                    for error in &stats.errors {
                        self.print_error(error)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Print header information
    fn print_header(&self) -> Result<()> {
        if self.output_format != OutputFormat::Human {
            return Ok(());
        }

        self.print_success("=== NOMION REFAC TOOL ===")?;
        self.print_info(&format!("Root directory: {}", self.config.root_dir.display()))?;
        self.print_info(&format!("Old string: '{}'", self.config.old_string))?;
        self.print_info(&format!("New string: '{}'", self.config.new_string))?;
        self.print_info(&format!("Mode: {:?}", self.mode))?;
        
        
        if self.config.backup {
            self.print_info("Backup mode: Enabled")?;
        }

        Ok(())
    }

    // Utility methods for printing
    fn print_info(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_info(message);
        } else if let Some(output) = &self.simple_output {
            output.print_info(message);
        }
        Ok(())
    }

    fn print_error(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_error(message);
        } else if let Some(output) = &self.simple_output {
            output.print_error(message);
        }
        Ok(())
    }

    fn print_warning(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_warning(message);
        } else if let Some(output) = &self.simple_output {
            output.print_warning(message);
        }
        Ok(())
    }

    fn print_success(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_success(message);
        } else if let Some(output) = &self.simple_output {
            output.print_success(message);
        }
        Ok(())
    }

    fn print_verbose(&self, message: &str) -> Result<()> {
        if let Some(progress) = &self.progress {
            progress.print_verbose(message);
        } else if let Some(output) = &self.simple_output {
            output.print_verbose(message);
        }
        Ok(())
    }

    // Mode checking methods
    fn should_process_files(&self) -> bool {
        self.mode.should_process_files()
    }

    fn should_process_dirs(&self) -> bool {
        self.mode.should_process_dirs()
    }

    fn should_process_content(&self) -> bool {
        !matches!(self.mode, Mode::NamesOnly)
    }

    fn should_process_names(&self) -> bool {
        !matches!(self.mode, Mode::ContentOnly)
    }
}

// Extension traits to add methods to the Mode and Config types
trait ModeExt {
    fn should_process_files(&self) -> bool;
    fn should_process_dirs(&self) -> bool;
}

impl ModeExt for Mode {
    fn should_process_files(&self) -> bool {
        matches!(self, Mode::Full | Mode::FilesOnly | Mode::ContentOnly | Mode::NamesOnly)
    }

    fn should_process_dirs(&self) -> bool {
        matches!(self, Mode::Full | Mode::DirsOnly | Mode::NamesOnly)
    }
}


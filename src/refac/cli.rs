use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "refac")]
#[command(version = crate::get_version(), about = "A robust cross-platform tool for recursive string replacement in file/folder names and contents - part of the workspace tool suite")]
#[command(long_about = None)]
pub struct Args {
    /// Root directory to search in
    #[arg(value_name = "ROOT_DIR")]
    pub root_dir: PathBuf,

    /// Pattern to find and replace
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// Replacement text
    #[arg(value_name = "SUBSTITUTE")]
    pub substitute: String,


    /// Assume "yes" to confirmation prompts (non-interactive mode)
    #[arg(short = 'y', long = "assume-yes")]
    pub assume_yes: bool,

    /// Show detailed output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Follow symbolic links
    #[arg(long = "follow-symlinks")]
    pub follow_symlinks: bool,

    /// Create backup files before modifying content
    #[arg(short = 'b', long = "backup")]
    pub backup: bool,

    /// Only process files (skip directories)
    #[arg(long = "files-only")]
    pub files_only: bool,

    /// Only process directories (skip files)
    #[arg(long = "dirs-only")]
    pub dirs_only: bool,

    /// Skip content replacement, only rename files/directories
    #[arg(long = "names-only")]
    pub names_only: bool,

    /// Skip file/directory renaming, only replace content
    #[arg(long = "content-only")]
    pub content_only: bool,

    /// Maximum depth to search (0 = unlimited)
    #[arg(long = "max-depth", default_value = "0")]
    pub max_depth: usize,

    /// Exclude files matching these patterns (glob patterns)
    #[arg(long = "exclude", value_name = "PATTERN")]
    pub exclude_patterns: Vec<String>,

    /// Include only files matching these patterns (glob patterns)
    #[arg(long = "include", value_name = "PATTERN")]
    pub include_patterns: Vec<String>,

    /// Output format
    #[arg(long = "format", default_value = "human")]
    pub format: OutputFormat,

    /// Number of threads to use for processing (0 = auto)
    #[arg(short = 'j', long = "threads", default_value = "0")]
    pub threads: usize,

    /// Progress display mode
    #[arg(long = "progress", default_value = "auto")]
    pub progress: ProgressMode,

    /// Ignore case when matching patterns
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Use regex patterns instead of literal strings
    #[arg(short = 'r', long = "regex")]
    pub use_regex: bool,

    /// Include hidden files and directories (starting with '.')
    #[arg(long = "include-hidden")]
    pub include_hidden: bool,

    /// Include binary file names in renaming operations (content will still be skipped)
    #[arg(long = "binary-names")]
    pub binary_names: bool,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum OutputFormat {
    /// Human-readable output with colors
    Human,
    /// JSON output for scripting
    Json,
    /// Plain text output
    Plain,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum ProgressMode {
    /// Auto-detect based on terminal capabilities
    Auto,
    /// Never show progress
    Never,
    /// Always show progress
    Always,
}

#[derive(Debug, Clone)]
pub enum Mode {
    /// Process both files and directories, both names and content
    Full,
    /// Only process files
    FilesOnly,
    /// Only process directories
    DirsOnly,
    /// Only rename files/directories (skip content)
    NamesOnly,
    /// Only replace content (skip renaming)
    ContentOnly,
}

impl Args {
    pub fn get_mode(&self) -> Mode {
        match (self.files_only, self.dirs_only, self.names_only, self.content_only) {
            (true, false, false, false) => Mode::FilesOnly,
            (false, true, false, false) => Mode::DirsOnly,
            (false, false, true, false) => Mode::NamesOnly,
            (false, false, false, true) => Mode::ContentOnly,
            _ => Mode::Full,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        // Check for conflicting options
        let mode_flags = [self.files_only, self.dirs_only, self.names_only, self.content_only];
        let active_modes = mode_flags.iter().filter(|&&x| x).count();
        
        if active_modes > 1 {
            return Err("Cannot specify more than one mode flag (--files-only, --dirs-only, --names-only, --content-only)".to_string());
        }

        // Validate root directory exists
        if !self.root_dir.exists() {
            return Err(format!("Root directory does not exist: {}", self.root_dir.display()));
        }

        if !self.root_dir.is_dir() {
            return Err(format!("Root path is not a directory: {}", self.root_dir.display()));
        }

        // Validate strings
        if self.pattern.is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }

        if self.substitute.is_empty() {
            return Err("Substitute cannot be empty".to_string());
        }

        // Check for path separators in substitute (only when processing names)
        if self.should_process_names() && (self.substitute.contains('/') || self.substitute.contains('\\')) {
            return Err("Substitute cannot contain path separators (/ or \\) when processing names".to_string());
        }

        // Validate thread count
        if self.threads > 1000 {
            return Err("Thread count cannot exceed 1000".to_string());
        }

        // Validate max depth
        if self.max_depth > 1000 {
            return Err("Max depth cannot exceed 1000".to_string());
        }

        Ok(())
    }

    pub fn should_process_files(&self) -> bool {
        !self.dirs_only
    }

    pub fn should_process_dirs(&self) -> bool {
        !self.files_only
    }

    pub fn should_process_content(&self) -> bool {
        !self.names_only
    }

    pub fn should_process_names(&self) -> bool {
        !self.content_only
    }

    pub fn get_thread_count(&self) -> usize {
        if self.threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        } else {
            self.threads
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_args_validation() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut args = Args {
            root_dir: temp_dir.path().to_path_buf(),
            pattern: "old".to_string(),
            substitute: "new".to_string(),
            assume_yes: false,
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
            format: OutputFormat::Human,
            threads: 0,
            progress: ProgressMode::Auto,
            ignore_case: false,
            use_regex: false,
            include_hidden: false,
            binary_names: false,
        };

        // Valid args should pass
        assert!(args.validate().is_ok());

        // Empty pattern should fail
        args.pattern = "".to_string();
        assert!(args.validate().is_err());
        args.pattern = "old".to_string();

        // Empty substitute should fail
        args.substitute = "".to_string();
        assert!(args.validate().is_err());
        args.substitute = "new".to_string();

        // Path separator in substitute should fail when processing names
        args.substitute = "new/path".to_string();
        assert!(args.validate().is_err());
        args.substitute = "new\\path".to_string();
        assert!(args.validate().is_err());
        args.substitute = "new".to_string();

        // Path separator should be allowed with content-only mode
        args.content_only = true;
        args.substitute = "new/path".to_string();
        assert!(args.validate().is_ok());
        args.substitute = "new\\path".to_string();
        assert!(args.validate().is_ok());
        args.substitute = "new".to_string();
        args.content_only = false;

        // Multiple mode flags should fail
        args.files_only = true;
        args.dirs_only = true;
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_mode_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        let base_args = Args {
            root_dir: temp_dir.path().to_path_buf(),
            pattern: "old".to_string(),
            substitute: "new".to_string(),
            assume_yes: false,
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
            format: OutputFormat::Human,
            threads: 0,
            progress: ProgressMode::Auto,
            ignore_case: false,
            use_regex: false,
            include_hidden: false,
            binary_names: false,
        };

        // Test default mode
        assert!(matches!(base_args.get_mode(), Mode::Full));

        // Test files only
        let mut args = base_args.clone();
        args.files_only = true;
        assert!(matches!(args.get_mode(), Mode::FilesOnly));

        // Test dirs only
        let mut args = base_args.clone();
        args.dirs_only = true;
        assert!(matches!(args.get_mode(), Mode::DirsOnly));

        // Test names only
        let mut args = base_args.clone();
        args.names_only = true;
        assert!(matches!(args.get_mode(), Mode::NamesOnly));

        // Test content only
        let mut args = base_args.clone();
        args.content_only = true;
        assert!(matches!(args.get_mode(), Mode::ContentOnly));
    }

    #[test]
    fn test_processing_flags() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut args = Args {
            root_dir: temp_dir.path().to_path_buf(),
            pattern: "old".to_string(),
            substitute: "new".to_string(),
            assume_yes: false,
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
            format: OutputFormat::Human,
            threads: 0,
            progress: ProgressMode::Auto,
            ignore_case: false,
            use_regex: false,
            include_hidden: false,
            binary_names: false,
        };

        // Default should process everything
        assert!(args.should_process_files());
        assert!(args.should_process_dirs());
        assert!(args.should_process_content());
        assert!(args.should_process_names());

        // Dirs only
        args.dirs_only = true;
        assert!(!args.should_process_files());
        assert!(args.should_process_dirs());
        args.dirs_only = false;

        // Names only
        args.content_only = true;
        assert!(args.should_process_content());
        assert!(!args.should_process_names());
    }
}
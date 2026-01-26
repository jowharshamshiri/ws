pub mod refac;
pub mod scrap;
pub mod st8;
pub mod ldiff;
pub mod logging;
pub mod workspace_state;
// Entity system
pub mod entities;
// MCP server - temporarily disabled during schema-based refactor
// pub mod mcp_server;
// MCP protocol
pub mod mcp_protocol;
// Code analysis with ast-grep
pub mod code_analysis;
// Interactive tree navigation
pub mod interactive_tree;

use anyhow::{Context, Result};
use std::path::Path;

/// Read version from version.txt file at project root
pub fn get_version() -> &'static str {
    include_str!("../version.txt").trim()
}

// Re-export from refac module for backward compatibility
pub use refac::cli as cli;
pub use refac::cli::{Args, Mode};
pub use refac::rename_engine::RenameEngine;

// Re-export from scrap module
pub use scrap::scrap_common::{ScrapMetadata, ScrapEntry};
pub use scrap::{run_scrap, run_unscrap};

// Re-export from ldiff module
pub use ldiff::run_ldiff;

// Re-export from st8 module
pub use st8::{St8Config, VersionInfo};

/// Main entry point for the refac operation within the workspace tool suite
pub fn run_refac(args: Args) -> Result<()> {
    let engine = RenameEngine::new(args)?;
    engine.execute()
}

/// Represents a file or directory that needs to be processed
#[derive(Debug, Clone)]
pub struct RenameItem {
    pub original_path: std::path::PathBuf,
    pub new_path: std::path::PathBuf,
    pub item_type: ItemType,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    File,
    Directory,
}

/// Configuration for the rename operation
#[derive(Debug, Clone)]
pub struct RenameConfig {
    pub root_dir: std::path::PathBuf,
    pub pattern: String,
    pub substitute: String,
    pub assume_yes: bool,
    pub verbose: bool,
    pub follow_symlinks: bool,
    pub backup: bool,
}

impl RenameConfig {
    pub fn new<P: AsRef<Path>>(
        root_dir: P,
        pattern: String,
        substitute: String,
    ) -> Result<Self> {
        let root_path = root_dir.as_ref().canonicalize()
            .with_context(|| format!("Failed to resolve root directory: {}", root_dir.as_ref().display()))?;
        
        if pattern.is_empty() {
            anyhow::bail!("Pattern cannot be empty");
        }
        
        if substitute.is_empty() {
            anyhow::bail!("Substitute cannot be empty");
        }
        
        
        Ok(Self {
            root_dir: root_path,
            pattern,
            substitute,
            assume_yes: false,
            verbose: false,
            follow_symlinks: false,
            backup: false,
        })
    }
    
    
    pub fn with_assume_yes(mut self, assume_yes: bool) -> Self {
        self.assume_yes = assume_yes;
        self
    }
    
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
        self.follow_symlinks = follow_symlinks;
        self
    }
    
    pub fn with_backup(mut self, backup: bool) -> Self {
        self.backup = backup;
        self
    }
}

/// Statistics about the rename operation
#[derive(Debug, Default, Clone)]
pub struct RenameStats {
    pub files_with_content_changes: usize,
    pub files_renamed: usize,
    pub directories_renamed: usize,
    pub files_processed: usize,
    pub errors: Vec<String>,
}

impl RenameStats {
    pub fn total_changes(&self) -> usize {
        self.files_with_content_changes + self.files_renamed + self.directories_renamed
    }
    
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
}

/// Utility functions
pub mod utils {
    use std::path::Path;
    
    /// Calculate the depth of a path relative to a root
    pub fn calculate_depth(path: &Path, root: &Path) -> usize {
        path.strip_prefix(root)
            .map(|p| p.components().count())
            .unwrap_or(0)
    }
    
    /// Check if a string contains the target pattern
    pub fn contains_pattern(text: &str, pattern: &str) -> bool {
        text.contains(pattern)
    }
    
    /// Replace all occurrences of old with new in the string
    pub fn replace_all(text: &str, old: &str, new: &str) -> String {
        text.replace(old, new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_rename_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RenameConfig::new(
            temp_dir.path(),
            "old".to_string(),
            "new".to_string(),
        ).unwrap();
        
        assert_eq!(config.pattern, "old");
        assert_eq!(config.substitute, "new");
        assert!(!config.assume_yes);
    }
    
    #[test]
    fn test_rename_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Empty old string should fail
        assert!(RenameConfig::new(temp_dir.path(), "".to_string(), "new".to_string()).is_err());
        
        // Empty new string should fail
        assert!(RenameConfig::new(temp_dir.path(), "old".to_string(), "".to_string()).is_err());
        
        // Path separator in new string should be allowed at this level (CLI validation handles this)
        assert!(RenameConfig::new(temp_dir.path(), "old".to_string(), "new/path".to_string()).is_ok());
        assert!(RenameConfig::new(temp_dir.path(), "old".to_string(), "new\\path".to_string()).is_ok());
    }
    
    #[test]
    fn test_utils_replace_all() {
        assert_eq!(utils::replace_all("hello world", "l", "x"), "hexxo worxd");
        assert_eq!(utils::replace_all("test test test", "test", "demo"), "demo demo demo");
        assert_eq!(utils::replace_all("no match", "xyz", "abc"), "no match");
    }
    
    #[test]
    fn test_utils_contains_pattern() {
        assert!(utils::contains_pattern("hello world", "hello"));
        assert!(utils::contains_pattern("hello world", "world"));
        assert!(utils::contains_pattern("hello world", "o w"));
        assert!(!utils::contains_pattern("hello world", "xyz"));
    }
    
    #[test]
    fn test_get_version() {
        let version = get_version();
        // Should not be empty
        assert!(!version.is_empty());
        // Should be a valid version format (x.y.z)
        assert!(version.contains('.'));
        // Should be parseable as a version-like string
        let parts: Vec<&str> = version.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major.minor format");
        // All parts should be numeric
        for part in &parts {
            assert!(part.parse::<u32>().is_ok(), "Version part '{}' should be numeric", part);
        }
    }
    
    #[test]
    fn test_version_consistency() {
        // Test that version.txt exists and is readable
        let version_content = std::fs::read_to_string("version.txt").expect("version.txt should exist");
        let file_version = version_content.trim();
        
        // Should match what get_version() returns
        assert_eq!(get_version(), file_version);
    }
}
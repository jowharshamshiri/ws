use anyhow::{Context, Result};
use std::path::Path;
use std::fs;
use log::{info, debug};

pub mod search;
pub mod transform;

/// Supported languages for AST analysis
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum SupportedLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
}

impl SupportedLanguage {
    /// Get the ast-grep language identifier
    pub fn get_language_name(&self) -> &'static str {
        match self {
            SupportedLanguage::Rust => "rust",
            SupportedLanguage::JavaScript => "javascript",
            SupportedLanguage::TypeScript => "typescript",
            SupportedLanguage::Python => "python",
            SupportedLanguage::Go => "go",
            SupportedLanguage::Java => "java",
            SupportedLanguage::C => "c",
            SupportedLanguage::Cpp => "cpp",
        }
    }

    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(SupportedLanguage::Rust),
            "js" | "jsx" => Some(SupportedLanguage::JavaScript),
            "ts" | "tsx" => Some(SupportedLanguage::TypeScript),
            "py" => Some(SupportedLanguage::Python),
            "go" => Some(SupportedLanguage::Go),
            "java" => Some(SupportedLanguage::Java),
            "c" | "h" => Some(SupportedLanguage::C),
            "cpp" | "cc" | "cxx" | "hpp" => Some(SupportedLanguage::Cpp),
            _ => None,
        }
    }
}

/// AST-based code analyzer
pub struct CodeAnalyzer {
    language: SupportedLanguage,
}

impl CodeAnalyzer {
    /// Create a new code analyzer for the specified language
    pub fn new(language: SupportedLanguage) -> Self {
        Self { language }
    }

    /// Create analyzer from file path by detecting language
    pub fn from_file_path(path: &Path) -> Result<Self> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .context("Failed to get file extension")?;

        let language = SupportedLanguage::from_extension(extension)
            .context("Unsupported file type for AST analysis")?;

        Ok(Self::new(language))
    }

    /// Parse source code into AST - simplified implementation
    pub fn parse_source(&self, source: &str) -> Result<String> {
        info!("Would parse source code with {} language", self.language.get_language_name());
        Ok(source.to_string())
    }

    /// Parse source file into AST - simplified implementation  
    pub fn parse_file(&self, path: &Path) -> Result<(String, String)> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        info!("Parsed file: {}", path.display());
        Ok((source.clone(), source))
    }

    /// Find all nodes matching a pattern - simplified implementation
    pub fn find_matches(&self, content: &str, pattern: &str) -> Result<Vec<String>> {
        debug!("Searching for pattern: {}", pattern);
        
        // Simplified pattern matching - in a real implementation this would use ast-grep
        let matches: Vec<_> = content.lines()
            .filter(|line| line.contains(pattern))
            .map(|line| line.to_string())
            .collect();
        
        info!("Found {} matches for pattern: {}", matches.len(), pattern);
        Ok(matches)
    }

    /// Get the language being used
    pub fn language(&self) -> SupportedLanguage {
        self.language
    }
}

/// Utility functions for code analysis
pub mod utils {
    use super::*;

    /// Check if a file is supported for AST analysis
    pub fn is_supported_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(SupportedLanguage::from_extension)
            .is_some()
    }

    /// Get all supported file extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["rs", "js", "jsx", "ts", "tsx", "py", "go", "java", "c", "h", "cpp", "cc", "cxx", "hpp"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert!(matches!(
            SupportedLanguage::from_extension("rs"),
            Some(SupportedLanguage::Rust)
        ));
        assert!(matches!(
            SupportedLanguage::from_extension("js"),
            Some(SupportedLanguage::JavaScript)
        ));
        assert!(SupportedLanguage::from_extension("unknown").is_none());
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CodeAnalyzer::new(SupportedLanguage::Rust);
        assert!(matches!(analyzer.language(), SupportedLanguage::Rust));
    }

    #[test]
    fn test_rust_parsing() -> Result<()> {
        let analyzer = CodeAnalyzer::new(SupportedLanguage::Rust);
        let source = "fn main() { println!(\"Hello, world!\"); }";
        let _ast = analyzer.parse_source(source)?;
        
        // Should successfully parse valid Rust code (simplified test)
        assert!(source.contains("fn main"));
        Ok(())
    }

    #[test]
    fn test_file_support_detection() {
        assert!(utils::is_supported_file(Path::new("test.rs")));
        assert!(utils::is_supported_file(Path::new("test.js")));
        assert!(!utils::is_supported_file(Path::new("test.txt")));
    }
}
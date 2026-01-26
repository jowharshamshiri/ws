use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use log::{info, debug, warn};

use super::{CodeAnalyzer, SupportedLanguage};

/// Transformation rule for code modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    pub name: String,
    pub pattern: String,
    pub replacement: String,
    pub language: SupportedLanguage,
}

/// Result of a transformation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    pub file_path: PathBuf,
    pub original_content: String,
    pub transformed_content: String,
    pub changes_made: usize,
    pub successful: bool,
    pub error_message: Option<String>,
}

/// Options for transformation operations
#[derive(Debug, Clone)]
pub struct TransformOptions {
    pub dry_run: bool,
    pub backup_files: bool,
    pub max_changes_per_file: Option<usize>,
    pub preserve_formatting: bool,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            backup_files: true,
            max_changes_per_file: Some(100),
            preserve_formatting: true,
        }
    }
}

/// AST-based code transformation engine
pub struct AstTransformEngine {
    options: TransformOptions,
}

impl AstTransformEngine {
    /// Create a new transformation engine with options
    pub fn new(options: TransformOptions) -> Self {
        Self { options }
    }

    /// Apply transformation rule to a single file
    pub fn transform_file(&self, file_path: &std::path::Path, rule: &TransformRule) -> Result<TransformResult> {
        let analyzer = CodeAnalyzer::from_file_path(file_path)
            .unwrap_or_else(|_| CodeAnalyzer::new(rule.language));

        let (_ast, original_content) = analyzer.parse_file(file_path)?;
        
        // Find all matches for the pattern
        let matches = analyzer.find_matches(&original_content, &rule.pattern)?;
        
        if matches.is_empty() {
            return Ok(TransformResult {
                file_path: file_path.to_path_buf(),
                original_content: original_content.clone(),
                transformed_content: original_content,
                changes_made: 0,
                successful: true,
                error_message: None,
            });
        }

        let changes_to_apply = if let Some(max_changes) = self.options.max_changes_per_file {
            std::cmp::min(matches.len(), max_changes)
        } else {
            matches.len()
        };

        debug!("Applying {} changes to {}", changes_to_apply, file_path.display());

        // Apply transformations (simplified implementation)
        let transformed_content = self.apply_simple_replacements(&original_content, &matches[..changes_to_apply], &rule.replacement)?;

        let result = TransformResult {
            file_path: file_path.to_path_buf(),
            original_content,
            transformed_content,
            changes_made: changes_to_apply,
            successful: true,
            error_message: None,
        };

        // Write changes to file if not in dry-run mode
        if !self.options.dry_run {
            self.write_transformed_content(file_path, &result)?;
        }

        info!("Transformed {} with {} changes", file_path.display(), changes_to_apply);
        Ok(result)
    }

    /// Apply transformation rule to multiple files
    pub fn transform_files(&self, file_paths: &[PathBuf], rule: &TransformRule) -> Result<Vec<TransformResult>> {
        let mut results = Vec::new();

        for file_path in file_paths {
            match self.transform_file(file_path, rule) {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to transform file {}: {}", file_path.display(), e);
                    results.push(TransformResult {
                        file_path: file_path.clone(),
                        original_content: String::new(),
                        transformed_content: String::new(),
                        changes_made: 0,
                        successful: false,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }

        let successful_transforms = results.iter().filter(|r| r.successful).count();
        let total_changes: usize = results.iter().map(|r| r.changes_made).sum();
        
        info!(
            "Transformed {}/{} files with {} total changes", 
            successful_transforms, 
            file_paths.len(),
            total_changes
        );

        Ok(results)
    }

    /// Apply replacements to source content (simplified implementation)
    fn apply_simple_replacements(&self, content: &str, matches: &[String], replacement: &str) -> Result<String> {
        let mut result = content.to_string();

        // Simple line-based replacement
        for match_line in matches {
            result = result.replace(match_line, replacement);
        }

        Ok(result)
    }

    /// Write transformed content to file
    fn write_transformed_content(&self, file_path: &std::path::Path, result: &TransformResult) -> Result<()> {
        if self.options.backup_files {
            self.create_backup(file_path)?;
        }

        std::fs::write(file_path, &result.transformed_content)
            .with_context(|| format!("Failed to write transformed content to {}", file_path.display()))?;

        Ok(())
    }

    /// Create backup of original file
    fn create_backup(&self, file_path: &std::path::Path) -> Result<()> {
        let backup_path = file_path.with_extension(
            format!("{}.bak", 
                file_path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
            )
        );

        std::fs::copy(file_path, &backup_path)
            .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

        debug!("Created backup: {}", backup_path.display());
        Ok(())
    }
}

/// Common transformation patterns
pub struct CommonTransforms;

impl CommonTransforms {
    /// Get common Rust transformations
    pub fn rust() -> Vec<TransformRule> {
        vec![
            TransformRule {
                name: "unwrap_to_expect".to_string(),
                pattern: "call_expression[function.name=\"unwrap\"]".to_string(),
                replacement: "expect(\"TODO: add meaningful error message\")".to_string(),
                language: SupportedLanguage::Rust,
            },
            TransformRule {
                name: "println_to_log_info".to_string(),
                pattern: "macro_invocation[name=\"println!\"]".to_string(),
                replacement: "log::info!".to_string(),
                language: SupportedLanguage::Rust,
            },
        ]
    }

    /// Get common JavaScript transformations
    pub fn javascript() -> Vec<TransformRule> {
        vec![
            TransformRule {
                name: "var_to_let".to_string(),
                pattern: "variable_declaration[kind=\"var\"]".to_string(),
                replacement: "let".to_string(),
                language: SupportedLanguage::JavaScript,
            },
            TransformRule {
                name: "console_log_to_logger".to_string(),
                pattern: "call_expression[callee.object.name=\"console\"][callee.property.name=\"log\"]".to_string(),
                replacement: "logger.info".to_string(),
                language: SupportedLanguage::JavaScript,
            },
        ]
    }

    /// Get transformations for the specified language
    pub fn for_language(lang: SupportedLanguage) -> Vec<TransformRule> {
        match lang {
            SupportedLanguage::Rust => Self::rust(),
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => Self::javascript(),
            _ => vec![], // Add more languages as needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_options_default() {
        let options = TransformOptions::default();
        assert!(!options.dry_run);
        assert!(options.backup_files);
        assert_eq!(options.max_changes_per_file, Some(100));
        assert!(options.preserve_formatting);
    }

    #[test]
    fn test_common_transforms() {
        let rust_transforms = CommonTransforms::rust();
        assert!(!rust_transforms.is_empty());
        assert!(rust_transforms.iter().any(|t| t.name == "unwrap_to_expect"));

        let js_transforms = CommonTransforms::javascript();
        assert!(!js_transforms.is_empty());
        assert!(js_transforms.iter().any(|t| t.name == "var_to_let"));
    }

    #[test]
    fn test_transform_engine_creation() {
        let options = TransformOptions::default();
        let engine = AstTransformEngine::new(options);
        assert!(!engine.options.dry_run);
    }
}
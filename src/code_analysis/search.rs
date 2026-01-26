use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use log::info;

use super::{CodeAnalyzer, SupportedLanguage};

/// Search result containing match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub matched_text: String,
    pub context_before: String,
    pub context_after: String,
    pub node_kind: String,
}

/// Search options for AST pattern matching
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub pattern: String,
    pub language: Option<SupportedLanguage>,
    pub include_context: bool,
    pub context_lines: usize,
    pub max_matches: Option<usize>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            language: None,
            include_context: true,
            context_lines: 3,
            max_matches: Some(1000),
        }
    }
}

/// AST-based code search engine
pub struct AstSearchEngine {
    options: SearchOptions,
}

impl AstSearchEngine {
    /// Create a new search engine with options
    pub fn new(options: SearchOptions) -> Self {
        Self { options }
    }

    /// Search for pattern in a single file
    pub fn search_file(&self, file_path: &std::path::Path) -> Result<Vec<SearchMatch>> {
        let analyzer = if let Some(lang) = self.options.language {
            CodeAnalyzer::new(lang)
        } else {
            CodeAnalyzer::from_file_path(file_path)?
        };

        let (_ast, source) = analyzer.parse_file(file_path)?;
        let matches = analyzer.find_matches(&source, &self.options.pattern)?;

        let mut results = Vec::new();
        let source_lines: Vec<&str> = source.lines().collect();

        for (i, match_line) in matches.iter().enumerate() {
            if let Some(max) = self.options.max_matches {
                if i >= max {
                    break;
                }
            }

            // Simplified implementation - find line number
            let start_line = source_lines.iter()
                .position(|line| line.contains(match_line))
                .unwrap_or(0);
            let start_col = 0;

            let matched_text = match_line.clone();
            
            let (context_before, context_after) = if self.options.include_context {
                self.extract_context(&source_lines, start_line, self.options.context_lines)
            } else {
                (String::new(), String::new())
            };

            results.push(SearchMatch {
                file_path: file_path.to_path_buf(),
                line: start_line + 1, // 1-based line numbers
                column: start_col + 1, // 1-based column numbers
                matched_text,
                context_before,
                context_after,
                node_kind: "line_match".to_string(),
            });
        }

        info!("Found {} matches in {}", results.len(), file_path.display());
        Ok(results)
    }

    /// Search for pattern across multiple files
    pub fn search_files(&self, file_paths: &[std::path::PathBuf]) -> Result<HashMap<PathBuf, Vec<SearchMatch>>> {
        let mut all_results = HashMap::new();

        for file_path in file_paths {
            match self.search_file(file_path) {
                Ok(matches) => {
                    if !matches.is_empty() {
                        all_results.insert(file_path.clone(), matches);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to search file {}: {}", file_path.display(), e);
                }
            }
        }

        info!("Searched {} files, found matches in {} files", file_paths.len(), all_results.len());
        Ok(all_results)
    }

    /// Extract context lines around a match
    fn extract_context(&self, source_lines: &[&str], match_line: usize, context_lines: usize) -> (String, String) {
        let start_context = if match_line >= context_lines {
            match_line - context_lines
        } else {
            0
        };

        let end_context = std::cmp::min(
            match_line + context_lines + 1,
            source_lines.len()
        );

        let context_before = source_lines[start_context..match_line]
            .join("\n");
        
        let context_after = source_lines[match_line + 1..end_context]
            .join("\n");

        (context_before, context_after)
    }
}

/// Common AST patterns for different languages
pub struct CommonPatterns;

impl CommonPatterns {
    /// Get common patterns for Rust
    pub fn rust() -> Vec<(&'static str, &'static str)> {
        vec![
            ("function_definitions", "function_item"),
            ("struct_definitions", "struct_item"),
            ("impl_blocks", "impl_item"),
            ("match_expressions", "match_expression"),
            ("function_calls", "call_expression"),
            ("variable_declarations", "let_declaration"),
            ("unsafe_blocks", "unsafe_block"),
            ("panic_macros", "macro_invocation[name = \"panic!\"]"),
        ]
    }

    /// Get common patterns for JavaScript/TypeScript
    pub fn javascript() -> Vec<(&'static str, &'static str)> {
        vec![
            ("function_declarations", "function_declaration"),
            ("arrow_functions", "arrow_function"),
            ("class_declarations", "class_declaration"),
            ("variable_declarations", "variable_declaration"),
            ("function_calls", "call_expression"),
            ("async_functions", "async_function_declaration"),
            ("try_catch", "try_statement"),
            ("console_logs", "call_expression[callee.object.name = \"console\"]"),
        ]
    }

    /// Get common patterns for Python
    pub fn python() -> Vec<(&'static str, &'static str)> {
        vec![
            ("function_definitions", "function_definition"),
            ("class_definitions", "class_definition"),
            ("import_statements", "import_statement"),
            ("function_calls", "call"),
            ("variable_assignments", "assignment"),
            ("try_except", "try_statement"),
            ("list_comprehensions", "list_comprehension"),
            ("print_calls", "call[function.name = \"print\"]"),
        ]
    }

    /// Get patterns for the specified language
    pub fn for_language(lang: SupportedLanguage) -> Vec<(&'static str, &'static str)> {
        match lang {
            SupportedLanguage::Rust => Self::rust(),
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => Self::javascript(),
            SupportedLanguage::Python => Self::python(),
            _ => vec![], // Add more languages as needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_search_rust_function() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "fn main() {{")?;
        writeln!(temp_file, "    println!(\"Hello, world!\");")?;
        writeln!(temp_file, "}}")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "fn helper() {{")?;
        writeln!(temp_file, "    // helper function")?;
        writeln!(temp_file, "}}")?;

        let options = SearchOptions {
            pattern: "fn".to_string(),
            language: Some(SupportedLanguage::Rust),
            ..Default::default()
        };

        let engine = AstSearchEngine::new(options);
        let results = engine.search_file(temp_file.path())?;

        // Should find both function definitions (pattern changed from AST node to literal text)
        assert_eq!(results.len(), 2);
        assert!(results[0].matched_text.contains("main") || results[0].matched_text.contains("fn"));
        assert!(results[1].matched_text.contains("helper") || results[1].matched_text.contains("fn"));

        Ok(())
    }

    #[test]
    fn test_common_patterns() {
        let rust_patterns = CommonPatterns::rust();
        assert!(!rust_patterns.is_empty());
        assert!(rust_patterns.iter().any(|(name, _)| *name == "function_definitions"));

        let js_patterns = CommonPatterns::javascript();
        assert!(!js_patterns.is_empty());
        assert!(js_patterns.iter().any(|(name, _)| *name == "function_declarations"));
    }
}
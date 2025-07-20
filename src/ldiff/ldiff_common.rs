use anyhow::Result;
use regex::Regex;
use std::io::{self, BufRead, BufReader, Write};

/// Processes a line, replacing repeated tokens with a substitute character
/// Returns the processed line and the current words for the next iteration
pub fn process_line(
    line: &str,
    previous_words: &[String],
    substitute_char: char,
) -> Result<(String, Vec<String>)> {
    // ANSI color code pattern
    let color_pattern = Regex::new(r"\x1b\[[0-9;]*[mGK]")?;

    // Comprehensive separators pattern - matches Python original
    let separators = r#"[:\.,:;!?@#$%^&*()+=\[\]{}<>~/\\|"'\-]"#;
    
    // Token pattern that closely matches the Python original
    // Captures: ANSI codes, separators/whitespace, numbers, and words
    let token_pattern = Regex::new(&format!(
        r"({})|([{}])|(\s+)|(\d+)|(\w+)",
        color_pattern.as_str(),
        separators
    ))?;

    let current_line = line.trim_end_matches('\n');
    let mut output = Vec::new();
    let mut current_words = Vec::new();

    // Find all matches and extract them
    for mat in token_pattern.find_iter(current_line) {
        let token = mat.as_str();
        
        if color_pattern.is_match(token) {
            // Preserve ANSI color codes
            output.push(token.to_string());
        } else if Regex::new(&format!(r"^[{}]$|^\s+$", separators))?.is_match(token) {
            // Preserve separators and whitespace exactly
            output.push(token.to_string());
        } else {
            // Word or number token - these are the ones we compare and potentially replace
            if current_words.len() < previous_words.len() 
                && token == previous_words[current_words.len()] {
                // Replace with substitute character pattern
                output.push(substitute_char.to_string().repeat(token.len()));
            } else {
                output.push(token.to_string());
            }
            current_words.push(token.to_string());
        }
    }

    let output_line = output.join("");
    Ok((output_line, current_words))
}

/// Processes input from stdin line by line, writing to stdout
pub fn process_stdin(substitute_char: char) -> Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();
    
    let mut previous_words = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let (processed_line, current_words) = process_line(&line, &previous_words, substitute_char)?;
        
        writeln!(stdout, "{}", processed_line)?;
        stdout.flush()?;
        
        previous_words = current_words;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_line_basic() {
        let previous_words = vec!["hello".to_string(), "world".to_string()];
        let line = "hello world test";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "░░░░░ ░░░░░ test");
        assert_eq!(words, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_process_line_partial_match() {
        let previous_words = vec!["hello".to_string()];
        let line = "hello new world";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "░░░░░ new world");
        assert_eq!(words, vec!["hello", "new", "world"]);
    }

    #[test]
    fn test_process_line_no_match() {
        let previous_words = vec!["foo".to_string(), "bar".to_string()];
        let line = "hello world";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "hello world");
        assert_eq!(words, vec!["hello", "world"]);
    }

    #[test]
    fn test_process_line_with_separators() {
        let previous_words = vec!["test".to_string(), "123".to_string()];
        let line = "test:123,new";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "░░░░:░░░,new");
        assert_eq!(words, vec!["test", "123", "new"]);
    }

    #[test]
    fn test_process_line_with_ansi_colors() {
        let previous_words = vec!["hello".to_string()];
        let line = "\x1b[31mhello\x1b[0m world";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "\x1b[31m░░░░░\x1b[0m world");
        assert_eq!(words, vec!["hello", "world"]);
    }

    #[test]
    fn test_process_line_empty() {
        let previous_words = vec![];
        let line = "";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "");
        assert_eq!(words, Vec::<String>::new());
    }

    #[test]
    fn test_process_line_whitespace_preservation() {
        let previous_words = vec!["hello".to_string()];
        let line = "  hello   world  ";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "  ░░░░░   world  ");
        assert_eq!(words, vec!["hello", "world"]);
    }

    #[test]
    fn test_process_line_numbers() {
        let previous_words = vec!["test".to_string(), "123".to_string()];
        let line = "test 123 456";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "░░░░ ░░░ 456");
        assert_eq!(words, vec!["test", "123", "456"]);
    }

    #[test]
    fn test_process_line_mixed_separators() {
        let previous_words = vec!["path".to_string(), "to".to_string()];
        let line = "/path/to/file.txt";
        let (output, words) = process_line(line, &previous_words, '░').unwrap();
        
        assert_eq!(output, "/░░░░/░░/file.txt");
        assert_eq!(words, vec!["path", "to", "file", "txt"]);
    }

    #[test]
    fn test_process_line_custom_substitute_char() {
        let previous_words = vec!["hello".to_string()];
        let line = "hello world";
        let (output, words) = process_line(line, &previous_words, '*').unwrap();
        
        assert_eq!(output, "***** world");
        assert_eq!(words, vec!["hello", "world"]);
    }
}
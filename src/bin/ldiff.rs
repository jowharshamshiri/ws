use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use nomion::ldiff::process_stdin;
use std::io;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "ldiff")]
#[command(about = "Process input lines, replacing repeated tokens with a substitute character - part of the nomion tool suite")]
#[command(long_about = "Process input lines, replacing repeated tokens with a substitute character.\nPreserves ASCII color codes, timestamps, brackets, separators, and whitespace in the output.\n\nUsage:\n  cat /var/log/system.log | tail -n 100 | ldiff\n  find / | ldiff\n  ldiff < input.txt")]
#[command(version = nomion::get_version())]
struct Args {
    /// Character to use for substitution (default: ░)
    #[arg(default_value = "░")]
    substitute_char: String,
}

fn main() {
    // Handle broken pipe errors gracefully (common when piping to head, less, etc.)
    if let Err(e) = run() {
        match e.downcast_ref::<io::Error>() {
            Some(io_err) if io_err.kind() == io::ErrorKind::BrokenPipe => {
                // Broken pipe is expected when piping to head, less, etc.
                process::exit(0);
            }
            _ => {
                eprintln!("{}: {:#}", "Error".red(), e);
                process::exit(1);
            }
        }
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    // Check if input is available from stdin
    if atty::is(atty::Stream::Stdin) {
        eprintln!("{}: No input provided. ldiff reads from stdin.", "Error".red());
        eprintln!("Usage examples:");
        eprintln!("  cat /var/log/system.log | tail -n 100 | ldiff");
        eprintln!("  find / | ldiff");
        eprintln!("  ldiff < input.txt");
        process::exit(1);
    }

    // Validate substitute character
    let substitute_char = args.substitute_char.chars().next()
        .context("Substitute character cannot be empty")?;

    if args.substitute_char.chars().count() > 1 {
        eprintln!("{}: Substitute character must be a single character, got: '{}'", 
                 "Warning".yellow(), args.substitute_char);
        eprintln!("Using first character: '{}'", substitute_char);
    }

    // Process stdin
    process_stdin(substitute_char)
        .context("Failed to process input from stdin")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(&["ldiff"]);
        assert_eq!(args.substitute_char, "░");
    }

    #[test]
    fn test_args_parsing_custom_char() {
        let args = Args::parse_from(&["ldiff", "*"]);
        assert_eq!(args.substitute_char, "*");
    }

    #[test]
    fn test_substitute_char_extraction() {
        let args = Args { substitute_char: "░".to_string() };
        let char = args.substitute_char.chars().next().unwrap();
        assert_eq!(char, '░');
    }

    #[test]
    fn test_substitute_char_extraction_multiple_chars() {
        let args = Args { substitute_char: "abc".to_string() };
        let char = args.substitute_char.chars().next().unwrap();
        assert_eq!(char, 'a');
    }

    #[test]
    fn test_help_output() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff", "--", "--help"])
            .output()
            .expect("Failed to execute command");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Process input lines"));
        assert!(stdout.contains("substitute character"));
    }

    #[test]
    fn test_version_output() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff", "--", "--version"])
            .output()
            .expect("Failed to execute command");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ldiff"));
    }

    #[test]
    fn test_basic_functionality() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "hello world").unwrap();
        writeln!(temp_file, "hello universe").unwrap();
        writeln!(temp_file, "goodbye world").unwrap();

        temp_file.flush().unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff"])
            .stdin(std::fs::File::open(temp_file.path()).unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.trim().split('\n').collect();
        
        // Filter out Cargo build messages
        let filtered_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.contains("Finished") && !line.contains("Running"))
            .copied()
            .collect();
        
        assert!(filtered_lines.len() >= 1);
        assert_eq!(filtered_lines[0], "hello world");
        assert_eq!(filtered_lines[1], "░░░░░ universe");
        assert_eq!(filtered_lines[2], "goodbye world");
    }

    #[test]
    fn test_custom_substitute_char() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test line").unwrap();
        writeln!(temp_file, "test another").unwrap();

        temp_file.flush().unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff", "--", "*"])
            .stdin(std::fs::File::open(temp_file.path()).unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.trim().split('\n').collect();
        
        // Filter out Cargo build messages
        let filtered_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.contains("Finished") && !line.contains("Running"))
            .copied()
            .collect();
        
        assert!(filtered_lines.len() >= 2);
        assert_eq!(filtered_lines[0], "test line");
        assert_eq!(filtered_lines[1], "**** another");
    }

    #[test]
    fn test_empty_input() {
        let mut temp_file = NamedTempFile::new().unwrap();
        
        temp_file.flush().unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff"])
            .stdin(std::fs::File::open(temp_file.path()).unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // For empty input, should contain only cargo messages, no actual output
        assert!(!stdout.contains("░"));
    }

    #[test]
    fn test_single_line() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "single line only").unwrap();

        temp_file.flush().unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff"])
            .stdin(std::fs::File::open(temp_file.path()).unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.trim().split('\n').collect();
        
        // Filter out Cargo build messages
        let filtered_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.contains("Finished") && !line.contains("Running"))
            .copied()
            .collect();
        
        assert!(filtered_lines.len() >= 1);
        assert_eq!(filtered_lines[0], "single line only");
    }

    #[test]
    fn test_log_file_simulation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-01-01 10:00:00 INFO Starting application").unwrap();
        writeln!(temp_file, "2023-01-01 10:00:01 INFO Loading configuration").unwrap();
        writeln!(temp_file, "2023-01-01 10:00:02 ERROR Failed to connect").unwrap();
        temp_file.flush().unwrap();

        temp_file.flush().unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff"])
            .stdin(std::fs::File::open(temp_file.path()).unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.trim().split('\n').collect();
        
        // Filter out Cargo build messages
        let filtered_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.contains("Finished") && !line.contains("Running"))
            .copied()
            .collect();
        
        assert!(filtered_lines.len() >= 3);
        assert_eq!(filtered_lines[0], "2023-01-01 10:00:00 INFO Starting application");
        assert_eq!(filtered_lines[1], "░░░░-░░-░░ ░░:░░:01 ░░░░ Loading configuration");
        assert_eq!(filtered_lines[2], "░░░░-░░-░░ ░░:░░:02 ERROR Failed to connect");
    }

    #[test]
    fn test_path_simulation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "/usr/local/bin/app").unwrap();
        writeln!(temp_file, "/usr/local/lib/libtest.so").unwrap();
        writeln!(temp_file, "/usr/share/doc/readme.txt").unwrap();

        temp_file.flush().unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--bin", "ldiff"])
            .stdin(std::fs::File::open(temp_file.path()).unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.trim().split('\n').collect();
        
        // Filter out Cargo build messages
        let filtered_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.contains("Finished") && !line.contains("Running"))
            .copied()
            .collect();
        
        assert!(filtered_lines.len() >= 1);
        assert_eq!(filtered_lines[0], "/usr/local/bin/app");
        assert_eq!(filtered_lines[1], "/░░░/░░░░░/lib/libtest.so");
        assert_eq!(filtered_lines[2], "/░░░/share/doc/readme.txt");
    }
}
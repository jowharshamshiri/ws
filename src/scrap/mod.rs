pub mod scrap_common;

pub use scrap_common::{ScrapMetadata, ScrapEntry};

use anyhow::Result;
use std::process::Command;

/// Run scrap command with the given arguments
pub fn run_scrap(args: Vec<String>) -> Result<()> {
    let output = Command::new("scrap")
        .args(&args)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Scrap command failed: {}", stderr);
    }
    
    // Print stdout from the command
    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);
    
    Ok(())
}

/// Run unscrap command with the given arguments
pub fn run_unscrap(args: Vec<String>) -> Result<()> {
    let output = Command::new("unscrap")
        .args(&args)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Unscrap command failed: {}", stderr);
    }
    
    // Print stdout from the command
    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);
    
    Ok(())
}
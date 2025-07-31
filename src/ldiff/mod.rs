pub mod ldiff_common;

pub use ldiff_common::*;

use anyhow::Result;
use std::process::Command;

/// Run ldiff command with the given arguments
pub fn run_ldiff(args: Vec<String>) -> Result<()> {
    let output = Command::new("ldiff")
        .args(&args)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Ldiff command failed: {}", stderr);
    }
    
    // Print stdout from the command
    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);
    
    Ok(())
}
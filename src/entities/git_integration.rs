// Git Integration for Session Management and Timeline Tracking

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tokio::fs;

/// Git repository manager for session tracking
pub struct GitManager {
    pub repo_path: std::path::PathBuf,
}

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub files_changed: Vec<String>,
    pub insertions: i32,
    pub deletions: i32,
}

/// File change information from git diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: String, // added, modified, deleted, renamed
    pub insertions: i32,
    pub deletions: i32,
    pub diff_content: String,
}

impl GitManager {
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    /// Initialize git repository if it doesn't exist
    pub async fn ensure_git_repo(&self) -> Result<()> {
        let git_dir = self.repo_path.join(".git");
        
        if !git_dir.exists() {
            // Check if we're inside a parent git repository
            if self.is_inside_parent_git_repo() {
                log::warn!(
                    "Initializing git repository inside parent git repo at {}. This will be isolated.",
                    self.repo_path.display()
                );
            }
            
            log::info!("Initializing isolated git repository at {}", self.repo_path.display());
            
            // Initialize with proper isolation
            let output = self.isolated_git_command()
                .arg("init")
                .output()?;
            
            if !output.status.success() {
                return Err(anyhow!("Failed to initialize git repository: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
            
            // Set up initial .gitignore for workspace files
            self.setup_gitignore().await?;
            
            // Create initial commit
            self.create_initial_commit().await?;
        }
        
        Ok(())
    }

    /// Set up .gitignore for workspace management files
    async fn setup_gitignore(&self) -> Result<()> {
        let gitignore_path = self.repo_path.join(".gitignore");
        let gitignore_content = r#"# Workspace Management Files
.ws/
CLAUDE.md
internal/

# Build and Dependencies
target/
Cargo.lock
node_modules/
*.log

# IDE and Editor Files
.vscode/
.idea/
*.swp
*.swo
*~

# OS Files
.DS_Store
Thumbs.db
"#;
        
        fs::write(gitignore_path, gitignore_content).await?;
        Ok(())
    }

    /// Create initial commit for new repository
    async fn create_initial_commit(&self) -> Result<String> {
        // Add .gitignore
        self.isolated_git_command()
            .args(&["add", ".gitignore"])
            .output()?;
        
        // Create initial commit
        let output = self.isolated_git_command()
            .args(&["commit", "-m", "Initial commit: workspace setup"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to create initial commit: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        self.get_current_commit_hash()
    }

    /// Create a commit for session end with comprehensive message
    pub async fn create_session_commit(&self, session_id: &str, summary: &str, files_modified: &[String]) -> Result<String> {
        // Stage all tracked files that have changes
        let output = Command::new("git")
            .args(&["add", "-u"])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            log::warn!("Git add warning: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // Stage any new files in src/, tests/, or other source directories
        for pattern in &["src/", "tests/", "Cargo.toml", "package.json", "*.rs", "*.js", "*.ts", "*.py"] {
            Command::new("git")
                .args(&["add", pattern])
                .current_dir(&self.repo_path)
                .output()
                .ok(); // Ignore errors for patterns that don't match
        }
        
        // Check if there are any changes to commit
        let status_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()?;
        
        if status_output.stdout.is_empty() {
            log::info!("No changes to commit for session {}", session_id);
            return self.get_current_commit_hash();
        }
        
        // Create detailed commit message
        let commit_message = format!(
            "Session {}: {}\n\nFiles modified:\n{}\n\n🤖 Automated session commit via ws",
            session_id,
            summary,
            files_modified.join("\n- ")
        );
        
        let output = Command::new("git")
            .args(&["commit", "-m", &commit_message])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to create session commit: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let commit_hash = self.get_current_commit_hash()?;
        log::info!("Created session commit {} for session {}", commit_hash, session_id);
        Ok(commit_hash)
    }

    /// Get current commit hash
    pub fn get_current_commit_hash(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to get current commit hash: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get commit information by hash
    pub fn get_commit_info(&self, commit_hash: &str) -> Result<GitCommit> {
        let output = Command::new("git")
            .args(&["show", "--format=%H|%an|%ad|%s", "--date=iso", "--stat", commit_hash])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to get commit info: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        if lines.is_empty() {
            return Err(anyhow!("Empty git show output"));
        }
        
        // Parse commit header
        let header_parts: Vec<&str> = lines[0].split('|').collect();
        if header_parts.len() != 4 {
            return Err(anyhow!("Invalid git show format"));
        }
        
        let hash = header_parts[0].to_string();
        let author = header_parts[1].to_string();
        let date_str = header_parts[2];
        let message = header_parts[3].to_string();
        
        // Parse date
        let date = DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z")
            .map_err(|e| anyhow!("Failed to parse date: {}", e))?
            .with_timezone(&Utc);
        
        // Parse file statistics
        let mut files_changed = Vec::new();
        let mut insertions = 0;
        let mut deletions = 0;
        
        for line in lines.iter().skip(1) {
            if line.contains(" | ") {
                let parts: Vec<&str> = line.split(" | ").collect();
                if !parts.is_empty() {
                    files_changed.push(parts[0].trim().to_string());
                }
            } else if line.contains(" insertion") || line.contains(" deletion") {
                // Parse summary line like "2 files changed, 15 insertions(+), 3 deletions(-)"
                for part in line.split(", ") {
                    if part.contains(" insertion") {
                        if let Ok(n) = part.split_whitespace().next().unwrap_or("0").parse::<i32>() {
                            insertions = n;
                        }
                    } else if part.contains(" deletion") {
                        if let Ok(n) = part.split_whitespace().next().unwrap_or("0").parse::<i32>() {
                            deletions = n;
                        }
                    }
                }
            }
        }
        
        Ok(GitCommit {
            hash,
            author,
            date,
            message,
            files_changed,
            insertions,
            deletions,
        })
    }

    /// Get diff between two commits
    pub fn get_diff(&self, from_commit: Option<&str>, to_commit: Option<&str>) -> Result<Vec<FileChange>> {
        let mut args = vec!["diff", "--name-status", "--numstat"];
        
        match (from_commit, to_commit) {
            (Some(from), Some(to)) => {
                args.push(from);
                args.push(to);
            }
            (Some(from), None) => {
                args.push(from);
                args.push("HEAD");
            }
            (None, Some(to)) => {
                args.push("HEAD~1");
                args.push(to);
            }
            (None, None) => {
                args.push("HEAD~1");
                args.push("HEAD");
            }
        }
        
        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to get git diff: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // Parse numstat output
        let mut file_changes = Vec::new();
        let lines = String::from_utf8_lossy(&output.stdout);
        
        for line in lines.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let insertions = parts[0].parse().unwrap_or(0);
                let deletions = parts[1].parse().unwrap_or(0);
                let path = parts[2].to_string();
                
                // Get detailed diff for this file
                let diff_content = self.get_file_diff(&path, from_commit, to_commit)?;
                
                file_changes.push(FileChange {
                    path,
                    change_type: if insertions > 0 && deletions == 0 {
                        "added".to_string()
                    } else if insertions == 0 && deletions > 0 {
                        "deleted".to_string()
                    } else {
                        "modified".to_string()
                    },
                    insertions,
                    deletions,
                    diff_content,
                });
            }
        }
        
        Ok(file_changes)
    }

    /// Get detailed diff for a specific file
    pub fn get_file_diff(&self, file_path: &str, from_commit: Option<&str>, to_commit: Option<&str>) -> Result<String> {
        let mut args = vec!["diff"];
        
        match (from_commit, to_commit) {
            (Some(from), Some(to)) => {
                args.push(from);
                args.push(to);
            }
            (Some(from), None) => {
                args.push(from);
                args.push("HEAD");
            }
            (None, Some(to)) => {
                args.push("HEAD~1");
                args.push(to);
            }
            (None, None) => {
                args.push("HEAD~1");
                args.push("HEAD");
            }
        }
        
        args.push("--");
        args.push(file_path);
        
        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Ok(String::new()); // File might not exist in one of the commits
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get list of commits for a session (by commit message pattern)
    pub fn get_session_commits(&self, session_pattern: &str) -> Result<Vec<GitCommit>> {
        let output = Command::new("git")
            .args(&["log", "--format=%H|%an|%ad|%s", "--date=iso", "--grep", session_pattern])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to get session commits: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let mut commits = Vec::new();
        let lines = String::from_utf8_lossy(&output.stdout);
        
        for line in lines.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                let hash = parts[0].to_string();
                let commit_info = self.get_commit_info(&hash)?;
                commits.push(commit_info);
            }
        }
        
        Ok(commits)
    }

    /// Get file content at a specific commit
    pub fn get_file_at_commit(&self, file_path: &str, commit_hash: &str) -> Result<String> {
        let output = Command::new("git")
            .args(&["show", &format!("{}:{}", commit_hash, file_path)])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to get file at commit: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// List all files in the repository at a specific commit
    pub fn list_files_at_commit(&self, commit_hash: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(&["ls-tree", "-r", "--name-only", commit_hash])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to list files at commit: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        
        Ok(files)
    }

    /// Check if repository exists and is valid (local only, not parent repos)
    pub fn is_git_repo(&self) -> bool {
        // Check for local .git directory first
        let local_git = self.repo_path.join(".git");
        if !local_git.exists() {
            return false;
        }
        
        // Verify it's a valid git repository using isolated git command
        Command::new("git")
            .args(&["rev-parse", "--git-dir"])
            .current_dir(&self.repo_path)
            .env("GIT_DIR", &local_git)
            .env("GIT_WORK_TREE", &self.repo_path)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if we're inside a parent git repository (potential conflict)
    pub fn is_inside_parent_git_repo(&self) -> bool {
        let mut current = self.repo_path.clone();
        current.pop(); // Go up one directory
        
        while current.parent().is_some() {
            if current.join(".git").exists() {
                return true;
            }
            if !current.pop() {
                break;
            }
        }
        false
    }

    /// Create git command with proper isolation to prevent parent repo discovery
    fn isolated_git_command(&self) -> Command {
        let mut cmd = Command::new("git");
        let git_dir = self.repo_path.join(".git");
        
        cmd.current_dir(&self.repo_path)
           .env("GIT_DIR", &git_dir)
           .env("GIT_WORK_TREE", &self.repo_path);
        
        cmd
    }
}
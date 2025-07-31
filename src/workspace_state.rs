use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Global workspace configuration and state management
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub version: u32,
    pub project_root: PathBuf,
    pub project_name: Option<String>,
    pub tools: HashMap<String, serde_json::Value>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            version: 1,
            project_root: PathBuf::new(),
            project_name: None,
            tools: HashMap::new(),
        }
    }
}

impl WorkspaceState {
    /// Initialize workspace state in a project directory
    pub fn initialize(project_root: &Path) -> Result<Self> {
        let workspace_dir = project_root.join(".ws");
        fs::create_dir_all(&workspace_dir)
            .context("Failed to create .ws directory")?;
        
        // Create subdirectories
        fs::create_dir_all(workspace_dir.join("st8").join("templates"))
            .context("Failed to create st8 templates directory")?;
        fs::create_dir_all(workspace_dir.join("st8").join("logs"))
            .context("Failed to create st8 logs directory")?;
        
        let mut state = Self::default();
        state.project_root = project_root.to_path_buf();
        state.project_name = detect_project_name(project_root);
        
        state.save(project_root)?;
        Ok(state)
    }
    
    /// Load workspace state from project directory
    pub fn load(project_root: &Path) -> Result<Self> {
        let state_file = project_root.join(".ws").join("state.json");
        
        if !state_file.exists() {
            return Self::initialize(project_root);
        }
        
        let content = fs::read_to_string(&state_file)
            .context("Failed to read workspace state file")?;
        
        let mut state: Self = serde_json::from_str(&content)
            .context("Failed to parse workspace state file")?;
        
        // Update project root in case it moved
        state.project_root = project_root.to_path_buf();
        
        Ok(state)
    }
    
    /// Save workspace state to project directory
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let workspace_dir = project_root.join(".ws");
        fs::create_dir_all(&workspace_dir)
            .context("Failed to create .ws directory")?;
        
        let state_file = workspace_dir.join("state.json");
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize workspace state")?;
        
        fs::write(&state_file, content)
            .context("Failed to write workspace state file")?;
        
        Ok(())
    }
    
    /// Get workspace directory path
    pub fn workspace_dir(&self) -> PathBuf {
        self.project_root.join(".ws")
    }
    
    /// Get tool-specific directory
    pub fn tool_dir(&self, tool_name: &str) -> PathBuf {
        self.ws_dir().join(tool_name)
    }
    
    /// Get or create tool-specific configuration
    pub fn get_tool_config<T>(&self, tool_name: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.tools.get(tool_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
    
    /// Set tool-specific configuration
    pub fn set_tool_config<T>(&mut self, tool_name: &str, config: &T) -> Result<()>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(config)
            .context("Failed to serialize tool config")?;
        
        self.tools.insert(tool_name.to_string(), value);
        Ok(())
    }
}

/// Detect project name from various project files
fn detect_project_name(project_root: &Path) -> Option<String> {
    // Check Cargo.toml
    if let Ok(cargo_content) = fs::read_to_string(project_root.join("Cargo.toml")) {
        if let Ok(cargo_toml) = cargo_content.parse::<toml::Value>() {
            if let Some(name) = cargo_toml.get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str()) {
                return Some(name.to_string());
            }
        }
    }
    
    // Check package.json
    if let Ok(package_content) = fs::read_to_string(project_root.join("package.json")) {
        if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&package_content) {
            if let Some(name) = package_json.get("name").and_then(|n| n.as_str()) {
                return Some(name.to_string());
            }
        }
    }
    
    // Check pyproject.toml
    if let Ok(pyproject_content) = fs::read_to_string(project_root.join("pyproject.toml")) {
        if let Ok(pyproject_toml) = pyproject_content.parse::<toml::Value>() {
            // Check tool.poetry.name first
            if let Some(name) = pyproject_toml.get("tool")
                .and_then(|t| t.get("poetry"))
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str()) {
                return Some(name.to_string());
            }
            
            // Check project.name
            if let Some(name) = pyproject_toml.get("project")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str()) {
                return Some(name.to_string());
            }
        }
    }
    
    // Fallback to directory name
    project_root.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_workspace_state_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let state = WorkspaceState::initialize(temp_dir.path()).unwrap();
        
        assert_eq!(state.version, 1);
        assert_eq!(state.project_root, temp_dir.path());
        assert!(temp_dir.path().join(".ws").exists());
        assert!(temp_dir.path().join(".ws").join("st8").join("templates").exists());
        assert!(temp_dir.path().join(".ws").join("state.json").exists());
    }
    
    #[test]
    fn test_workspace_state_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = WorkspaceState::initialize(temp_dir.path()).unwrap();
        
        // Set some tool config
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestConfig {
            enabled: bool,
            value: String,
        }
        
        let test_config = TestConfig {
            enabled: true,
            value: "test".to_string(),
        };
        
        state.set_tool_config("test_tool", &test_config).unwrap();
        state.save(temp_dir.path()).unwrap();
        
        // Load and verify
        let loaded_state = WorkspaceState::load(temp_dir.path()).unwrap();
        let loaded_config: TestConfig = loaded_state.get_tool_config("test_tool").unwrap();
        
        assert_eq!(loaded_config, test_config);
    }
    
    #[test]
    fn test_detect_project_name_cargo() {
        let temp_dir = TempDir::new().unwrap();
        
        let cargo_content = r#"[package]
name = "test-project"
version = "0.1.0"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        
        let name = detect_project_name(temp_dir.path());
        assert_eq!(name, Some("test-project".to_string()));
    }
    
    #[test]
    fn test_detect_project_name_package_json() {
        let temp_dir = TempDir::new().unwrap();
        
        let package_content = r#"{
  "name": "test-package",
  "version": "1.0.0"
}"#;
        fs::write(temp_dir.path().join("package.json"), package_content).unwrap();
        
        let name = detect_project_name(temp_dir.path());
        assert_eq!(name, Some("test-package".to_string()));
    }
    
    #[test]
    fn test_detect_project_name_fallback() {
        let temp_dir = TempDir::new().unwrap();
        
        let name = detect_project_name(temp_dir.path());
        assert!(name.is_some());
        // Should be the temp directory name
    }
    
    #[test]
    fn test_tool_directory_paths() {
        let temp_dir = TempDir::new().unwrap();
        let state = WorkspaceState::initialize(temp_dir.path()).unwrap();
        
        assert_eq!(state.ws_dir(), temp_dir.path().join(".ws"));
        assert_eq!(state.tool_dir("st8"), temp_dir.path().join(".ws").join("st8"));
    }
}
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A single project registered for wstemplate scanning.
///
/// Each project has exactly one entry defining:
/// - `alias`: the identifier used in `{{ projects.ALIAS.* }}` template vars
/// - `root`: the directory tree to scan for `.wstemplate` files on `ws update`
///
/// All other projects referenced in a template are resolved dynamically by
/// scanning the `root` directory for `.ws/state.json` files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WstemplateEntry {
    pub root: PathBuf,
    pub alias: String,
}

/// Per-project workspace configuration, persisted as `.ws/state.json`.
///
/// Each project has at most one [`WstemplateEntry`] defining its alias and
/// scan root. Cross-project references are resolved dynamically by scanning
/// the root for sibling `.ws/state.json` files — no explicit cross-project
/// entries are needed. Loading a state file with more than one entry is a
/// hard error.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub version: u32,
    pub project_root: PathBuf,
    pub project_name: Option<String>,
    pub tools: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub wstemplate_entries: Vec<WstemplateEntry>,
    /// Set to true after the shell completion hint has been shown once.
    #[serde(default)]
    pub completion_hint_shown: bool,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            version: 1,
            project_root: PathBuf::new(),
            project_name: None,
            tools: HashMap::new(),
            wstemplate_entries: Vec::new(),
            completion_hint_shown: false,
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
        fs::create_dir_all(workspace_dir.join("templates"))
            .context("Failed to create templates directory")?;
        fs::create_dir_all(workspace_dir.join("logs"))
            .context("Failed to create logs directory")?;

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

        // Enforce single-entry wstemplate model.
        // Each project has at most one entry (its own alias + scan root).
        // All other projects are discovered dynamically by scanning the root.
        if state.wstemplate_entries.len() > 1 {
            anyhow::bail!(
                "state.json at {} has {} wstemplate entries, expected at most 1. \
                 Each project should have exactly one entry (its own alias + scan root). \
                 Remove extra entries from {}",
                state_file.display(),
                state.wstemplate_entries.len(),
                state_file.display()
            );
        }

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
        self.workspace_dir().join(tool_name)
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

    /// Return the project's wstemplate entry, if configured.
    ///
    /// Each project has at most one entry: its own alias and scan root.
    /// All other projects are discovered dynamically by scanning the root.
    pub fn wstemplate_entry(&self) -> Option<&WstemplateEntry> {
        self.wstemplate_entries.first()
    }

    /// Set the project's wstemplate entry, replacing any existing entry.
    ///
    /// Each project has exactly ONE entry: its own alias and scan root.
    pub fn set_wstemplate_entry(&mut self, entry: WstemplateEntry) {
        self.wstemplate_entries = vec![entry];
    }

    /// Remove the project's wstemplate entry entirely.
    pub fn clear_wstemplate_entry(&mut self) {
        self.wstemplate_entries.clear();
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
        assert!(temp_dir.path().join(".ws").join("templates").exists());
        assert!(temp_dir.path().join(".ws").join("state.json").exists());
        assert!(state.wstemplate_entries.is_empty());
        assert!(!state.completion_hint_shown);
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
    fn test_wstemplate_entry_roundtrips_through_state_json() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = WorkspaceState::initialize(temp_dir.path()).unwrap();

        let entry = WstemplateEntry {
            alias: "my_project".to_string(),
            root: PathBuf::from("/some/scan/root"),
        };
        state.set_wstemplate_entry(entry.clone());
        state.save(temp_dir.path()).unwrap();

        let loaded = WorkspaceState::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.wstemplate_entries.len(), 1);
        assert_eq!(loaded.wstemplate_entries[0], entry);
    }

    #[test]
    fn test_state_without_wstemplate_entries_loads_with_empty_vec() {
        // State JSON written without the wstemplate_entries field must deserialize
        // to an empty Vec (via serde default), not fail.
        let temp_dir = TempDir::new().unwrap();
        let ws_dir = temp_dir.path().join(".ws");
        std::fs::create_dir_all(&ws_dir).unwrap();
        let state_json = r#"{
            "version": 1,
            "project_root": "/some/path",
            "project_name": "test",
            "tools": {}
        }"#;
        std::fs::write(ws_dir.join("state.json"), state_json).unwrap();

        let state = WorkspaceState::load(temp_dir.path()).unwrap();
        assert!(state.wstemplate_entries.is_empty(),
            "missing wstemplate_entries field must default to empty Vec");
        assert!(!state.completion_hint_shown,
            "missing completion_hint_shown field must default to false");
    }

    #[test]
    fn test_wstemplate_entry_returns_configured_entry() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = WorkspaceState::initialize(temp_dir.path()).unwrap();
        assert!(state.wstemplate_entry().is_none());

        state.set_wstemplate_entry(WstemplateEntry {
            alias: "my_alias".to_string(),
            root: PathBuf::from("/scan/root"),
        });
        assert_eq!(state.wstemplate_entry().unwrap().alias, "my_alias");
    }

    #[test]
    fn test_set_wstemplate_entry_replaces_not_appends() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = WorkspaceState::initialize(temp_dir.path()).unwrap();

        state.set_wstemplate_entry(WstemplateEntry {
            alias: "first".to_string(),
            root: PathBuf::from("/scan/root"),
        });
        state.set_wstemplate_entry(WstemplateEntry {
            alias: "second".to_string(),
            root: PathBuf::from("/scan/root"),
        });

        assert_eq!(state.wstemplate_entries.len(), 1);
        assert_eq!(state.wstemplate_entry().unwrap().alias, "second");
    }

    #[test]
    fn test_clear_wstemplate_entry() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = WorkspaceState::initialize(temp_dir.path()).unwrap();

        state.set_wstemplate_entry(WstemplateEntry {
            alias: "test".to_string(),
            root: PathBuf::from("/root"),
        });
        assert!(state.wstemplate_entry().is_some());

        state.clear_wstemplate_entry();
        assert!(state.wstemplate_entry().is_none());
        assert!(state.wstemplate_entries.is_empty());
    }

    #[test]
    fn test_load_fails_hard_on_multi_entry() {
        let temp_dir = TempDir::new().unwrap();
        let ws_dir = temp_dir.path().join(".ws");
        fs::create_dir_all(&ws_dir).unwrap();

        let state_json = serde_json::json!({
            "version": 1,
            "project_root": temp_dir.path().to_str().unwrap(),
            "project_name": "test",
            "tools": {},
            "wstemplate_entries": [
                {"alias": "self_alias", "root": "/scan/root"},
                {"alias": "other_proj", "root": "/some/other"},
                {"alias": "third_proj", "root": "/yet/another"}
            ]
        });
        fs::write(
            ws_dir.join("state.json"),
            serde_json::to_string_pretty(&state_json).unwrap(),
        ).unwrap();

        let err = WorkspaceState::load(temp_dir.path()).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("3 wstemplate entries"),
            "error must report the count, got: {}",
            msg
        );
        assert!(
            msg.contains("expected at most 1"),
            "error must explain the constraint, got: {}",
            msg
        );
    }

    #[test]
    fn test_completion_hint_shown_persists() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = WorkspaceState::initialize(temp_dir.path()).unwrap();
        assert!(!state.completion_hint_shown);

        state.completion_hint_shown = true;
        state.save(temp_dir.path()).unwrap();

        let loaded = WorkspaceState::load(temp_dir.path()).unwrap();
        assert!(loaded.completion_hint_shown,
            "completion_hint_shown must persist through save/load");
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

        assert_eq!(state.workspace_dir(), temp_dir.path().join(".ws"));
        assert_eq!(state.tool_dir("st8"), temp_dir.path().join(".ws").join("st8"));
    }
}

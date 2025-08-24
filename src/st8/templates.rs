use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

use crate::workspace_state::WorkspaceState;
use crate::st8::VersionInfo;

/// Template configuration and metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateConfig {
    pub name: String,
    pub description: Option<String>,
    pub source_path: String,
    pub output_path: String,
    pub enabled: bool,
}

/// Template manager for st8
pub struct TemplateManager {
    templates_dir: PathBuf,
    tera_engine: Tera,
    templates: HashMap<String, TemplateConfig>,
}

impl TemplateManager {
    /// Initialize template manager for a project
    pub fn new(workspace_state: &WorkspaceState) -> Result<Self> {
        let templates_dir = workspace_state.workspace_dir().join("templates");
        fs::create_dir_all(&templates_dir)
            .context("Failed to create templates directory")?;
        
        // Initialize empty Tera engine
        let mut tera_engine = Tera::default();
        
        // Load template configurations
        let templates = Self::load_template_configs(&templates_dir)?;
        
        // Register templates with Tera
        for template_config in templates.values() {
            let template_path = templates_dir.join(&template_config.source_path);
            if template_path.exists() {
                let template_content = fs::read_to_string(&template_path)
                    .with_context(|| format!("Failed to read template: {}", template_path.display()))?;
                
                tera_engine.add_raw_template(&template_config.name, &template_content)
                    .with_context(|| format!("Failed to register template: {}", template_config.name))?;
            }
        }
        
        Ok(Self {
            templates_dir,
            tera_engine,
            templates,
        })
    }
    
    /// Add a new template
    pub fn add_template(&mut self, name: &str, template_content: &str, output_path: &str, description: Option<String>) -> Result<()> {
        let template_filename = format!("{}.tera", name);
        let template_path = self.templates_dir.join(&template_filename);
        
        // Write template file
        fs::write(&template_path, template_content)
            .with_context(|| format!("Failed to write template file: {}", template_path.display()))?;
        
        // Create template config
        let template_config = TemplateConfig {
            name: name.to_string(),
            description,
            source_path: template_filename,
            output_path: output_path.to_string(),
            enabled: true,
        };
        
        // Register with Tera
        self.tera_engine.add_raw_template(name, template_content)
            .with_context(|| format!("Failed to register template: {}", name))?;
        
        // Store config
        self.templates.insert(name.to_string(), template_config);
        
        // Save configurations
        self.save_template_configs()?;
        
        Ok(())
    }
    
    /// Remove a template
    pub fn remove_template(&mut self, name: &str) -> Result<bool> {
        if let Some(template_config) = self.templates.remove(name) {
            let template_path = self.templates_dir.join(&template_config.source_path);
            
            // Remove template file if it exists
            if template_path.exists() {
                fs::remove_file(&template_path)
                    .with_context(|| format!("Failed to remove template file: {}", template_path.display()))?;
            }
            
            // Remove from Tera (note: Tera doesn't have a remove method, so we reconstruct)
            self.rebuild_tera_engine()?;
            
            // Save updated configurations
            self.save_template_configs()?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// List all templates
    pub fn list_templates(&self) -> Vec<&TemplateConfig> {
        self.templates.values().collect()
    }
    
    /// Get a specific template
    pub fn get_template(&self, name: &str) -> Option<&TemplateConfig> {
        self.templates.get(name)
    }
    
    /// Enable or disable a template
    pub fn set_template_enabled(&mut self, name: &str, enabled: bool) -> Result<bool> {
        if let Some(template_config) = self.templates.get_mut(name) {
            template_config.enabled = enabled;
            self.save_template_configs()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Render all enabled templates
    pub fn render_all_templates(&self, version_info: &VersionInfo, project_name: Option<&str>) -> Result<Vec<String>> {
        let mut rendered_files = Vec::new();
        let context = self.create_template_context(version_info, project_name);
        
        for template_config in self.templates.values() {
            if template_config.enabled {
                match self.render_template(template_config, &context) {
                    Ok(output_path) => {
                        rendered_files.push(output_path);
                    }
                    Err(e) => {
                        log::warn!("Failed to render template '{}': {}", template_config.name, e);
                    }
                }
            }
        }
        
        Ok(rendered_files)
    }
    
    /// Render a specific template
    pub fn render_template(&self, template_config: &TemplateConfig, context: &TeraContext) -> Result<String> {
        let rendered_content = self.tera_engine.render(&template_config.name, context)
            .with_context(|| format!("Failed to render template: {}", template_config.name))?;
        
        // Write to output file
        let output_path = PathBuf::from(&template_config.output_path);
        
        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
        }
        
        fs::write(&output_path, rendered_content)
            .with_context(|| format!("Failed to write rendered template to: {}", output_path.display()))?;
        
        Ok(output_path.display().to_string())
    }
    
    /// Create template context with all available variables
    fn create_template_context(&self, version_info: &VersionInfo, project_name: Option<&str>) -> TeraContext {
        let mut context = TeraContext::new();
        
        // Project information
        let mut project = HashMap::new();
        project.insert("version".to_string(), version_info.full_version.clone());
        project.insert("major_version".to_string(), version_info.major_version.clone());
        project.insert("minor_version".to_string(), version_info.minor_version.to_string());
        project.insert("patch_version".to_string(), version_info.patch_version.to_string());
        
        if let Some(name) = project_name {
            project.insert("name".to_string(), name.to_string());
        }
        
        context.insert("project", &project);
        
        // Date and time
        let now = chrono::Local::now();
        let mut datetime = HashMap::new();
        datetime.insert("iso".to_string(), now.to_rfc3339());
        datetime.insert("date".to_string(), now.format("%Y-%m-%d").to_string());
        datetime.insert("time".to_string(), now.format("%H:%M:%S").to_string());
        datetime.insert("year".to_string(), now.format("%Y").to_string());
        datetime.insert("month".to_string(), now.format("%m").to_string());
        datetime.insert("day".to_string(), now.format("%d").to_string());
        
        context.insert("datetime", &datetime);
        
        context
    }
    
    /// Load template configurations from disk
    fn load_template_configs(templates_dir: &Path) -> Result<HashMap<String, TemplateConfig>> {
        let config_file = templates_dir.join("templates.json");
        
        if !config_file.exists() {
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(&config_file)
            .context("Failed to read template configurations")?;
        
        let configs: HashMap<String, TemplateConfig> = serde_json::from_str(&content)
            .context("Failed to parse template configurations")?;
        
        Ok(configs)
    }
    
    /// Save template configurations to disk
    fn save_template_configs(&self) -> Result<()> {
        let config_file = self.templates_dir.join("templates.json");
        let content = serde_json::to_string_pretty(&self.templates)
            .context("Failed to serialize template configurations")?;
        
        fs::write(&config_file, content)
            .context("Failed to write template configurations")?;
        
        Ok(())
    }
    
    /// Rebuild Tera engine after template removal
    fn rebuild_tera_engine(&mut self) -> Result<()> {
        self.tera_engine = Tera::default();
        
        for template_config in self.templates.values() {
            let template_path = self.templates_dir.join(&template_config.source_path);
            if template_path.exists() {
                let template_content = fs::read_to_string(&template_path)
                    .with_context(|| format!("Failed to read template: {}", template_path.display()))?;
                
                self.tera_engine.add_raw_template(&template_config.name, &template_content)
                    .with_context(|| format!("Failed to register template: {}", template_config.name))?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_state(temp_dir: &Path) -> WorkspaceState {
        WorkspaceState::initialize(temp_dir).unwrap()
    }
    
    #[test]
    fn test_template_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state(temp_dir.path());
        
        let manager = TemplateManager::new(&state).unwrap();
        assert!(manager.templates_dir.exists());
        assert_eq!(manager.list_templates().len(), 0);
    }
    
    #[test]
    fn test_add_and_remove_template() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state(temp_dir.path());
        let mut manager = TemplateManager::new(&state).unwrap();
        
        let template_content = "# {{ project.name }} v{{ project.version }}\n\nRelease notes for version {{ project.version }}.";
        let output_path = "RELEASE_NOTES.md";
        
        // Add template
        manager.add_template("release_notes", template_content, output_path, Some("Release notes template".to_string())).unwrap();
        
        assert_eq!(manager.list_templates().len(), 1);
        let template = manager.get_template("release_notes").unwrap();
        assert_eq!(template.name, "release_notes");
        assert_eq!(template.output_path, output_path);
        assert!(template.enabled);
        
        // Remove template
        let removed = manager.remove_template("release_notes").unwrap();
        assert!(removed);
        assert_eq!(manager.list_templates().len(), 0);
        
        // Try to remove non-existent template
        let removed = manager.remove_template("non_existent").unwrap();
        assert!(!removed);
    }
    
    #[test]
    fn test_enable_disable_template() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state(temp_dir.path());
        let mut manager = TemplateManager::new(&state).unwrap();
        
        let template_content = "Version: {{ project.version }}";
        manager.add_template("version", template_content, "VERSION.txt", None).unwrap();
        
        // Disable template
        let result = manager.set_template_enabled("version", false).unwrap();
        assert!(result);
        
        let template = manager.get_template("version").unwrap();
        assert!(!template.enabled);
        
        // Enable template
        let result = manager.set_template_enabled("version", true).unwrap();
        assert!(result);
        
        let template = manager.get_template("version").unwrap();
        assert!(template.enabled);
        
        // Try to modify non-existent template
        let result = manager.set_template_enabled("non_existent", true).unwrap();
        assert!(!result);
    }
    
    #[test]
    fn test_template_context_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state(temp_dir.path());
        let manager = TemplateManager::new(&state).unwrap();
        
        let version_info = VersionInfo {
            major_version: "v1.0".to_string(),
            minor_version: 5,
            patch_version: 100,
            full_version: "1.0.5.100".to_string(),
        };
        
        let context = manager.create_template_context(&version_info, Some("test-project"));
        
        // Verify context contains expected values
        let project = context.get("project").unwrap();
        assert!(project.is_object());
        
        let datetime = context.get("datetime").unwrap();
        assert!(datetime.is_object());
    }
    
    #[test]
    fn test_render_template() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state(temp_dir.path());
        let mut manager = TemplateManager::new(&state).unwrap();
        
        let template_content = "# {{ project.name | default(value='Unknown') }} v{{ project.version }}\n\nReleased on {{ datetime.date }}\n\n## Changes\n- Version bump to {{ project.version }}";
        let output_path = temp_dir.path().join("CHANGELOG.md");
        
        manager.add_template("changelog", template_content, output_path.to_str().unwrap(), None).unwrap();
        
        let version_info = VersionInfo {
            major_version: "v1.0".to_string(),
            minor_version: 5,
            patch_version: 100,
            full_version: "1.0.5.100".to_string(),
        };
        
        let rendered_files = manager.render_all_templates(&version_info, Some("test-project")).unwrap();
        
        assert_eq!(rendered_files.len(), 1);
        assert!(output_path.exists());
        
        let rendered_content = fs::read_to_string(&output_path).unwrap();
        assert!(rendered_content.contains("# test-project v1.0.5.100"));
        assert!(rendered_content.contains("Version bump to 1.0.5.100"));
    }
    
    #[test]
    fn test_template_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state(temp_dir.path());
        
        // Create manager and add template
        {
            let mut manager = TemplateManager::new(&state).unwrap();
            manager.add_template("test", "{{ project.version }}", "version.txt", Some("Test template".to_string())).unwrap();
        }
        
        // Create new manager and verify template persisted
        {
            let manager = TemplateManager::new(&state).unwrap();
            assert_eq!(manager.list_templates().len(), 1);
            
            let template = manager.get_template("test").unwrap();
            assert_eq!(template.description, Some("Test template".to_string()));
        }
    }
}
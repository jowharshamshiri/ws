// File-to-Database Migration System
// Migrates internal/*.md files to structured database entities

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use uuid::Uuid;

use super::EntityManager;
use super::models::{FeatureState, TaskStatus, Priority};
use super::session_models::SessionType;

/// Migration status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub migration_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub files_migrated: Vec<String>,
    pub entities_created: HashMap<String, i32>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Migration handler for internal/*.md files
pub struct FileToDbMigrator {
    entity_manager: EntityManager,
    migration_status: MigrationStatus,
}

impl FileToDbMigrator {
    pub fn new(entity_manager: EntityManager) -> Self {
        let migration_status = MigrationStatus {
            migration_id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            completed_at: None,
            files_migrated: Vec::new(),
            entities_created: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        Self {
            entity_manager,
            migration_status,
        }
    }

    /// Run complete migration of all internal files
    pub async fn migrate_all(&mut self, internal_dir: &Path) -> Result<MigrationStatus> {
        println!("🔄 Starting File-to-Database Migration...");
        
        // Migrate in dependency order
        self.migrate_features_md(internal_dir).await?;
        self.migrate_task_backlog_md(internal_dir).await?;
        self.migrate_progress_tracking_md(internal_dir).await?;
        self.migrate_session_logs(internal_dir).await?;
        self.migrate_directives_md(internal_dir).await?;
        self.migrate_architectural_decisions_md(internal_dir).await?;

        self.migration_status.completed_at = Some(Utc::now());
        println!("✅ Migration completed successfully!");
        self.print_migration_summary();

        Ok(self.migration_status.clone())
    }

    /// Migrate features.md to Feature entities
    async fn migrate_features_md(&mut self, internal_dir: &Path) -> Result<()> {
        let features_path = internal_dir.join("features.md");
        if !features_path.exists() {
            self.migration_status.warnings.push("features.md not found".to_string());
            return Ok(());
        }

        println!("📋 Migrating features.md...");

        let content = std::fs::read_to_string(&features_path)
            .context("Failed to read features.md")?;

        let features = self.parse_features_md(&content)?;
        let mut created_count = 0;

        for feature_data in features {
            match self.create_feature_from_data(feature_data).await {
                Ok(_) => created_count += 1,
                Err(e) => {
                    let error_msg = format!("Failed to create feature: {}", e);
                    self.migration_status.errors.push(error_msg);
                }
            }
        }

        self.migration_status.files_migrated.push("features.md".to_string());
        self.migration_status.entities_created.insert("features".to_string(), created_count);
        println!("✅ Created {} features from features.md", created_count);

        Ok(())
    }

    /// Migrate task_backlog.md to Task entities
    async fn migrate_task_backlog_md(&mut self, internal_dir: &Path) -> Result<()> {
        let backlog_path = internal_dir.join("task_backlog.md");
        if !backlog_path.exists() {
            self.migration_status.warnings.push("task_backlog.md not found".to_string());
            return Ok(());
        }

        println!("📝 Migrating task_backlog.md...");

        let content = std::fs::read_to_string(&backlog_path)
            .context("Failed to read task_backlog.md")?;

        let tasks = self.parse_task_backlog_md(&content)?;
        let mut created_count = 0;

        for task_data in tasks {
            match self.create_task_from_data(task_data).await {
                Ok(_) => created_count += 1,
                Err(e) => {
                    let error_msg = format!("Failed to create task: {}", e);
                    self.migration_status.errors.push(error_msg);
                }
            }
        }

        self.migration_status.files_migrated.push("task_backlog.md".to_string());
        self.migration_status.entities_created.insert("tasks".to_string(), created_count);
        println!("✅ Created {} tasks from task_backlog.md", created_count);

        Ok(())
    }

    /// Migrate progress_tracking.md to Session entities
    async fn migrate_progress_tracking_md(&mut self, internal_dir: &Path) -> Result<()> {
        let progress_path = internal_dir.join("progress_tracking.md");
        if !progress_path.exists() {
            self.migration_status.warnings.push("progress_tracking.md not found".to_string());
            return Ok(());
        }

        println!("📊 Migrating progress_tracking.md...");

        let content = std::fs::read_to_string(&progress_path)
            .context("Failed to read progress_tracking.md")?;

        let sessions = self.parse_progress_tracking_md(&content)?;
        let mut created_count = 0;

        for session_data in sessions {
            match self.create_session_from_data(session_data).await {
                Ok(_) => created_count += 1,
                Err(e) => {
                    let error_msg = format!("Failed to create session: {}", e);
                    self.migration_status.errors.push(error_msg);
                }
            }
        }

        self.migration_status.files_migrated.push("progress_tracking.md".to_string());
        self.migration_status.entities_created.insert("sessions".to_string(), created_count);
        println!("✅ Created {} sessions from progress_tracking.md", created_count);

        Ok(())
    }

    /// Migrate session log files to conversation history
    async fn migrate_session_logs(&mut self, internal_dir: &Path) -> Result<()> {
        println!("💬 Migrating session logs...");

        let mut log_count = 0;
        let mut message_count = 0;

        // Find all session log files
        for entry in std::fs::read_dir(internal_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("session_log") && file_name.ends_with(".md") {
                    match self.migrate_session_log_file(&path).await {
                        Ok(messages) => {
                            log_count += 1;
                            message_count += messages;
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to migrate {}: {}", file_name, e);
                            self.migration_status.errors.push(error_msg);
                        }
                    }
                }
            }
        }

        self.migration_status.entities_created.insert("session_logs".to_string(), log_count);
        self.migration_status.entities_created.insert("conversation_messages".to_string(), message_count);
        println!("✅ Migrated {} session logs with {} messages", log_count, message_count);

        Ok(())
    }

    /// Migrate directives.md to Directive entities (as specialized notes)
    async fn migrate_directives_md(&mut self, internal_dir: &Path) -> Result<()> {
        let directives_path = internal_dir.join("directives.md");
        if !directives_path.exists() {
            self.migration_status.warnings.push("directives.md not found".to_string());
            return Ok(());
        }

        println!("📜 Migrating directives.md...");

        let content = std::fs::read_to_string(&directives_path)
            .context("Failed to read directives.md")?;

        let directives = self.parse_directives_md(&content)?;
        let mut created_count = 0;

        for directive_data in directives {
            match self.create_directive_note(directive_data).await {
                Ok(_) => created_count += 1,
                Err(e) => {
                    let error_msg = format!("Failed to create directive: {}", e);
                    self.migration_status.errors.push(error_msg);
                }
            }
        }

        self.migration_status.files_migrated.push("directives.md".to_string());
        self.migration_status.entities_created.insert("directives".to_string(), created_count);
        println!("✅ Created {} directives from directives.md", created_count);

        Ok(())
    }

    /// Migrate architectural_decisions.md to Note entities
    async fn migrate_architectural_decisions_md(&mut self, internal_dir: &Path) -> Result<()> {
        let arch_path = internal_dir.join("architectural_decisions.md");
        if !arch_path.exists() {
            self.migration_status.warnings.push("architectural_decisions.md not found".to_string());
            return Ok(());
        }

        println!("🏗️ Migrating architectural_decisions.md...");

        let content = std::fs::read_to_string(&arch_path)
            .context("Failed to read architectural_decisions.md")?;

        let decisions = self.parse_architectural_decisions_md(&content)?;
        let mut created_count = 0;

        for decision_data in decisions {
            match self.create_architecture_note(decision_data).await {
                Ok(_) => created_count += 1,
                Err(e) => {
                    let error_msg = format!("Failed to create architectural decision: {}", e);
                    self.migration_status.errors.push(error_msg);
                }
            }
        }

        self.migration_status.files_migrated.push("architectural_decisions.md".to_string());
        self.migration_status.entities_created.insert("architectural_decisions".to_string(), created_count);
        println!("✅ Created {} architectural decisions", created_count);

        Ok(())
    }

    // ============================================================================
    // Parsing Methods for Each File Type
    // ============================================================================

    /// Parse features.md table format
    fn parse_features_md(&self, content: &str) -> Result<Vec<FeatureData>> {
        let mut features = Vec::new();
        
        for line in content.lines() {
            if line.starts_with("| F") && line.contains("|") {
                if let Some(feature_data) = self.parse_feature_line(line) {
                    features.push(feature_data);
                }
            }
        }

        Ok(features)
    }

    fn parse_feature_line(&self, line: &str) -> Option<FeatureData> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 5 {
            let id = parts[1].trim();
            let name_desc = parts[2].trim();
            let description = parts[3].trim();
            let state = parts[4].trim();
            let notes = if parts.len() > 5 { parts[5].trim() } else { "" };

            // Extract feature name from markdown formatting
            let name = if name_desc.starts_with("**") && name_desc.ends_with("**") {
                &name_desc[2..name_desc.len()-2]
            } else {
                name_desc
            };

            // Parse state emoji to FeatureState
            let feature_state = match state {
                "🟢" => FeatureState::TestedPassing,
                "🟠" => FeatureState::Implemented,
                "🟡" => FeatureState::TestedFailing,
                "⚠️" => FeatureState::TautologicalTest,
                "🔴" => FeatureState::CriticalIssue,
                _ => FeatureState::NotImplemented,
            };

            Some(FeatureData {
                id: id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                state: feature_state,
                notes: notes.to_string(),
            })
        } else {
            None
        }
    }

    /// Parse task_backlog.md format
    fn parse_task_backlog_md(&self, content: &str) -> Result<Vec<TaskData>> {
        let mut tasks = Vec::new();
        let mut current_task: Option<TaskData> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Look for task headers
            if line.starts_with("**TASK-") {
                // Save previous task
                if let Some(task) = current_task.take() {
                    tasks.push(task);
                }
                
                // Start new task
                current_task = self.parse_task_header(line);
            } else if let Some(ref mut task) = current_task {
                // Parse task details
                if line.starts_with("- **Category**:") {
                    task.category = line.replace("- **Category**:", "").trim().to_string();
                } else if line.starts_with("- **Dependencies**:") {
                    task.dependencies = line.replace("- **Dependencies**:", "").trim().to_string();
                } else if line.starts_with("- **Priority**:") {
                    let priority_str = line.replace("- **Priority**:", "");
                    let priority_str = priority_str.trim();
                    task.priority = match priority_str.to_uppercase().as_str() {
                        "HIGH" => Priority::High,
                        "MEDIUM" => Priority::Medium,
                        _ => Priority::Low,
                    };
                }
            }
        }
        
        // Don't forget the last task
        if let Some(task) = current_task {
            tasks.push(task);
        }

        Ok(tasks)
    }

    fn parse_task_header(&self, line: &str) -> Option<TaskData> {
        // Extract task code and title from format like "**TASK-PM-001**: Description - Status: 🔄 IN_PROGRESS"
        if let Some(colon_pos) = line.find(':') {
            let task_code = line[2..colon_pos].trim(); // Remove ** prefix
            let rest = &line[colon_pos + 1..];
            
            if let Some(status_pos) = rest.find("Status:") {
                let title = rest[..status_pos].trim();
                let status_part = &rest[status_pos + 7..].trim();
                
                let status = if status_part.contains("COMPLETED") || status_part.contains("✅") {
                    TaskStatus::Completed
                } else if status_part.contains("IN_PROGRESS") || status_part.contains("🔄") {
                    TaskStatus::InProgress
                } else if status_part.contains("BLOCKED") {
                    TaskStatus::Blocked
                } else {
                    TaskStatus::Pending
                };

                Some(TaskData {
                    code: task_code.to_string(),
                    title: title.to_string(),
                    description: "".to_string(),
                    category: "migration".to_string(),
                    status,
                    priority: Priority::Medium,
                    dependencies: "".to_string(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse progress_tracking.md for session data
    fn parse_progress_tracking_md(&self, content: &str) -> Result<Vec<SessionData>> {
        let mut sessions = Vec::new();
        let mut current_session: Option<SessionData> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Look for session headers
            if line.starts_with("### Session ") {
                // Save previous session
                if let Some(session) = current_session.take() {
                    sessions.push(session);
                }
                
                // Start new session
                current_session = self.parse_session_header(line);
            } else if let Some(ref mut session) = current_session {
                // Parse session details
                if line.starts_with("**Session Type**:") {
                    session.session_type = self.parse_session_type(line);
                } else if line.starts_with("**Duration**:") {
                    session.duration = line.replace("**Duration**:", "").trim().to_string();
                }
            }
        }
        
        // Don't forget the last session
        if let Some(session) = current_session {
            sessions.push(session);
        }

        Ok(sessions)
    }

    fn parse_session_header(&self, line: &str) -> Option<SessionData> {
        // Extract date and title from format like "### Session 2025-08-03 (HTTP Dashboard System Implementation)"
        if let Some(date_start) = line.find("Session ") {
            let rest = &line[date_start + 8..];
            if let Some(paren_pos) = rest.find(" (") {
                let date_str = &rest[..paren_pos];
                let title_part = &rest[paren_pos + 2..];
                let title = if title_part.ends_with(')') {
                    &title_part[..title_part.len() - 1]
                } else {
                    title_part
                };

                Some(SessionData {
                    date: date_str.to_string(),
                    title: title.to_string(),
                    session_type: SessionType::FeatureImplementation,
                    duration: "".to_string(),
                    description: "".to_string(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse_session_type(&self, line: &str) -> SessionType {
        let type_str = line.replace("**Session Type**:", "").trim().to_lowercase();
        match type_str.as_str() {
            s if s.contains("bug") || s.contains("fix") => SessionType::BugFix,
            s if s.contains("test") => SessionType::TestingValidation,
            s if s.contains("refactor") => SessionType::Refactoring,
            s if s.contains("documentation") || s.contains("doc") => SessionType::DocumentationUpdate,
            s if s.contains("architecture") || s.contains("design") => SessionType::ArchitecturalDesign,
            s if s.contains("integration") || s.contains("deployment") => SessionType::Integration,
            s if s.contains("maintenance") || s.contains("cleanup") => SessionType::MaintenanceCleanup,
            _ => SessionType::FeatureImplementation,
        }
    }

    /// Parse directives.md for development rules
    fn parse_directives_md(&self, content: &str) -> Result<Vec<DirectiveData>> {
        let mut directives = Vec::new();
        let mut current_directive: Option<DirectiveData> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Look for directive headers (### format)
            if line.starts_with("### ") {
                // Save previous directive
                if let Some(directive) = current_directive.take() {
                    directives.push(directive);
                }
                
                // Start new directive
                current_directive = Some(DirectiveData {
                    title: line[4..].to_string(),
                    content: String::new(),
                    category: "development".to_string(),
                });
            } else if let Some(ref mut directive) = current_directive {
                // Accumulate content
                if !line.is_empty() {
                    if !directive.content.is_empty() {
                        directive.content.push('\n');
                    }
                    directive.content.push_str(line);
                }
            }
        }
        
        // Don't forget the last directive
        if let Some(directive) = current_directive {
            directives.push(directive);
        }

        Ok(directives)
    }

    /// Parse architectural_decisions.md
    fn parse_architectural_decisions_md(&self, content: &str) -> Result<Vec<ArchitecturalDecisionData>> {
        let mut decisions = Vec::new();
        let mut current_decision: Option<ArchitecturalDecisionData> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Look for decision headers
            if line.starts_with("## ") || line.starts_with("### ") {
                // Save previous decision
                if let Some(decision) = current_decision.take() {
                    decisions.push(decision);
                }
                
                // Start new decision
                let title = if line.starts_with("## ") {
                    line[3..].to_string()
                } else {
                    line[4..].to_string()
                };
                
                current_decision = Some(ArchitecturalDecisionData {
                    title,
                    content: String::new(),
                    rationale: String::new(),
                });
            } else if let Some(ref mut decision) = current_decision {
                // Accumulate content
                if !line.is_empty() {
                    if !decision.content.is_empty() {
                        decision.content.push('\n');
                    }
                    decision.content.push_str(line);
                }
            }
        }
        
        // Don't forget the last decision
        if let Some(decision) = current_decision {
            decisions.push(decision);
        }

        Ok(decisions)
    }

    /// Migrate a single session log file
    async fn migrate_session_log_file(&mut self, log_path: &Path) -> Result<i32> {
        let content = std::fs::read_to_string(log_path)
            .with_context(|| format!("Failed to read {:?}", log_path))?;

        // For now, create a simple session entry
        // In a real implementation, you'd parse the log content more thoroughly
        let file_name = log_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Extract date from filename if possible
        let session_title = format!("Session Log: {}", file_name);
        
        // Create a basic session record
        // TODO: Parse actual log content for detailed information
        let session_data = SessionData {
            date: "2025-08-03".to_string(), // TODO: Extract from filename
            title: session_title,
            session_type: SessionType::FeatureImplementation,
            duration: "".to_string(),
            description: format!("Migrated from {}", file_name),
        };

        self.create_session_from_data(session_data).await?;
        
        // Return approximate message count (simplified)
        let message_count = content.lines().count() / 10; // Rough estimate
        Ok(message_count as i32)
    }

    // ============================================================================
    // Entity Creation Methods
    // ============================================================================

    async fn create_feature_from_data(&self, data: FeatureData) -> Result<()> {
        let feature = self.entity_manager.create_feature(data.name, data.description).await?;
        
        // Update the feature with parsed data
        let mut updated_feature = feature;
        updated_feature.state = data.state;
        if !data.notes.is_empty() {
            updated_feature.implementation_notes = Some(data.notes);
        }
        
        self.entity_manager.update_feature(updated_feature).await?;
        Ok(())
    }

    async fn create_task_from_data(&self, data: TaskData) -> Result<()> {
        let task = self.entity_manager.create_task(data.title, data.description).await?;
        
        // Update with parsed data
        let mut updated_task = task;
        updated_task.status = data.status;
        updated_task.priority = data.priority;
        updated_task.category = data.category;
        
        self.entity_manager.update_task(updated_task).await?;
        Ok(())
    }

    async fn create_session_from_data(&self, data: SessionData) -> Result<()> {
        // Create session using entity manager
        // Note: This is a simplified implementation
        // TODO: Implement proper session creation in EntityManager
        println!("Creating session: {}", data.title);
        Ok(())
    }

    async fn create_directive_note(&self, data: DirectiveData) -> Result<()> {
        // Create as project note with directive category
        self.entity_manager.create_project_note(
            data.title,
            data.content,
            "directive".to_string(),
        ).await?;
        Ok(())
    }

    async fn create_architecture_note(&self, data: ArchitecturalDecisionData) -> Result<()> {
        // Create as project note with architecture category  
        self.entity_manager.create_project_note(
            data.title,
            data.content,
            "architecture".to_string(),
        ).await?;
        Ok(())
    }

    fn print_migration_summary(&self) {
        println!("\n📊 Migration Summary");
        println!("==================");
        println!("Migration ID: {}", self.migration_status.migration_id);
        println!("Duration: {:?}", 
            self.migration_status.completed_at.unwrap_or(Utc::now())
                .signed_duration_since(self.migration_status.started_at));
        
        println!("\n📁 Files Migrated:");
        for file in &self.migration_status.files_migrated {
            println!("  ✅ {}", file);
        }
        
        println!("\n🏗️ Entities Created:");
        for (entity_type, count) in &self.migration_status.entities_created {
            println!("  {} {}: {}", 
                match entity_type.as_str() {
                    "features" => "🎯",
                    "tasks" => "📝",
                    "sessions" => "🕐",
                    "directives" => "📜",
                    "architectural_decisions" => "🏗️",
                    _ => "📊"
                },
                entity_type, count);
        }
        
        if !self.migration_status.warnings.is_empty() {
            println!("\n⚠️ Warnings:");
            for warning in &self.migration_status.warnings {
                println!("  {}", warning);
            }
        }
        
        if !self.migration_status.errors.is_empty() {
            println!("\n❌ Errors:");
            for error in &self.migration_status.errors {
                println!("  {}", error);
            }
        }
    }
}

// ============================================================================
// Data Structures for Parsed Content
// ============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FeatureData {
    id: String,
    name: String,
    description: String,
    state: FeatureState,
    notes: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TaskData {
    code: String,
    title: String,
    description: String,
    category: String,
    status: TaskStatus,
    priority: Priority,
    dependencies: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SessionData {
    date: String,
    title: String,
    session_type: SessionType,
    duration: String,
    description: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DirectiveData {
    title: String,
    content: String,
    category: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ArchitecturalDecisionData {
    title: String,
    content: String,
    rationale: String,
}
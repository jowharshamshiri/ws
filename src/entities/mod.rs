// Workspace Entity Management System
// Comprehensive entity-based project management with SQLite backend

pub mod models;
pub mod database;
pub mod crud;
pub mod notes;
pub mod relationships;
pub mod state_machine;
pub mod session_models;
pub mod api_models;
pub mod migration;
pub mod conversation_manager;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct FeatureData {
    pub code: String,
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub priority: Priority,
    pub state: FeatureState,
    pub dependencies: Vec<String>,
}

/// Entity types that can have notes attached
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "entity_type", rename_all = "snake_case")]
pub enum EntityType {
    Project,
    Feature,
    Task,
    Session,
    Directive,
    Template,
    Test,
    Dependency,
    Note,
}

/// Feature implementation states following the state machine
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "feature_state", rename_all = "snake_case")]
pub enum FeatureState {
    NotImplemented,    // ❌
    Implemented,       // 🟠 - implemented but needs tests
    TestedPassing,     // 🟢 - implemented and tests pass
    TestedFailing,     // 🟡 - implemented, tests fail
    TautologicalTest,  // ⚠️ - fake/broken tests detected
    CriticalIssue,     // 🔴 - critical issue requiring attention
    Completed,         // Legacy: completed state
    InProgress,        // Legacy: in progress state
    Testing,           // Legacy: testing state
}

impl ToString for FeatureState {
    fn to_string(&self) -> String {
        match self {
            FeatureState::NotImplemented => "not_implemented".to_string(),
            FeatureState::Implemented => "implemented".to_string(),
            FeatureState::TestedPassing => "tested_passing".to_string(),
            FeatureState::TestedFailing => "tested_failing".to_string(),
            FeatureState::TautologicalTest => "tautological_test".to_string(),
            FeatureState::CriticalIssue => "critical_issue".to_string(),
            FeatureState::Completed => "completed".to_string(),
            FeatureState::InProgress => "in_progress".to_string(),
            FeatureState::Testing => "testing".to_string(),
        }
    }
}

/// Task status workflow
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Completed => "completed",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

/// Task and feature priority levels
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "priority", rename_all = "snake_case")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Session states
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "session_state", rename_all = "snake_case")]
pub enum SessionState {
    Active,
    Completed,
    Interrupted,
}

/// Note types for categorization
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "note_type", rename_all = "snake_case")]
pub enum NoteType {
    Architecture,
    Decision,
    Reminder,
    Observation,
    Reference,
    Evidence,
    Progress,
    Issue,
}

/// Directive categories
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "directive_category", rename_all = "snake_case")]
pub enum DirectiveCategory {
    Development,
    Testing,
    Deployment,
    Security,
    Workflow,
    Architecture,
    Communication,
}

/// Base trait for all entities
pub trait Entity {
    fn id(&self) -> Uuid;
    fn entity_type(&self) -> EntityType;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn title(&self) -> &str;
    fn description(&self) -> Option<&str>;
}

/// Entity manager for CRUD operations and relationships
pub struct EntityManager {
    pool: SqlitePool,
}

impl EntityManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize database with all required tables
    pub async fn initialize_database(&self) -> Result<()> {
        database::initialize_tables(&self.pool).await
    }

    /// Create comprehensive project snapshot for dashboard
    pub async fn get_project_snapshot(&self, project_id: &str) -> Result<ProjectSnapshot> {
        let project = self.get_project(project_id).await?;
        let features = self.get_features_by_project(project_id).await?;
        let tasks = self.get_tasks_by_project(project_id).await?;
        let sessions = self.get_sessions_by_project(project_id).await?;
        let directives = self.get_directives_by_project(project_id).await?;
        
        // Calculate metrics
        let feature_stats = self.calculate_feature_stats(&features);
        let task_stats = self.calculate_task_stats(&tasks);
        let session_stats = self.calculate_session_stats(&sessions);
        
        Ok(ProjectSnapshot {
            project,
            features,
            tasks,
            sessions,
            directives,
            feature_stats,
            task_stats,
            session_stats,
        })
    }

    /// Search across all entities
    pub async fn search_entities(
        &self,
        _project_id: Uuid,
        _query: &str,
        _entity_types: Option<Vec<EntityType>>,
    ) -> Result<SearchResults> {
        // Implementation will search across all entity types
        todo!("Implement comprehensive entity search")
    }

    /// Migrate features from features.md to database
    pub async fn migrate_features_from_file(&self, features_md_path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(features_md_path)?;
        
        // Parse features.md table format
        let features = self.parse_features_md(&content)?;
        
        // Insert features into database
        for feature_data in features {
            let feature = self.create_feature(
                feature_data.name,
                feature_data.description,
            ).await?;
            
            // Update the feature with additional properties
            let mut updated_feature = feature;
            updated_feature.code = feature_data.code;
            updated_feature.state = feature_data.state;
            updated_feature.category = feature_data.category;
            updated_feature.priority = feature_data.priority;
            
            self.update_feature(updated_feature).await?;
        }
        
        Ok(())
    }

    /// Parse features.md table format
    fn parse_features_md(&self, content: &str) -> Result<Vec<FeatureData>> {
        let mut features = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        // Find table rows (lines starting with | F)
        for line in lines {
            let line = line.trim();
            if line.starts_with("| F") && line.contains("|") {
                if let Some(feature_data) = self.parse_feature_row(line)? {
                    features.push(feature_data);
                }
            }
        }
        
        Ok(features)
    }

    /// Parse individual feature table row
    fn parse_feature_row(&self, line: &str) -> Result<Option<FeatureData>> {
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() < 5 {
            return Ok(None);
        }
        
        let code = parts[1].to_string();
        let name_desc = parts[2];
        let description = parts[3].to_string();
        let state_str = parts[4];
        let _notes = if parts.len() > 5 { parts[5].to_string() } else { String::new() };
        
        // Extract feature name from **Feature Name** format
        let name = if name_desc.starts_with("**") && name_desc.ends_with("**") {
            name_desc[2..name_desc.len()-2].to_string()
        } else {
            name_desc.to_string()
        };
        
        // Parse state emoji to FeatureState
        let state = match state_str {
            "🟢" => FeatureState::TestedPassing,
            "🟠" => FeatureState::Implemented,
            "🟡" => FeatureState::TestedFailing,
            "⚠️" => FeatureState::TautologicalTest,
            "🔴" => FeatureState::CriticalIssue,
            "❌" | _ => FeatureState::NotImplemented,
        };
        
        Ok(Some(FeatureData {
            code,
            name,
            description,
            category: Some("core".to_string()), // Default category
            priority: Priority::Medium, // Default priority
            state,
            dependencies: Vec::new(), // Parse from notes if needed
        }))
    }

    /// Get entity relationships
    pub async fn get_entity_relationships(
        &self,
        entity_id: Uuid,
    ) -> Result<HashMap<EntityType, Vec<Uuid>>> {
        relationships::get_relationships(&self.pool, entity_id).await
    }

    /// Add note to any entity
    pub async fn add_entity_note(
        &self,
        entity_id: Uuid,
        entity_type: EntityType,
        note_type: NoteType,
        title: String,
        content: String,
    ) -> Result<models::Note> {
        notes::create_entity_note(
            &self.pool,
            entity_id,
            entity_type,
            note_type,
            title,
            content,
        ).await
    }

    /// Get all notes for an entity
    pub async fn get_entity_notes(
        &self,
        entity_id: Uuid,
    ) -> Result<Vec<models::Note>> {
        notes::get_notes_for_entity(&self.pool, entity_id).await
    }

    /// List all notes in project
    pub async fn list_notes(&self) -> Result<Vec<models::Note>> {
        notes::list_all(&self.pool).await
    }

    /// Search notes by content
    pub async fn search_notes(&self, _params: &crate::mcp_server::NoteSearchParams) -> Result<Vec<models::Note>> {
        // Basic implementation - just return all notes for now
        self.list_notes().await
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<models::Session>> {
        crud::sessions::list_all(&self.pool).await
    }

    /// Start a new session
    pub async fn start_session(&self, description: String) -> Result<models::Session> {
        // For now, use a default project ID - in real implementation, get from context
        let default_project_id = uuid::Uuid::new_v4();
        crud::sessions::create(&self.pool, default_project_id, "New Session".to_string(), Some(description)).await
    }

    /// End a session
    pub async fn end_session(&self, _id: &str) -> Result<models::Session> {
        // Basic implementation - return placeholder session
        Ok(models::Session {
            id: models::SqliteUuid::new(),
            project_id: models::SqliteUuid::new(),
            title: "Ended Session".to_string(),
            description: Some("Session ended".to_string()),
            state: SessionState::Completed,
            started_at: chrono::Utc::now(),
            ended_at: Some(chrono::Utc::now()),
            summary: None,
            achievements: None,
            files_modified: None,
            features_worked: None,
            tasks_completed: None,
            next_priority: None,
            reminder: None,
            validation_evidence: None,
            context_remaining: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// List all features
    pub async fn list_features(&self) -> Result<Vec<models::Feature>> {
        crud::features::list_all(&self.pool).await
    }

    /// Create a new feature
    pub async fn create_feature(&self, name: String, description: String) -> Result<models::Feature> {
        // Use the current project
        let project = self.get_current_project().await?;
        crud::features::create(&self.pool, &project.id, name, description, None, Priority::Medium).await
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Result<Vec<models::Task>> {
        crud::tasks::list_all(&self.pool).await
    }

    /// Create a new task
    pub async fn create_task(&self, title: String, description: String) -> Result<models::Task> {
        let project = self.get_current_project().await?;
        crud::tasks::create(&self.pool, &project.id, title, description, "feature".to_string(), Priority::Medium, None).await
    }

    /// Get task by ID
    pub async fn get_task(&self, id: &str) -> Result<models::Task> {
        crud::tasks::get(&self.pool, id).await
    }

    /// Create a new project
    pub async fn create_project(&self, name: String) -> Result<models::Project> {
        crud::projects::create(&self.pool, name, None, None).await
    }

    /// Get current project (get or create workspace project)
    pub async fn get_current_project(&self) -> Result<models::Project> {
        // Try to get existing projects first
        let projects = crud::projects::list_all(&self.pool).await?;
        
        if let Some(project) = projects.first() {
            Ok(project.clone())
        } else {
            // Create the workspace project
            let current_dir = std::env::current_dir()
                .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
            let project_name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Workspace Project")
                .to_string();
            
            crud::projects::create(
                &self.pool,
                project_name,
                Some("AI-assisted development tool suite".to_string()),
                None
            ).await
        }
    }

    /// Create a note
    pub async fn create_note(&self, _entity_type: String, _entity_id: String, content: String, category: Option<String>) -> Result<models::Note> {
        let project = self.get_current_project().await?;
        let note_type = match category.as_deref() {
            Some("architecture") => NoteType::Architecture,
            Some("decision") => NoteType::Decision,
            Some("reminder") => NoteType::Reminder,
            Some("observation") => NoteType::Observation,
            Some("evidence") => NoteType::Evidence,
            Some("progress") => NoteType::Progress,
            Some("issue") => NoteType::Issue,
            _ => NoteType::Reference,
        };
        notes::create_project_note(&self.pool, &project.id, note_type, "Note".to_string(), content).await
    }

    /// Create a project note
    pub async fn create_project_note(&self, title: String, content: String, category: String) -> Result<models::Note> {
        let project = self.get_current_project().await?;
        let note_type = match category.as_str() {
            "architecture" => NoteType::Architecture,
            "decision" => NoteType::Decision,
            "reminder" => NoteType::Reminder,
            "observation" => NoteType::Observation,
            "evidence" => NoteType::Evidence,
            "progress" => NoteType::Progress,
            "issue" => NoteType::Issue,
            _ => NoteType::Reference,
        };
        notes::create_project_note(&self.pool, &project.id, note_type, title, content).await
    }

    /// Update task
    pub async fn update_task(&self, task: models::Task) -> Result<models::Task> {
        crud::tasks::update(&self.pool, task).await
    }

    /// Get feature by ID
    pub async fn get_feature(&self, id: &str) -> Result<models::Feature> {
        crud::features::get(&self.pool, id).await
    }

    /// Update feature
    pub async fn update_feature(&self, feature: models::Feature) -> Result<models::Feature> {
        crud::features::update(&self.pool, feature).await
    }


    /// Find features by parameters (placeholder)
    pub async fn find_features(&self, _params: &crate::mcp_server::ListFeaturesQuery) -> Result<Vec<models::Feature>> {
        // Basic implementation - just return all features
        self.list_features().await
    }

    // Entity-specific methods will be implemented in crud module
    pub async fn get_project(&self, id: &str) -> Result<models::Project> {
        crud::projects::get(&self.pool, id).await
    }

    pub async fn get_features_by_project(&self, project_id: &str) -> Result<Vec<models::Feature>> {
        crud::features::get_by_project(&self.pool, project_id).await
    }

    pub async fn get_tasks_by_project(&self, project_id: &str) -> Result<Vec<models::Task>> {
        crud::tasks::get_by_project(&self.pool, project_id).await
    }

    pub async fn get_sessions_by_project(&self, project_id: &str) -> Result<Vec<models::Session>> {
        crud::sessions::get_by_project(&self.pool, project_id).await
    }

    pub async fn get_directives_by_project(&self, project_id: &str) -> Result<Vec<models::Directive>> {
        crud::directives::get_by_project(&self.pool, project_id).await
    }

    fn calculate_feature_stats(&self, features: &[models::Feature]) -> FeatureStats {
        let total = features.len();
        let mut stats = FeatureStats::default();
        
        for feature in features {
            match feature.state {
                FeatureState::NotImplemented => stats.not_implemented += 1,
                FeatureState::Implemented => stats.implemented += 1,
                FeatureState::TestedPassing => stats.tested_passing += 1,
                FeatureState::TestedFailing => stats.tested_failing += 1,
                FeatureState::TautologicalTest => stats.tautological_test += 1,
                FeatureState::CriticalIssue => stats.critical_issue += 1,
                // Legacy states - map to modern equivalents for stats
                FeatureState::Completed => stats.tested_passing += 1,
                FeatureState::InProgress => stats.implemented += 1,
                FeatureState::Testing => stats.tested_failing += 1,
            }
        }
        
        stats.total = total;
        stats.implementation_percentage = if total > 0 {
            ((stats.implemented + stats.tested_passing + stats.tested_failing + stats.tautological_test) as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        stats.test_coverage_percentage = if total > 0 {
            ((stats.tested_passing + stats.tested_failing + stats.tautological_test) as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        stats.quality_percentage = if total > 0 {
            (stats.tested_passing as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        stats
    }

    fn calculate_task_stats(&self, tasks: &[models::Task]) -> TaskStats {
        let total = tasks.len();
        let mut stats = TaskStats::default();
        
        for task in tasks {
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::InProgress => stats.in_progress += 1,
                TaskStatus::Blocked => stats.blocked += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Cancelled => stats.cancelled += 1,
            }
        }
        
        stats.total = total;
        stats.completion_percentage = if total > 0 {
            (stats.completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        stats
    }

    fn calculate_session_stats(&self, sessions: &[models::Session]) -> SessionStats {
        let total = sessions.len();
        let active = sessions.iter().filter(|s| matches!(s.state, SessionState::Active)).count();
        let completed = sessions.iter().filter(|s| matches!(s.state, SessionState::Completed)).count();
        
        SessionStats {
            total,
            active,
            completed,
            interrupted: total - active - completed,
        }
    }
}

/// Project statistics for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStats {
    pub total: usize,
    pub not_implemented: usize,
    pub implemented: usize,
    pub tested_passing: usize,
    pub tested_failing: usize,
    pub tautological_test: usize,
    pub critical_issue: usize,
    pub implementation_percentage: f64,
    pub test_coverage_percentage: f64,
    pub quality_percentage: f64,
}

impl Default for FeatureStats {
    fn default() -> Self {
        Self {
            total: 0,
            not_implemented: 0,
            implemented: 0,
            tested_passing: 0,
            tested_failing: 0,
            tautological_test: 0,
            critical_issue: 0,
            implementation_percentage: 0.0,
            test_coverage_percentage: 0.0,
            quality_percentage: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub completed: usize,
    pub cancelled: usize,
    pub completion_percentage: f64,
}

impl Default for TaskStats {
    fn default() -> Self {
        Self {
            total: 0,
            pending: 0,
            in_progress: 0,
            blocked: 0,
            completed: 0,
            cancelled: 0,
            completion_percentage: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total: usize,
    pub active: usize,
    pub completed: usize,
    pub interrupted: usize,
}

/// Comprehensive project snapshot for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub project: models::Project,
    pub features: Vec<models::Feature>,
    pub tasks: Vec<models::Task>,
    pub sessions: Vec<models::Session>,
    pub directives: Vec<models::Directive>,
    pub feature_stats: FeatureStats,
    pub task_stats: TaskStats,
    pub session_stats: SessionStats,
}

/// Search results across all entity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub features: Vec<models::Feature>,
    pub tasks: Vec<models::Task>,
    pub sessions: Vec<models::Session>,
    pub notes: Vec<models::Note>,
    pub total_count: usize,
}
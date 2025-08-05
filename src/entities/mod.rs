// Workspace Entity Management System
// Comprehensive entity-based project management with SQLite backend

pub mod models;
pub mod database;
pub mod crud;
pub mod notes;
pub mod relationships;
pub mod state_machine;
pub mod session_models;
pub mod validation;
pub mod api_models;
pub mod migration;
pub mod conversation_manager;
pub mod git_integration;
pub mod audit;

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
// No UUID imports - all entities use string IDs

// FeatureData struct removed - was only used for file-based migration

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
    Milestone,
    Note,
    AuditTrail,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Project => write!(f, "project"),
            EntityType::Feature => write!(f, "feature"),
            EntityType::Task => write!(f, "task"),
            EntityType::Session => write!(f, "session"),
            EntityType::Directive => write!(f, "directive"),
            EntityType::Template => write!(f, "template"),
            EntityType::Test => write!(f, "test"),
            EntityType::Dependency => write!(f, "dependency"),
            EntityType::Milestone => write!(f, "milestone"),
            EntityType::Note => write!(f, "note"),
            EntityType::AuditTrail => write!(f, "audit_trail"),
        }
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
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

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Active => "active",
            SessionState::Completed => "completed",
            SessionState::Interrupted => "interrupted",
        }
    }
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
    fn id(&self) -> &str;
    fn entity_type(&self) -> EntityType;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn title(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Entity manager for CRUD operations and relationships
pub struct EntityManager {
    pool: SqlitePool,
    validator: validation::EntityValidator,
}

impl EntityManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { 
            pool,
            validator: validation::EntityValidator::new(),
        }
    }

    /// Get access to the database pool for advanced operations
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
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
        _project_id: &str,
        _query: &str,
        _entity_types: Option<Vec<EntityType>>,
    ) -> Result<SearchResults> {
        // Implementation will search across all entity types
        todo!("Implement comprehensive entity search")
    }

    // File-based feature migration removed - use database-only operations

    // File-based feature parsing removed - use database-only operations

    // File-based feature row parsing removed - use database-only operations

    /// Get entity relationships
    pub async fn get_entity_relationships(
        &self,
        entity_id: &str,
    ) -> Result<HashMap<EntityType, Vec<String>>> {
        relationships::get_relationships(&self.pool, entity_id).await
    }

    /// Add note to any entity
    pub async fn add_entity_note(
        &self,
        entity_id: &str,
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
        entity_id: &str,
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

    /// Start a new session with git repository initialization
    pub async fn start_session(&self, description: String) -> Result<models::Session> {
        // Initialize git repository if needed
        let current_dir = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
        let git_manager = git_integration::GitManager::new(&current_dir);
        git_manager.ensure_git_repo().await
            .context("Failed to initialize git repository")?;
        
        // Create session in database
        let project = self.get_current_project().await?;
        crud::sessions::create(&self.pool, &project.id, "New Session".to_string(), Some(description)).await
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: &str) -> Result<models::Session> {
        crud::sessions::get_by_id(&self.pool, id).await
    }

    /// End a session with automatic git commit
    pub async fn end_session(&self, id: &str) -> Result<models::Session> {
        // Get the session
        let mut session = self.get_session(id).await?;
        
        // Get current directory for git operations
        let current_dir = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
        let git_manager = git_integration::GitManager::new(&current_dir);
        
        // Create session commit if git repo exists
        let commit_id = if git_manager.is_git_repo() {
            let summary = session.description.as_deref().unwrap_or("Session completed");
            let files_modified = vec!["src/".to_string(), "tests/".to_string()]; // TODO: track actual files
            
            match git_manager.create_session_commit(id, summary, &files_modified).await {
                Ok(commit_hash) => Some(commit_hash),
                Err(e) => {
                    log::warn!("Failed to create session commit: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Update session in database
        session.state = SessionState::Completed;
        session.ended_at = Some(chrono::Utc::now());
        session.updated_at = chrono::Utc::now();
        
        // Store commit ID if available
        if let Some(commit_hash) = commit_id {
            session.metadata = Some(format!(r#"{{"commit_id": "{}"}}"#, commit_hash));
        }
        
        crud::sessions::update(&self.pool, session).await
    }

    /// Get session metrics by session ID (returns latest metrics)
    pub async fn get_session_metrics(&self, session_id: &str) -> Result<crate::mcp_server::SessionMetrics> {
        // Try to get latest metrics from database
        if let Some(metrics) = crud::session_metrics::get_latest_by_session_id(&self.pool, session_id).await? {
            Ok(metrics)
        } else {
            // Return default metrics if none found
            Ok(crate::mcp_server::SessionMetrics {
                session_id: session_id.to_string(),
                session_duration_seconds: 0,
                total_messages: 0,
                tool_calls: 0,
                context_usage_tokens: 0,
                average_response_time_ms: 0,
                peak_response_time_ms: 0,
                total_tool_calls: 0,
                total_response_time_ms: 0,
                context_used: 0,
                session_duration_ms: 0,
                features_created: 0,
                features_updated: 0,
                tasks_created: 0,
                tasks_completed: 0,
                files_modified: 0,
                issues_resolved: 0,
                timestamp: chrono::Utc::now(),
            })
        }
    }

    /// Store session metrics in database for timeseries tracking
    pub async fn store_session_metrics(&self, _session_id: &str, metrics: &crate::mcp_server::SessionMetrics) -> Result<()> {
        crud::session_metrics::store(&self.pool, metrics).await
    }

    /// Get all session metrics for a session (full timeseries)
    pub async fn get_session_metrics_timeseries(&self, session_id: &str) -> Result<Vec<crate::mcp_server::SessionMetrics>> {
        crud::session_metrics::get_by_session_id(&self.pool, session_id).await
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

    /// Create a new directive
    pub async fn create_directive(
        &self,
        project_id: &str,
        title: String,
        rule: String,
        category: DirectiveCategory,
        priority: Priority,
        context: Option<String>,
    ) -> Result<models::Directive> {
        crud::directives::create(&self.pool, project_id, title, rule, category, priority, context).await
    }

    /// Get directive by ID
    pub async fn get_directive(&self, id: &str) -> Result<models::Directive> {
        crud::directives::get(&self.pool, id).await
    }

    /// Get all directives for current project
    pub async fn get_directives(&self, project_id: &str) -> Result<Vec<models::Directive>> {
        crud::directives::get_by_project(&self.pool, project_id).await
    }

    /// Get active directives for current project
    pub async fn get_active_directives(&self, project_id: &str) -> Result<Vec<models::Directive>> {
        crud::directives::get_active(&self.pool, project_id).await
    }

    /// Update directive
    pub async fn update_directive(&self, directive: models::Directive) -> Result<models::Directive> {
        crud::directives::update(&self.pool, directive).await
    }

    /// Delete directive (soft delete)
    pub async fn delete_directive(&self, id: &str) -> Result<()> {
        crud::directives::delete(&self.pool, id).await
    }

    // Milestone management methods
    
    /// Create new milestone
    pub async fn create_milestone(
        &self,
        project_id: &str,
        title: &str,
        description: &str,
        target_date: Option<DateTime<Utc>>,
        feature_ids: Option<Vec<String>>,
        success_criteria: Option<Vec<String>>,
    ) -> Result<models::Milestone> {
        crud::milestones::create(
            &self.pool,
            project_id,
            title,
            description,
            target_date,
            feature_ids,
            success_criteria,
        ).await
    }

    /// Get milestone by ID
    pub async fn get_milestone(&self, id: &str) -> Result<Option<models::Milestone>> {
        crud::milestones::get_by_id(&self.pool, id).await
    }

    /// Get all milestones for a project
    pub async fn get_milestones_by_project(&self, project_id: &str) -> Result<Vec<models::Milestone>> {
        crud::milestones::get_by_project(&self.pool, project_id).await
    }

    /// Get milestones by status
    pub async fn get_milestones_by_status(&self, project_id: &str, status: &str) -> Result<Vec<models::Milestone>> {
        crud::milestones::get_by_status(&self.pool, project_id, status).await
    }

    /// Update milestone
    pub async fn update_milestone(
        &self,
        id: &str,
        title: Option<&str>,
        description: Option<&str>,
        target_date: Option<Option<DateTime<Utc>>>,
        status: Option<&str>,
        completion_percentage: Option<f64>,
        feature_ids: Option<Vec<String>>,
        success_criteria: Option<Vec<String>>,
    ) -> Result<models::Milestone> {
        crud::milestones::update(
            &self.pool,
            id,
            title,
            description,
            target_date,
            status,
            completion_percentage,
            feature_ids,
            success_criteria,
        ).await
    }

    /// Mark milestone as achieved
    pub async fn achieve_milestone(&self, id: &str) -> Result<models::Milestone> {
        crud::milestones::mark_achieved(&self.pool, id).await
    }

    /// Delete milestone
    pub async fn delete_milestone(&self, id: &str) -> Result<()> {
        crud::milestones::delete(&self.pool, id).await
    }

    /// Validate entity against all applicable rules
    pub fn validate_entity(&self, entity: &dyn Entity, project_id: &str, operation: validation::ValidationOperation) -> validation::ValidationResult {
        let context = validation::ValidationContext {
            project_id: project_id.to_string(),
            operation,
            current_user: None,
            metadata: std::collections::HashMap::new(),
        };
        
        self.validator.validate_entity(entity, &context)
    }

    // F0131 Entity State Tracking - Audit Trail Methods

    /// Get audit trail for specific entity
    pub async fn get_entity_audit_trail(
        &self,
        entity_id: &str,
        entity_type: Option<EntityType>,
    ) -> Result<Vec<models::EntityAuditTrail>> {
        audit::get_entity_audit_trail(&self.pool, entity_id, entity_type).await
    }

    /// Query audit trail with filters
    pub async fn query_audit_trail(
        &self,
        query: &models::AuditTrailQuery,
    ) -> Result<Vec<models::EntityAuditTrail>> {
        audit::query_audit_trail(&self.pool, query).await
    }

    /// Get audit trail statistics
    pub async fn get_audit_statistics(
        &self,
        project_id: Option<&str>,
    ) -> Result<HashMap<String, i64>> {
        audit::get_audit_statistics(&self.pool, project_id).await
    }

    /// Get entity state at specific timestamp (for rollback analysis)
    pub async fn get_entity_state_at_timestamp(
        &self,
        entity_id: &str,
        entity_type: EntityType,
        target_timestamp: DateTime<Utc>,
    ) -> Result<HashMap<String, String>> {
        audit::get_entity_state_at_timestamp(&self.pool, entity_id, entity_type, target_timestamp).await
    }

    /// Record operation audit trail (internal helper)
    pub async fn record_operation_audit(
        &self,
        entity_id: &str,
        entity_type: EntityType,
        project_id: &str,
        operation_type: String,
        triggered_by: String,
        session_id: Option<String>,
        change_reason: Option<String>,
        entity_data: Option<String>,
    ) -> Result<models::EntityAuditTrail> {
        audit::record_operation_audit(
            &self.pool,
            entity_id,
            entity_type,
            project_id,
            operation_type,
            triggered_by,
            session_id,
            change_reason,
            entity_data,
        ).await
    }

    /// Cleanup old audit records
    pub async fn cleanup_old_audit_records(&self, older_than_days: i64) -> Result<i64> {
        audit::cleanup_old_audit_records(&self.pool, older_than_days).await
    }

    /// Validate entity before create operation
    pub fn validate_before_create(&self, entity: &dyn Entity, project_id: &str) -> Result<()> {
        let result = self.validate_entity(entity, project_id, validation::ValidationOperation::Create);
        
        if !result.is_valid {
            let error_messages: Vec<String> = result.errors.iter()
                .map(|e| format!("{}: {}", e.error_code, e.message))
                .collect();
            return Err(anyhow::anyhow!("Validation failed: {}", error_messages.join("; ")));
        }
        
        Ok(())
    }

    /// Validate entity before update operation
    pub fn validate_before_update(&self, entity: &dyn Entity, project_id: &str) -> Result<()> {
        let result = self.validate_entity(entity, project_id, validation::ValidationOperation::Update);
        
        if !result.is_valid {
            let error_messages: Vec<String> = result.errors.iter()
                .map(|e| format!("{}: {}", e.error_code, e.message))
                .collect();
            return Err(anyhow::anyhow!("Validation failed: {}", error_messages.join("; ")));
        }
        
        Ok(())
    }

    /// Get detailed validation errors for an entity
    pub fn get_validation_errors(&self, entity: &dyn Entity, project_id: &str, operation: validation::ValidationOperation) -> Vec<validation::ValidationError> {
        let result = self.validate_entity(entity, project_id, operation);
        result.errors
    }

    /// Validate multiple entities with cross-entity rules
    pub fn validate_entities(&self, entities: &[&dyn Entity], project_id: &str, operation: validation::ValidationOperation) -> Vec<validation::ValidationResult> {
        let context = validation::ValidationContext {
            project_id: project_id.to_string(),
            operation,
            current_user: None,
            metadata: std::collections::HashMap::new(),
        };
        
        self.validator.validate_entities(entities, &context)
    }

    /// Run methodology compliance validation
    pub async fn validate_methodology_compliance(&self, project_id: &str) -> Result<validation::ValidationResult> {
        // Get all entities for the project
        let features = self.list_features().await?;
        let tasks = self.list_tasks().await?;
        let sessions = self.list_sessions().await?;
        
        // Convert to trait objects for validation
        let mut entities: Vec<&dyn Entity> = Vec::new();
        for feature in &features {
            entities.push(feature as &dyn Entity);
        }
        for task in &tasks {
            entities.push(task as &dyn Entity);
        }
        for session in &sessions {
            entities.push(session as &dyn Entity);
        }

        let context = validation::ValidationContext {
            project_id: project_id.to_string(),
            operation: validation::ValidationOperation::BulkOperation,
            current_user: None,
            metadata: std::collections::HashMap::new(),
        };
        
        let results = self.validator.validate_entities(&entities, &context);
        
        // Combine all validation results into a single result
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut is_valid = true;
        
        for result in results {
            if !result.is_valid {
                is_valid = false;
            }
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }
        
        Ok(validation::ValidationResult {
            is_valid,
            errors: all_errors,
            warnings: all_warnings,
            entity_type: "ProjectWide".to_string(),
            entity_id: Some(project_id.to_string()),
        })
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
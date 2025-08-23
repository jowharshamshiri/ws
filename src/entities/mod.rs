// Workspace Entity Management System - Schema-Based Architecture
// Complete replacement following D081: Zero backward compatibility

pub mod database;
pub mod crud;
pub mod schema_models;
pub mod schema_traits;

// Re-export key types for easy access
pub use schema_models::*;
pub use schema_traits::*;

use anyhow::Result;
use sqlx::SqlitePool;

/// Entity Manager - Unified interface for all entity operations
pub struct EntityManager {
    pub pool: SqlitePool,
}

impl EntityManager {
    /// Create new entity manager with database connection
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get database pool reference
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Create a new project
    pub async fn create_project(&self, name: String, description: String) -> Result<Project> {
        crud::projects::create(&self.pool, name, description).await
    }

    /// Get project by ID
    pub async fn get_project(&self, id: &str) -> Result<Option<Project>> {
        crud::projects::get_by_id(&self.pool, id).await
    }

    /// List all active projects
    pub async fn list_active_projects(&self) -> Result<Vec<Project>> {
        crud::projects::list_active(&self.pool).await
    }


    /// Create a new feature (backward compatibility with 2-param signature)
    pub async fn create_feature(
        &self,
        name: String,
        description: String,
    ) -> Result<Feature> {
        // Use first active project if available
        let project = self.get_current_project().await?;
        let project_id = project.map(|p| p.id).unwrap_or_else(|| "P001".to_string());
        crud::features::create(&self.pool, project_id, name, description, None).await
    }

    /// Create a new feature with full parameters
    pub async fn create_feature_full(
        &self,
        project_id: String,
        name: String,
        description: String,
        category: Option<String>,
    ) -> Result<Feature> {
        crud::features::create(&self.pool, project_id, name, description, category).await
    }

    /// Get feature by ID
    pub async fn get_feature(&self, id: &str) -> Result<Option<Feature>> {
        crud::features::get_by_id(&self.pool, id).await
    }

    /// List features by project
    pub async fn list_features_by_project(&self, project_id: &str) -> Result<Vec<Feature>> {
        crud::features::list_by_project(&self.pool, project_id).await
    }

    /// List all features (backward compatibility)
    pub async fn list_features(&self) -> Result<Vec<Feature>> {
        // Get current project and list its features
        let project = self.get_current_project().await?;
        if let Some(project) = project {
            self.list_features_by_project(&project.id).await
        } else {
            Ok(vec![])
        }
    }

    /// Update feature state
    pub async fn update_feature_state(&self, id: &str, new_state: FeatureState) -> Result<()> {
        crud::features::update_state(&self.pool, id, new_state).await
    }

    /// Create a new task (backward compatibility with 2-param signature)
    pub async fn create_task(
        &self,
        title: String,
        _description: String,
    ) -> Result<Task> {
        // Use first active project and feature if available
        let project = self.get_current_project().await?;
        let project_id = project.map(|p| p.id).unwrap_or_else(|| "P001".to_string());
        let features = self.list_features_by_project(&project_id).await?;
        let feature_id = features.first().map(|f| f.id.clone()).unwrap_or_else(|| "F00001".to_string());
        crud::tasks::create(&self.pool, project_id, feature_id, title, "feature".to_string()).await
    }

    /// Create a new task with full parameters
    pub async fn create_task_full(
        &self,
        project_id: String,
        feature_id: String,
        task_description: String,
        category: String,
    ) -> Result<Task> {
        crud::tasks::create(&self.pool, project_id, feature_id, task_description, category).await
    }

    /// Get task by ID
    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        crud::tasks::get_by_id(&self.pool, id).await
    }

    /// List tasks by project and optional status filter
    pub async fn list_tasks_by_project(
        &self,
        project_id: &str,
        status: Option<TaskStatus>,
    ) -> Result<Vec<Task>> {
        crud::tasks::list_by_project(&self.pool, project_id, status).await
    }

    /// List all tasks (backward compatibility)
    pub async fn list_tasks(&self) -> Result<Vec<Task>> {
        // Get current project and list its tasks
        let project = self.get_current_project().await?;
        if let Some(project) = project {
            self.list_tasks_by_project(&project.id, None).await
        } else {
            Ok(vec![])
        }
    }

    /// Update task status
    pub async fn update_task_status(&self, id: &str, new_status: TaskStatus) -> Result<()> {
        crud::tasks::update_status(&self.pool, id, new_status).await
    }

    /// Update task (full object update)
    pub async fn update_task(&self, task: Task) -> Result<()> {
        crud::tasks::update(&self.pool, &task).await
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        project_id: String,
        title: String,
        focus: String,
    ) -> Result<Session> {
        crud::sessions::create(&self.pool, project_id, title, Some(focus)).await
    }

    /// Get session by ID
    pub async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        crud::sessions::get_by_id(&self.pool, id).await
    }

    /// List sessions by project
    pub async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<Session>> {
        // Use CRUD function to list sessions by project
        crud::sessions::list_by_project(&self.pool, project_id).await
    }

    /// Complete a session
    pub async fn complete_session(&self, id: &str, summary: String) -> Result<()> {
        crud::sessions::complete(&self.pool, id, summary).await
    }

    /// Create a new directive
    pub async fn create_directive(
        &self,
        project_id: String,
        title: String,
        rule: String,
        category: DirectiveCategory,
        priority: Priority,
    ) -> Result<Directive> {
        crud::directives::create(&self.pool, project_id, title, rule, category, priority).await
    }

    /// Get directive by ID
    pub async fn get_directive(&self, id: &str) -> Result<Option<Directive>> {
        crud::directives::get_by_id(&self.pool, id).await
    }

    /// List active directives by project
    pub async fn list_active_directives(&self, project_id: &str) -> Result<Vec<Directive>> {
        crud::directives::list_active_by_project(&self.pool, project_id).await
    }

    /// Deactivate a directive
    pub async fn deactivate_directive(&self, id: &str) -> Result<()> {
        crud::directives::deactivate(&self.pool, id).await
    }

    /// Delete a project (CASCADE will handle dependent entities)
    pub async fn delete_project(&self, id: &str) -> Result<()> {
        crud::projects::delete(&self.pool, id).await
    }

    /// Delete a feature (SET NULL will update dependent tests)
    pub async fn delete_feature(&self, id: &str) -> Result<()> {
        crud::features::delete(&self.pool, id).await
    }

    /// Delete a task
    pub async fn delete_task(&self, id: &str) -> Result<()> {
        crud::tasks::delete(&self.pool, id).await
    }

    /// Delete a session (SET NULL will update dependent tasks and audit trails)
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        crud::sessions::delete(&self.pool, id).await
    }

    /// Get current project (placeholder - needs project detection logic)
    pub async fn get_current_project(&self) -> Result<Option<Project>> {
        // For now, get the first active project
        let projects = self.list_active_projects().await?;
        Ok(projects.into_iter().next())
    }

    /// Create note link (placeholder - needs note linking implementation)
    pub async fn create_note_link(
        &self,
        _note_id: String,
        _entity_id: String,
        _entity_type: String,
        _relationship_type: String,
    ) -> Result<String> {
        // TODO: Implement note linking system
        Ok("link_placeholder".to_string())
    }

    /// Remove note link (placeholder)
    pub async fn remove_note_link(&self, _link_id: &str) -> Result<bool> {
        // TODO: Implement note link removal
        Ok(true)
    }

    /// Get bidirectional links (placeholder)
    pub async fn get_bidirectional_links(&self, _id: &str, _entity_type: Option<String>) -> Result<Vec<String>> {
        // TODO: Implement bidirectional link retrieval
        Ok(vec![])
    }

    /// Delete a directive
    pub async fn delete_directive(&self, id: &str) -> Result<()> {
        crud::directives::delete(&self.pool, id).await
    }
}
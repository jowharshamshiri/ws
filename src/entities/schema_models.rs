// Schema-Based Entity Models - Complete Replacement Implementation
// Following Directive D081: Zero backward compatibility, complete replacement

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::any::Any;

use super::schema_traits::{
    Entity, EntityType, ProjectEntity, StatefulEntity, PrioritizedEntity, 
    NotableEntity, TimeTrackableEntity, ValidatedEntity, SearchableEntity,
    ComparableEntity
};

/// Project Status Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Paused => "paused", 
            ProjectStatus::Completed => "completed",
            ProjectStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(ProjectStatus::Active),
            "paused" => Ok(ProjectStatus::Paused),
            "completed" => Ok(ProjectStatus::Completed),
            "archived" => Ok(ProjectStatus::Archived),
            _ => Err(format!("Invalid project status: {}", s)),
        }
    }
}

/// Feature State Enumeration - Core Implementation Tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum FeatureState {
    NotImplemented,          // ❌ - not_implemented
    ImplementedNoTests,      // 🟠 - implemented_no_tests
    ImplementedFailingTests, // 🟡 - implemented_failing_tests
    ImplementedPassingTests, // 🟢 - implemented_passing_tests
    TestsBroken,             // ⚠️ - tests_broken
    CriticalIssue,           // 🔴 - critical_issue
}

impl FeatureState {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeatureState::NotImplemented => "not_implemented",
            FeatureState::ImplementedNoTests => "implemented_no_tests",
            FeatureState::ImplementedFailingTests => "implemented_failing_tests",
            FeatureState::ImplementedPassingTests => "implemented_passing_tests",
            FeatureState::TestsBroken => "tests_broken",
            FeatureState::CriticalIssue => "critical_issue",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "not_implemented" => Ok(FeatureState::NotImplemented),
            "implemented_no_tests" => Ok(FeatureState::ImplementedNoTests),
            "implemented_failing_tests" => Ok(FeatureState::ImplementedFailingTests),
            "implemented_passing_tests" => Ok(FeatureState::ImplementedPassingTests),
            "tests_broken" => Ok(FeatureState::TestsBroken),
            "critical_issue" => Ok(FeatureState::CriticalIssue),
            _ => Err(format!("Invalid feature state: {}", s)),
        }
    }

    /// Validate state transitions according to business rules
    pub fn can_transition_to(&self, new_state: &FeatureState) -> bool {
        use FeatureState::*;
        match (self, new_state) {
            // Normal progression
            (NotImplemented, ImplementedNoTests) => true,
            (ImplementedNoTests, ImplementedFailingTests) => true,
            (ImplementedNoTests, TestsBroken) => true,
            (ImplementedFailingTests, ImplementedPassingTests) => true,
            (ImplementedFailingTests, TestsBroken) => true,
            (ImplementedPassingTests, TestsBroken) => true,
            (TestsBroken, ImplementedPassingTests) => true,
            
            // Critical issue can transition from/to any state
            (_, CriticalIssue) => true,
            (CriticalIssue, _) => true,
            
            // Same state (no change)
            (a, b) if a == b => true,
            
            // Invalid transitions
            _ => false,
        }
    }

    /// Get the emoji representation for display
    pub fn emoji(&self) -> &'static str {
        match self {
            FeatureState::NotImplemented => "❌",
            FeatureState::ImplementedNoTests => "🟠",
            FeatureState::ImplementedFailingTests => "🟡",
            FeatureState::ImplementedPassingTests => "🟢",
            FeatureState::TestsBroken => "⚠️",
            FeatureState::CriticalIssue => "🔴",
        }
    }
}

/// Task Priority Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum TaskPriority {
    High,
    Medium,
    Low,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::High => "high",
            TaskPriority::Medium => "medium",
            TaskPriority::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "high" => Ok(TaskPriority::High),
            "medium" => Ok(TaskPriority::Medium),
            "low" => Ok(TaskPriority::Low),
            _ => Err(format!("Invalid task priority: {}", s)),
        }
    }
}

/// Task Status Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
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

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "in_progress" => Ok(TaskStatus::InProgress),
            "blocked" => Ok(TaskStatus::Blocked),
            "completed" => Ok(TaskStatus::Completed),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(format!("Invalid task status: {}", s)),
        }
    }

    /// Validate status transitions according to business rules
    pub fn can_transition_to(&self, new_status: &TaskStatus) -> bool {
        use TaskStatus::*;
        match (self, new_status) {
            (Pending, InProgress) => true,
            (Pending, Cancelled) => true,
            (InProgress, Completed) => true,
            (InProgress, Blocked) => true,
            (InProgress, Cancelled) => true,
            (Blocked, InProgress) => true,
            (Blocked, Cancelled) => true,
            
            // Same status (no change)
            (a, b) if a == b => true,
            
            // Invalid transitions
            _ => false,
        }
    }
}

/// Session Status Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum SessionStatus {
    Active,
    Completed,
    Cancelled,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Active => "active",
            SessionStatus::Completed => "completed",
            SessionStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(SessionStatus::Active),
            "completed" => Ok(SessionStatus::Completed),
            "cancelled" => Ok(SessionStatus::Cancelled),
            _ => Err(format!("Invalid session status: {}", s)),
        }
    }
}

/// Directive Priority Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum DirectivePriority {
    Critical,
    High,
    Medium,
    Low,
}

impl DirectivePriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            DirectivePriority::Critical => "critical",
            DirectivePriority::High => "high",
            DirectivePriority::Medium => "medium",
            DirectivePriority::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "critical" => Ok(DirectivePriority::Critical),
            "high" => Ok(DirectivePriority::High),
            "medium" => Ok(DirectivePriority::Medium),
            "low" => Ok(DirectivePriority::Low),
            _ => Err(format!("Invalid directive priority: {}", s)),
        }
    }
}

/// Directive Status Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum DirectiveStatus {
    Active,
    Inactive,
    Archived,
}

impl DirectiveStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DirectiveStatus::Active => "active",
            DirectiveStatus::Inactive => "inactive",
            DirectiveStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(DirectiveStatus::Active),
            "inactive" => Ok(DirectiveStatus::Inactive),
            "archived" => Ok(DirectiveStatus::Archived),
            _ => Err(format!("Invalid directive status: {}", s)),
        }
    }
}

/// Task Category Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum TaskCategory {
    Feature,
    Bug,
    Refactor,
    Testing,
    Documentation,
    Infrastructure,
    Api,
    Migration,
}

impl TaskCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskCategory::Feature => "feature",
            TaskCategory::Bug => "bug",
            TaskCategory::Refactor => "refactor",
            TaskCategory::Testing => "testing",
            TaskCategory::Documentation => "documentation",
            TaskCategory::Infrastructure => "infrastructure",
            TaskCategory::Api => "api",
            TaskCategory::Migration => "migration",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "feature" => Ok(TaskCategory::Feature),
            "bug" => Ok(TaskCategory::Bug),
            "refactor" => Ok(TaskCategory::Refactor),
            "testing" => Ok(TaskCategory::Testing),
            "documentation" => Ok(TaskCategory::Documentation),
            "infrastructure" => Ok(TaskCategory::Infrastructure),
            "api" => Ok(TaskCategory::Api),
            "migration" => Ok(TaskCategory::Migration),
            _ => Err(format!("Invalid task category: {}", s)),
        }
    }
}

/// Directive Category Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum DirectiveCategory {
    Development,
    Testing,
    Deployment,
    Security,
    Workflow,
    Quality,
    Architecture,
    Performance,
}

impl DirectiveCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            DirectiveCategory::Development => "development",
            DirectiveCategory::Testing => "testing",
            DirectiveCategory::Deployment => "deployment",
            DirectiveCategory::Security => "security",
            DirectiveCategory::Workflow => "workflow",
            DirectiveCategory::Quality => "quality",
            DirectiveCategory::Architecture => "architecture",
            DirectiveCategory::Performance => "performance",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "development" => Ok(DirectiveCategory::Development),
            "testing" => Ok(DirectiveCategory::Testing),
            "deployment" => Ok(DirectiveCategory::Deployment),
            "security" => Ok(DirectiveCategory::Security),
            "workflow" => Ok(DirectiveCategory::Workflow),
            "quality" => Ok(DirectiveCategory::Quality),
            "architecture" => Ok(DirectiveCategory::Architecture),
            "performance" => Ok(DirectiveCategory::Performance),
            _ => Err(format!("Invalid directive category: {}", s)),
        }
    }
}

/// General Priority Enumeration (used by tasks and directives)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "critical" => Ok(Priority::Critical),
            "high" => Ok(Priority::High),
            "medium" => Ok(Priority::Medium),
            "low" => Ok(Priority::Low),
            _ => Err(format!("Invalid priority: {}", s)),
        }
    }
}

/// ID Pattern Validation
pub struct IdValidator;

impl IdValidator {
    /// Validate Project ID pattern: P### (P001, P002, P003...)
    pub fn validate_project_id(id: &str) -> bool {
        regex::Regex::new(r"^P\d{3}$").unwrap().is_match(id)
    }

    /// Validate Feature ID pattern: F##### (F00001, F00002, F00003...)
    pub fn validate_feature_id(id: &str) -> bool {
        regex::Regex::new(r"^F\d{5}$").unwrap().is_match(id)
    }

    /// Validate Task ID pattern: T###### (T000001, T000002, T000003...)
    pub fn validate_task_id(id: &str) -> bool {
        regex::Regex::new(r"^T\d{6}$").unwrap().is_match(id)
    }

    /// Validate Session ID pattern: S###### (S000001, S000002, S000003...)
    pub fn validate_session_id(id: &str) -> bool {
        regex::Regex::new(r"^S\d{6}$").unwrap().is_match(id)
    }

    /// Validate Directive ID pattern: D### (D001, D002, D003...)
    pub fn validate_directive_id(id: &str) -> bool {
        regex::Regex::new(r"^D\d{3}$").unwrap().is_match(id)
    }

    /// Validate Metric code pattern: M## or P## (M01, M02, P01, P02...)
    pub fn validate_metric_code(code: &str) -> bool {
        let m_pattern = regex::Regex::new(r"^M\d{2}$").unwrap();
        let p_pattern = regex::Regex::new(r"^P\d{2}$").unwrap();
        m_pattern.is_match(code) || p_pattern.is_match(code)
    }
}

/// Project Entity - Root Container for All Project Data
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    /// Project ID in P### format (P001, P002, P003...)
    pub id: String,
    /// Project name
    pub name: String,
    /// Project description
    pub description: String,
    /// Project status (active, paused, completed, archived)
    pub status: String, // Stored as string in DB, converted to enum in application
    /// Optional current development phase
    pub current_phase: Option<String>,
    /// Project creation timestamp
    pub created_at: DateTime<Utc>,
    /// Project last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Create new project with validation
    pub fn new(id: String, name: String, description: String) -> Result<Self, String> {
        if !IdValidator::validate_project_id(&id) {
            return Err(format!("Invalid project ID pattern: {}. Must be P### format (P001, P002, etc.)", id));
        }

        if name.trim().is_empty() {
            return Err("Project name cannot be empty".to_string());
        }

        if description.trim().is_empty() {
            return Err("Project description cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Project {
            id,
            name,
            description,
            status: ProjectStatus::Active.as_str().to_string(),
            current_phase: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create from database row
    pub fn from_db_row(
        id: String,
        name: String,
        description: String,
        status: String,
        current_phase: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        if !IdValidator::validate_project_id(&id) {
            return Err(format!("Invalid project ID pattern: {}", id));
        }

        Ok(Project {
            id,
            name,
            description,
            status,
            current_phase,
            created_at,
            updated_at,
        })
    }

    /// Get project status as enum
    pub fn get_status(&self) -> Result<ProjectStatus, String> {
        ProjectStatus::from_str(&self.status)
    }

    /// Set project status with validation
    pub fn set_status(&mut self, status: ProjectStatus) {
        self.status = status.as_str().to_string();
        self.updated_at = Utc::now();
    }

    /// Update project with validation
    pub fn update(&mut self, name: Option<String>, description: Option<String>, current_phase: Option<String>) -> Result<(), String> {
        if let Some(new_name) = name {
            if new_name.trim().is_empty() {
                return Err("Project name cannot be empty".to_string());
            }
            self.name = new_name;
        }

        if let Some(new_description) = description {
            if new_description.trim().is_empty() {
                return Err("Project description cannot be empty".to_string());
            }
            self.description = new_description;
        }

        if let Some(new_phase) = current_phase {
            self.current_phase = if new_phase.trim().is_empty() { None } else { Some(new_phase) };
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Feature Entity - Central Implementation Tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feature {
    /// Feature ID in F##### format (F00001, F00002, F00003...)
    pub id: String,
    /// Foreign key to Project
    pub project_id: String,
    /// Feature code (matches ID for consistency)
    pub code: String,
    /// Feature name
    pub name: String,
    /// Feature description
    pub description: String,
    /// Feature category (optional)
    pub category: Option<String>,
    /// Feature implementation state (stored as string, converted to enum)
    pub state: String,
    /// Test status (stored as string)
    pub test_status: String,
    /// Priority level (stored as string)
    pub priority: String,
    /// Brief operational notes (max 100 characters)
    pub notes: Option<String>,
    /// Feature creation timestamp
    pub created_at: DateTime<Utc>,
    /// Feature last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Feature {
    /// Create new feature with validation
    pub fn new(id: String, project_id: String, code: String, name: String, description: String, category: Option<String>) -> Result<Self, String> {
        if !IdValidator::validate_feature_id(&id) {
            return Err(format!("Invalid feature ID pattern: {}. Must be F##### format (F00001, F00002, etc.)", id));
        }

        if !IdValidator::validate_project_id(&project_id) {
            return Err(format!("Invalid project ID pattern: {}. Must be P### format", project_id));
        }

        if name.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }

        if description.trim().is_empty() {
            return Err("Feature description cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Feature {
            id,
            project_id,
            code,
            name,
            description,
            category,
            state: FeatureState::NotImplemented.as_str().to_string(),
            test_status: "not_tested".to_string(),
            priority: "medium".to_string(),
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create from database row
    pub fn from_db_row(
        id: String,
        project_id: String,
        code: String,
        name: String,
        description: String,
        category: Option<String>,
        state: String,
        test_status: String,
        priority: String,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        if !IdValidator::validate_feature_id(&id) {
            return Err(format!("Invalid feature ID pattern: {}", id));
        }

        Ok(Feature {
            id,
            project_id,
            code,
            name,
            description,
            category,
            state,
            test_status,
            priority,
            notes,
            created_at,
            updated_at,
        })
    }

    /// Get feature state as enum
    pub fn get_state(&self) -> Result<FeatureState, String> {
        FeatureState::from_str(&self.state)
    }

    /// Set feature state with transition validation
    pub fn set_state(&mut self, new_state: FeatureState) -> Result<(), String> {
        let current_state = self.get_state()?;
        
        if !current_state.can_transition_to(&new_state) {
            return Err(format!(
                "Invalid state transition from {} to {}",
                current_state.as_str(),
                new_state.as_str()
            ));
        }

        self.state = new_state.as_str().to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update feature with validation
    pub fn update(&mut self, name: Option<String>, description: Option<String>, notes: Option<String>) -> Result<(), String> {
        if let Some(new_name) = name {
            if new_name.trim().is_empty() {
                return Err("Feature name cannot be empty".to_string());
            }
            self.name = new_name;
        }

        if let Some(new_description) = description {
            if new_description.trim().is_empty() {
                return Err("Feature description cannot be empty".to_string());
            }
            self.description = new_description;
        }

        if let Some(new_notes) = notes {
            if new_notes.len() > 100 {
                return Err("Notes cannot exceed 100 characters".to_string());
            }
            self.notes = if new_notes.trim().is_empty() { None } else { Some(new_notes) };
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Task Entity - Work Item Management with Feature Integration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    /// Task ID in T###### format (T000001, T000002, T000003...)
    pub id: String,
    /// Foreign key to Project
    pub project_id: String,
    /// Foreign key to Feature
    pub feature_id: String,
    /// Task description
    pub task: String,
    /// Task priority (stored as string, converted to enum)
    pub priority: String,
    /// Task status (stored as string, converted to enum)
    pub status: String,
    /// Task category (feature, bug, refactor, testing, documentation)
    pub category: String,
    /// Task dependencies as JSON array of task IDs
    pub dependencies: Option<String>,
    /// Assigned person or identifier
    pub assigned: Option<String>,
    /// Brief operational notes (max 100 characters)
    pub notes: Option<String>,
    /// Task creation timestamp
    pub created_at: DateTime<Utc>,
    /// Task last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// Create new task with validation
    pub fn new(
        id: String,
        project_id: String,
        feature_id: String,
        task: String,
        category: String,
    ) -> Result<Self, String> {
        if !IdValidator::validate_task_id(&id) {
            return Err(format!("Invalid task ID pattern: {}. Must be T###### format (T000001, T000002, etc.)", id));
        }

        if !IdValidator::validate_project_id(&project_id) {
            return Err(format!("Invalid project ID pattern: {}. Must be P### format", project_id));
        }

        if !IdValidator::validate_feature_id(&feature_id) {
            return Err(format!("Invalid feature ID pattern: {}. Must be F##### format", feature_id));
        }

        if task.trim().is_empty() {
            return Err("Task description cannot be empty".to_string());
        }

        if category.trim().is_empty() {
            return Err("Task category cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Task {
            id,
            project_id,
            feature_id,
            task,
            priority: TaskPriority::Medium.as_str().to_string(),
            status: TaskStatus::Pending.as_str().to_string(),
            category,
            dependencies: None,
            assigned: None,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create from database row
    pub fn from_db_row(
        id: String,
        project_id: String,
        feature_id: String,
        task: String,
        priority: String,
        status: String,
        category: String,
        dependencies: Option<String>,
        assigned: Option<String>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        if !IdValidator::validate_task_id(&id) {
            return Err(format!("Invalid task ID pattern: {}", id));
        }

        Ok(Task {
            id,
            project_id,
            feature_id,
            task,
            priority,
            status,
            category,
            dependencies,
            assigned,
            notes,
            created_at,
            updated_at,
        })
    }

    /// Get task priority as enum
    pub fn get_priority(&self) -> Result<TaskPriority, String> {
        TaskPriority::from_str(&self.priority)
    }

    /// Set task priority
    pub fn set_priority(&mut self, priority: TaskPriority) {
        self.priority = priority.as_str().to_string();
        self.updated_at = Utc::now();
    }

    /// Get task status as enum
    pub fn get_status(&self) -> Result<TaskStatus, String> {
        TaskStatus::from_str(&self.status)
    }

    /// Set task status with transition validation
    pub fn set_status(&mut self, new_status: TaskStatus) -> Result<(), String> {
        let current_status = self.get_status()?;
        
        if !current_status.can_transition_to(&new_status) {
            return Err(format!(
                "Invalid status transition from {} to {}",
                current_status.as_str(),
                new_status.as_str()
            ));
        }

        self.status = new_status.as_str().to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get task dependencies as vector
    pub fn get_dependencies(&self) -> Result<Vec<String>, String> {
        match &self.dependencies {
            Some(deps_json) => {
                serde_json::from_str(deps_json)
                    .map_err(|e| format!("Failed to parse dependencies JSON: {}", e))
            },
            None => Ok(Vec::new()),
        }
    }

    /// Set task dependencies with validation
    pub fn set_dependencies(&mut self, dependencies: Vec<String>) -> Result<(), String> {
        // Validate all dependency IDs
        for dep_id in &dependencies {
            if !IdValidator::validate_task_id(dep_id) {
                return Err(format!("Invalid dependency task ID pattern: {}. Must be T###### format", dep_id));
            }
        }

        // Check for circular dependencies (self-reference)
        if dependencies.contains(&self.id) {
            return Err("Task cannot depend on itself".to_string());
        }

        self.dependencies = if dependencies.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&dependencies)
                .map_err(|e| format!("Failed to serialize dependencies: {}", e))?)
        };
        
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update task with validation
    pub fn update(
        &mut self,
        task: Option<String>,
        category: Option<String>,
        assigned: Option<String>,
        notes: Option<String>,
    ) -> Result<(), String> {
        if let Some(new_task) = task {
            if new_task.trim().is_empty() {
                return Err("Task description cannot be empty".to_string());
            }
            self.task = new_task;
        }

        if let Some(new_category) = category {
            if new_category.trim().is_empty() {
                return Err("Task category cannot be empty".to_string());
            }
            self.category = new_category;
        }

        if let Some(new_assigned) = assigned {
            self.assigned = if new_assigned.trim().is_empty() { None } else { Some(new_assigned) };
        }

        if let Some(new_notes) = notes {
            if new_notes.len() > 100 {
                return Err("Notes cannot exceed 100 characters".to_string());
            }
            self.notes = if new_notes.trim().is_empty() { None } else { Some(new_notes) };
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_validation() {
        assert!(IdValidator::validate_project_id("P001"));
        assert!(IdValidator::validate_project_id("P999"));
        assert!(!IdValidator::validate_project_id("P1"));
        assert!(!IdValidator::validate_project_id("P0001"));
        assert!(!IdValidator::validate_project_id("PR001"));
        assert!(!IdValidator::validate_project_id("p001"));
    }

    #[test]
    fn test_feature_id_validation() {
        assert!(IdValidator::validate_feature_id("F00001"));
        assert!(IdValidator::validate_feature_id("F99999"));
        assert!(!IdValidator::validate_feature_id("F001"));
        assert!(!IdValidator::validate_feature_id("F000001"));
        assert!(!IdValidator::validate_feature_id("FT0001"));
        assert!(!IdValidator::validate_feature_id("f00001"));
    }

    #[test]
    fn test_task_id_validation() {
        assert!(IdValidator::validate_task_id("T000001"));
        assert!(IdValidator::validate_task_id("T999999"));
        assert!(!IdValidator::validate_task_id("T001"));
        assert!(!IdValidator::validate_task_id("T0000001"));
        assert!(!IdValidator::validate_task_id("TA00001"));
        assert!(!IdValidator::validate_task_id("t000001"));
    }

    #[test]
    fn test_feature_state_transitions() {
        use FeatureState::*;
        
        // Valid transitions
        assert!(NotImplemented.can_transition_to(&ImplementedNoTests));
        assert!(ImplementedNoTests.can_transition_to(&ImplementedFailingTests));
        assert!(ImplementedFailingTests.can_transition_to(&ImplementedPassingTests));
        assert!(ImplementedPassingTests.can_transition_to(&TestsBroken));
        assert!(TestsBroken.can_transition_to(&ImplementedPassingTests));
        
        // Critical issue transitions
        assert!(NotImplemented.can_transition_to(&CriticalIssue));
        assert!(CriticalIssue.can_transition_to(&ImplementedPassingTests));
        
        // Invalid transitions
        assert!(!ImplementedPassingTests.can_transition_to(&NotImplemented));
        assert!(!ImplementedFailingTests.can_transition_to(&NotImplemented));
    }

    #[test]
    fn test_task_status_transitions() {
        use TaskStatus::*;
        
        // Valid transitions
        assert!(Pending.can_transition_to(&InProgress));
        assert!(InProgress.can_transition_to(&Completed));
        assert!(InProgress.can_transition_to(&Blocked));
        assert!(Blocked.can_transition_to(&InProgress));
        
        // Cancellation
        assert!(Pending.can_transition_to(&Cancelled));
        assert!(InProgress.can_transition_to(&Cancelled));
        assert!(Blocked.can_transition_to(&Cancelled));
        
        // Invalid transitions
        assert!(!Completed.can_transition_to(&InProgress));
        assert!(!Completed.can_transition_to(&Pending));
        assert!(!Cancelled.can_transition_to(&InProgress));
    }

    #[test]
    fn test_project_creation() {
        let project = Project::new(
            "P001".to_string(),
            "Test Project".to_string(),
            "A test project".to_string(),
        ).unwrap();
        
        assert_eq!(project.id, "P001");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, "A test project");
        assert_eq!(project.get_status().unwrap(), ProjectStatus::Active);
    }

    #[test]
    fn test_feature_creation() {
        let feature = Feature::new(
            "F00001".to_string(),
            "P001".to_string(),
            "F00001".to_string(), // code
            "Test Feature".to_string(),
            "A test feature".to_string(),
            None, // category
        ).unwrap();
        
        assert_eq!(feature.id, "F00001");
        assert_eq!(feature.project_id, "P001");
        assert_eq!(feature.name, "Test Feature");
        assert_eq!(feature.get_state().unwrap(), FeatureState::NotImplemented);
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "T000001".to_string(),
            "P001".to_string(),
            "F00001".to_string(),
            "Test task".to_string(),
            "feature".to_string(),
        ).unwrap();
        
        assert_eq!(task.id, "T000001");
        assert_eq!(task.project_id, "P001");
        assert_eq!(task.feature_id, "F00001");
        assert_eq!(task.task, "Test task");
        assert_eq!(task.get_status().unwrap(), TaskStatus::Pending);
        assert_eq!(task.get_priority().unwrap(), TaskPriority::Medium);
    }

    #[test]
    fn test_task_dependencies() {
        let mut task = Task::new(
            "T000002".to_string(),
            "P001".to_string(),
            "F00001".to_string(),
            "Test task with deps".to_string(),
            "feature".to_string(),
        ).unwrap();
        
        let deps = vec!["T000001".to_string(), "T000003".to_string()];
        task.set_dependencies(deps.clone()).unwrap();
        
        assert_eq!(task.get_dependencies().unwrap(), deps);
    }

    #[test]
    fn test_invalid_self_dependency() {
        let mut task = Task::new(
            "T000001".to_string(),
            "P001".to_string(),
            "F00001".to_string(),
            "Test task".to_string(),
            "feature".to_string(),
        ).unwrap();
        
        let result = task.set_dependencies(vec!["T000001".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot depend on itself"));
    }
}

/// JSON Data Structures for Session Activity Tracking

/// Completed task information within a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletedTask {
    /// Task ID reference (T###### format)
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Evidence of task completion
    pub evidence: Option<String>,
    /// Validation results for the task
    pub validation_results: Option<String>,
}

impl CompletedTask {
    pub fn new(task_id: String, description: String) -> Result<Self, String> {
        if !IdValidator::validate_task_id(&task_id) {
            return Err(format!("Invalid task ID pattern: {}. Must be T###### format", task_id));
        }

        if description.trim().is_empty() {
            return Err("Task description cannot be empty".to_string());
        }

        Ok(CompletedTask {
            task_id,
            description,
            evidence: None,
            validation_results: None,
        })
    }
}

/// Key achievement information within a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Achievement {
    /// Achievement description
    pub achievement: String,
    /// Impact assessment
    pub impact: Option<String>,
    /// Validation evidence
    pub validation: Option<String>,
}

impl Achievement {
    pub fn new(achievement: String) -> Result<Self, String> {
        if achievement.trim().is_empty() {
            return Err("Achievement description cannot be empty".to_string());
        }

        Ok(Achievement {
            achievement,
            impact: None,
            validation: None,
        })
    }
}

/// File modification information within a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileModification {
    /// Path to the modified file
    pub file_path: String,
    /// Lines changed information (e.g., "10 added, 5 removed")
    pub lines_changed: Option<String>,
    /// Description of the change
    pub change: String,
    /// Reason for the change
    pub reason: String,
    /// Validation of the change
    pub validation: Option<String>,
    /// Related feature ID (F##### format)
    pub feature_id: Option<String>,
}

impl FileModification {
    pub fn new(file_path: String, change: String, reason: String) -> Result<Self, String> {
        if file_path.trim().is_empty() {
            return Err("File path cannot be empty".to_string());
        }

        if change.trim().is_empty() {
            return Err("Change description cannot be empty".to_string());
        }

        if reason.trim().is_empty() {
            return Err("Reason cannot be empty".to_string());
        }

        Ok(FileModification {
            file_path,
            lines_changed: None,
            change,
            reason,
            validation: None,
            feature_id: None,
        })
    }

    pub fn set_feature_id(&mut self, feature_id: String) -> Result<(), String> {
        if !IdValidator::validate_feature_id(&feature_id) {
            return Err(format!("Invalid feature ID pattern: {}. Must be F##### format", feature_id));
        }
        self.feature_id = Some(feature_id);
        Ok(())
    }
}

/// Issue resolution information within a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IssueResolution {
    /// Issue description
    pub issue: String,
    /// Root cause analysis
    pub root_cause: Option<String>,
    /// Solution description
    pub solution: String,
    /// Evidence of resolution
    pub evidence: Option<String>,
}

impl IssueResolution {
    pub fn new(issue: String, solution: String) -> Result<Self, String> {
        if issue.trim().is_empty() {
            return Err("Issue description cannot be empty".to_string());
        }

        if solution.trim().is_empty() {
            return Err("Solution description cannot be empty".to_string());
        }

        Ok(IssueResolution {
            issue,
            root_cause: None,
            solution,
            evidence: None,
        })
    }
}

/// Session Entity - Development Activity Tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    /// Session ID in S###### format (S000001, S000002, S000003...)
    pub id: String,
    /// Foreign key to Project
    pub project_id: String,
    /// Session title
    pub title: String,
    /// Session date (YYYY-MM-DD format)
    pub date: String,
    /// Session start time (HH:MM:SS format)
    pub start_time: Option<String>,
    /// Session end time (HH:MM:SS format)  
    pub end_time: Option<String>,
    /// Session status (stored as string, converted to enum)
    pub status: String,
    /// Session focus area
    pub focus: String,
    /// Major achievement description
    pub major_achievement: Option<String>,
    /// JSON array of CompletedTask objects
    pub completed_tasks: Option<String>,
    /// JSON array of Achievement objects
    pub key_achievements: Option<String>,
    /// JSON array of FileModification objects
    pub files_modified: Option<String>,
    /// JSON array of IssueResolution objects
    pub issues_resolved: Option<String>,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Session last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Session {
    /// Create new session with validation
    pub fn new(id: String, project_id: String, title: String, focus: String) -> Result<Self, String> {
        if !IdValidator::validate_session_id(&id) {
            return Err(format!("Invalid session ID pattern: {}. Must be S###### format (S000001, S000002, etc.)", id));
        }

        if !IdValidator::validate_project_id(&project_id) {
            return Err(format!("Invalid project ID pattern: {}. Must be P### format", project_id));
        }

        if title.trim().is_empty() {
            return Err("Session title cannot be empty".to_string());
        }

        if focus.trim().is_empty() {
            return Err("Session focus cannot be empty".to_string());
        }

        let now = Utc::now();
        let today = now.format("%Y-%m-%d").to_string();

        Ok(Session {
            id,
            project_id,
            title,
            date: today,
            start_time: Some(now.format("%H:%M:%S").to_string()),
            end_time: None,
            status: SessionStatus::Active.as_str().to_string(),
            focus,
            major_achievement: None,
            completed_tasks: None,
            key_achievements: None,
            files_modified: None,
            issues_resolved: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create from database row
    pub fn from_db_row(
        id: String,
        project_id: String,
        title: String,
        date: String,
        start_time: Option<String>,
        end_time: Option<String>,
        status: String,
        focus: String,
        major_achievement: Option<String>,
        completed_tasks: Option<String>,
        key_achievements: Option<String>,
        files_modified: Option<String>,
        issues_resolved: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        if !IdValidator::validate_session_id(&id) {
            return Err(format!("Invalid session ID pattern: {}", id));
        }

        Ok(Session {
            id,
            project_id,
            title,
            date,
            start_time,
            end_time,
            status,
            focus,
            major_achievement,
            completed_tasks,
            key_achievements,
            files_modified,
            issues_resolved,
            created_at,
            updated_at,
        })
    }

    /// Get session status as enum
    pub fn get_status(&self) -> Result<SessionStatus, String> {
        SessionStatus::from_str(&self.status)
    }

    /// Set session status
    pub fn set_status(&mut self, status: SessionStatus) {
        self.status = status.as_str().to_string();
        if status == SessionStatus::Completed && self.end_time.is_none() {
            self.end_time = Some(Utc::now().format("%H:%M:%S").to_string());
        }
        self.updated_at = Utc::now();
    }

    /// Get completed tasks as vector
    pub fn get_completed_tasks(&self) -> Result<Vec<CompletedTask>, String> {
        match &self.completed_tasks {
            Some(tasks_json) => {
                serde_json::from_str(tasks_json)
                    .map_err(|e| format!("Failed to parse completed_tasks JSON: {}", e))
            },
            None => Ok(Vec::new()),
        }
    }

    /// Add completed task with validation
    pub fn add_completed_task(&mut self, task: CompletedTask) -> Result<(), String> {
        let mut tasks = self.get_completed_tasks()?;
        
        // Check for duplicates
        if tasks.iter().any(|t| t.task_id == task.task_id) {
            return Err(format!("Task {} already marked as completed in this session", task.task_id));
        }
        
        tasks.push(task);
        self.completed_tasks = Some(serde_json::to_string(&tasks)
            .map_err(|e| format!("Failed to serialize completed_tasks: {}", e))?);
        
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get key achievements as vector
    pub fn get_key_achievements(&self) -> Result<Vec<Achievement>, String> {
        match &self.key_achievements {
            Some(achievements_json) => {
                serde_json::from_str(achievements_json)
                    .map_err(|e| format!("Failed to parse key_achievements JSON: {}", e))
            },
            None => Ok(Vec::new()),
        }
    }

    /// Add key achievement
    pub fn add_key_achievement(&mut self, achievement: Achievement) -> Result<(), String> {
        let mut achievements = self.get_key_achievements()?;
        achievements.push(achievement);
        
        self.key_achievements = Some(serde_json::to_string(&achievements)
            .map_err(|e| format!("Failed to serialize key_achievements: {}", e))?);
        
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get files modified as vector
    pub fn get_files_modified(&self) -> Result<Vec<FileModification>, String> {
        match &self.files_modified {
            Some(files_json) => {
                serde_json::from_str(files_json)
                    .map_err(|e| format!("Failed to parse files_modified JSON: {}", e))
            },
            None => Ok(Vec::new()),
        }
    }

    /// Add file modification
    pub fn add_file_modification(&mut self, file_mod: FileModification) -> Result<(), String> {
        let mut files = self.get_files_modified()?;
        files.push(file_mod);
        
        self.files_modified = Some(serde_json::to_string(&files)
            .map_err(|e| format!("Failed to serialize files_modified: {}", e))?);
        
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get issues resolved as vector
    pub fn get_issues_resolved(&self) -> Result<Vec<IssueResolution>, String> {
        match &self.issues_resolved {
            Some(issues_json) => {
                serde_json::from_str(issues_json)
                    .map_err(|e| format!("Failed to parse issues_resolved JSON: {}", e))
            },
            None => Ok(Vec::new()),
        }
    }

    /// Add issue resolution
    pub fn add_issue_resolution(&mut self, issue: IssueResolution) -> Result<(), String> {
        let mut issues = self.get_issues_resolved()?;
        issues.push(issue);
        
        self.issues_resolved = Some(serde_json::to_string(&issues)
            .map_err(|e| format!("Failed to serialize issues_resolved: {}", e))?);
        
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update session basic information
    pub fn update(&mut self, title: Option<String>, focus: Option<String>, major_achievement: Option<String>) -> Result<(), String> {
        if let Some(new_title) = title {
            if new_title.trim().is_empty() {
                return Err("Session title cannot be empty".to_string());
            }
            self.title = new_title;
        }

        if let Some(new_focus) = focus {
            if new_focus.trim().is_empty() {
                return Err("Session focus cannot be empty".to_string());
            }
            self.focus = new_focus;
        }

        if let Some(new_achievement) = major_achievement {
            self.major_achievement = if new_achievement.trim().is_empty() { None } else { Some(new_achievement) };
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Directive Entity - Development Rules and Constraints  
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Directive {
    /// Directive ID in D### format (D001, D002, D003...)
    pub id: String,
    /// Foreign key to Project
    pub project_id: String,
    /// Directive title
    pub title: String,
    /// The actual rule text
    pub rule: String,
    /// Directive priority (stored as string, converted to enum)
    pub priority: String,
    /// Directive status (stored as string, converted to enum)
    pub status: String,
    /// When rule applies
    pub context: Option<String>,
    /// Why rule exists
    pub rationale: Option<String>,
    /// Optional category for organization
    pub category: Option<String>,
    /// Directive creation timestamp
    pub created_at: DateTime<Utc>,
    /// Directive last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Directive {
    /// Create new directive with validation
    pub fn new(id: String, project_id: String, title: String, rule: String) -> Result<Self, String> {
        if !IdValidator::validate_directive_id(&id) {
            return Err(format!("Invalid directive ID pattern: {}. Must be D### format (D001, D002, etc.)", id));
        }

        if !IdValidator::validate_project_id(&project_id) {
            return Err(format!("Invalid project ID pattern: {}. Must be P### format", project_id));
        }

        if title.trim().is_empty() {
            return Err("Directive title cannot be empty".to_string());
        }

        if rule.trim().is_empty() {
            return Err("Directive rule cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Directive {
            id,
            project_id,
            title,
            rule,
            priority: DirectivePriority::Medium.as_str().to_string(),
            status: DirectiveStatus::Active.as_str().to_string(),
            context: None,
            rationale: None,
            category: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create from database row
    pub fn from_db_row(
        id: String,
        project_id: String,
        title: String,
        rule: String,
        priority: String,
        status: String,
        context: Option<String>,
        rationale: Option<String>,
        category: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        if !IdValidator::validate_directive_id(&id) {
            return Err(format!("Invalid directive ID pattern: {}", id));
        }

        Ok(Directive {
            id,
            project_id,
            title,
            rule,
            priority,
            status,
            context,
            rationale,
            category,
            created_at,
            updated_at,
        })
    }

    /// Get directive priority as enum
    pub fn get_priority(&self) -> Result<DirectivePriority, String> {
        DirectivePriority::from_str(&self.priority)
    }

    /// Set directive priority
    pub fn set_priority(&mut self, priority: DirectivePriority) {
        self.priority = priority.as_str().to_string();
        self.updated_at = Utc::now();
    }

    /// Get directive status as enum
    pub fn get_status(&self) -> Result<DirectiveStatus, String> {
        DirectiveStatus::from_str(&self.status)
    }

    /// Set directive status
    pub fn set_status(&mut self, status: DirectiveStatus) {
        self.status = status.as_str().to_string();
        self.updated_at = Utc::now();
    }

    /// Update directive with validation
    pub fn update(
        &mut self,
        title: Option<String>,
        rule: Option<String>,
        context: Option<String>,
        rationale: Option<String>,
        category: Option<String>,
    ) -> Result<(), String> {
        if let Some(new_title) = title {
            if new_title.trim().is_empty() {
                return Err("Directive title cannot be empty".to_string());
            }
            self.title = new_title;
        }

        if let Some(new_rule) = rule {
            if new_rule.trim().is_empty() {
                return Err("Directive rule cannot be empty".to_string());
            }
            self.rule = new_rule;
        }

        if let Some(new_context) = context {
            self.context = if new_context.trim().is_empty() { None } else { Some(new_context) };
        }

        if let Some(new_rationale) = rationale {
            self.rationale = if new_rationale.trim().is_empty() { None } else { Some(new_rationale) };
        }

        if let Some(new_category) = category {
            self.category = if new_category.trim().is_empty() { None } else { Some(new_category) };
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Additional test cases for Session and Directive entities
#[cfg(test)]
mod session_directive_tests {
    use super::*;

    #[test]
    fn test_session_id_validation() {
        assert!(IdValidator::validate_session_id("S000001"));
        assert!(IdValidator::validate_session_id("S999999"));
        assert!(!IdValidator::validate_session_id("S001"));
        assert!(!IdValidator::validate_session_id("S0000001"));
        assert!(!IdValidator::validate_session_id("SE00001"));
        assert!(!IdValidator::validate_session_id("s000001"));
    }

    #[test]
    fn test_directive_id_validation() {
        assert!(IdValidator::validate_directive_id("D001"));
        assert!(IdValidator::validate_directive_id("D999"));
        assert!(!IdValidator::validate_directive_id("D1"));
        assert!(!IdValidator::validate_directive_id("D0001"));
        assert!(!IdValidator::validate_directive_id("DIR001"));
        assert!(!IdValidator::validate_directive_id("d001"));
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "S000001".to_string(),
            "P001".to_string(),
            "Test Session".to_string(),
            "Testing new features".to_string(),
        ).unwrap();
        
        assert_eq!(session.id, "S000001");
        assert_eq!(session.project_id, "P001");
        assert_eq!(session.title, "Test Session");
        assert_eq!(session.focus, "Testing new features");
        assert_eq!(session.get_status().unwrap(), SessionStatus::Active);
        assert!(session.start_time.is_some());
        assert!(session.end_time.is_none());
    }

    #[test]
    fn test_session_completed_tasks() {
        let mut session = Session::new(
            "S000001".to_string(),
            "P001".to_string(),
            "Test Session".to_string(),
            "Testing".to_string(),
        ).unwrap();

        let task = CompletedTask::new(
            "T000001".to_string(),
            "Test task completed".to_string(),
        ).unwrap();

        session.add_completed_task(task.clone()).unwrap();
        
        let tasks = session.get_completed_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0], task);
    }

    #[test]
    fn test_session_duplicate_task_prevention() {
        let mut session = Session::new(
            "S000001".to_string(),
            "P001".to_string(),
            "Test Session".to_string(),
            "Testing".to_string(),
        ).unwrap();

        let task = CompletedTask::new(
            "T000001".to_string(),
            "Test task".to_string(),
        ).unwrap();

        session.add_completed_task(task.clone()).unwrap();
        
        let result = session.add_completed_task(task);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already marked as completed"));
    }

    #[test]
    fn test_directive_creation() {
        let directive = Directive::new(
            "D001".to_string(),
            "P001".to_string(),
            "Test Directive".to_string(),
            "Always test before committing".to_string(),
        ).unwrap();
        
        assert_eq!(directive.id, "D001");
        assert_eq!(directive.project_id, "P001");
        assert_eq!(directive.title, "Test Directive");
        assert_eq!(directive.rule, "Always test before committing");
        assert_eq!(directive.get_priority().unwrap(), DirectivePriority::Medium);
        assert_eq!(directive.get_status().unwrap(), DirectiveStatus::Active);
    }

    #[test]
    fn test_file_modification_feature_validation() {
        let mut file_mod = FileModification::new(
            "src/main.rs".to_string(),
            "Added new feature".to_string(),
            "Implementing user request".to_string(),
        ).unwrap();

        file_mod.set_feature_id("F00001".to_string()).unwrap();
        assert_eq!(file_mod.feature_id, Some("F00001".to_string()));

        let result = file_mod.set_feature_id("INVALID".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid feature ID pattern"));
    }

    #[test]
    fn test_metric_code_validation() {
        assert!(IdValidator::validate_metric_code("M01"));
        assert!(IdValidator::validate_metric_code("M99"));
        assert!(IdValidator::validate_metric_code("P01"));
        assert!(IdValidator::validate_metric_code("P99"));
        assert!(!IdValidator::validate_metric_code("M1"));
        assert!(!IdValidator::validate_metric_code("M001"));
        assert!(!IdValidator::validate_metric_code("X01"));
        assert!(!IdValidator::validate_metric_code("m01"));
    }
}

// Entity Trait Implementations - Complete replacement with new trait system
// Following Directive D081: Zero backward compatibility

impl Entity for Project {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Project }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { Some(&self.description) }
    fn project_id(&self) -> &str { &self.id } // Project references itself
    fn as_any(&self) -> &dyn Any { self }
}

impl StatefulEntity for Project {
    fn current_status(&self) -> &str { &self.status }
    fn is_active(&self) -> bool { self.status == ProjectStatus::Active.as_str() }
    fn is_completed(&self) -> bool { self.status == ProjectStatus::Completed.as_str() }
}

impl ValidatedEntity for Project {}
impl SearchableEntity for Project {}
impl ComparableEntity for Project {}

impl Entity for Feature {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Feature }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { Some(&self.description) }
    fn project_id(&self) -> &str { &self.project_id }
    fn as_any(&self) -> &dyn Any { self }
}

impl ProjectEntity for Feature {}

impl StatefulEntity for Feature {
    fn current_status(&self) -> &str { &self.state }
    fn is_active(&self) -> bool { 
        matches!(self.get_state().unwrap_or(FeatureState::NotImplemented), 
                FeatureState::ImplementedNoTests | FeatureState::ImplementedFailingTests | FeatureState::ImplementedPassingTests)
    }
    fn is_completed(&self) -> bool { 
        matches!(self.get_state().unwrap_or(FeatureState::NotImplemented), 
                FeatureState::ImplementedPassingTests)
    }
    fn status_display(&self) -> String {
        match self.get_state().unwrap_or(FeatureState::NotImplemented) {
            FeatureState::NotImplemented => format!("{} Not Implemented", FeatureState::NotImplemented.emoji()),
            FeatureState::ImplementedNoTests => format!("{} Implemented (No Tests)", FeatureState::ImplementedNoTests.emoji()),
            FeatureState::ImplementedFailingTests => format!("{} Implemented (Failing Tests)", FeatureState::ImplementedFailingTests.emoji()),
            FeatureState::ImplementedPassingTests => format!("{} Implemented (Passing Tests)", FeatureState::ImplementedPassingTests.emoji()),
            FeatureState::TestsBroken => format!("{} Tests Broken", FeatureState::TestsBroken.emoji()),
            FeatureState::CriticalIssue => format!("{} Critical Issue", FeatureState::CriticalIssue.emoji()),
        }
    }
}

impl NotableEntity for Feature {
    fn notes(&self) -> Option<&str> { self.notes.as_deref() }
}

impl ValidatedEntity for Feature {}
impl SearchableEntity for Feature {}
impl ComparableEntity for Feature {}

impl Entity for Task {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Task }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.task }
    fn description(&self) -> Option<&str> { None } // Tasks don't have separate description
    fn project_id(&self) -> &str { &self.project_id }
    fn as_any(&self) -> &dyn Any { self }
}

impl ProjectEntity for Task {}

impl StatefulEntity for Task {
    fn current_status(&self) -> &str { &self.status }
    fn is_active(&self) -> bool { 
        matches!(self.get_status().unwrap_or(TaskStatus::Pending), 
                TaskStatus::InProgress)
    }
    fn is_completed(&self) -> bool { 
        matches!(self.get_status().unwrap_or(TaskStatus::Pending), 
                TaskStatus::Completed)
    }
}

impl PrioritizedEntity for Task {
    fn priority_level(&self) -> &str { &self.priority }
}

impl NotableEntity for Task {
    fn notes(&self) -> Option<&str> { self.notes.as_deref() }
}

impl ValidatedEntity for Task {}
impl SearchableEntity for Task {}
impl ComparableEntity for Task {}

impl Entity for Session {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Session }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { Some(&self.focus) }
    fn project_id(&self) -> &str { &self.project_id }
    fn as_any(&self) -> &dyn Any { self }
}

impl ProjectEntity for Session {}

impl StatefulEntity for Session {
    fn current_status(&self) -> &str { &self.status }
    fn is_active(&self) -> bool { 
        matches!(self.get_status().unwrap_or(SessionStatus::Active), 
                SessionStatus::Active)
    }
    fn is_completed(&self) -> bool { 
        matches!(self.get_status().unwrap_or(SessionStatus::Active), 
                SessionStatus::Completed)
    }
}

impl TimeTrackableEntity for Session {
    fn start_time(&self) -> Option<DateTime<Utc>> {
        self.start_time.as_ref().and_then(|time_str| {
            // Parse time string in format HH:MM:SS
            // Combine with date to create DateTime
            let date_time_str = format!("{}T{}Z", self.date, time_str);
            DateTime::parse_from_rfc3339(&date_time_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        })
    }
    
    fn end_time(&self) -> Option<DateTime<Utc>> {
        self.end_time.as_ref().and_then(|time_str| {
            let date_time_str = format!("{}T{}Z", self.date, time_str);
            DateTime::parse_from_rfc3339(&date_time_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        })
    }
}

impl ValidatedEntity for Session {}
impl SearchableEntity for Session {}
impl ComparableEntity for Session {}

impl Entity for Directive {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Directive }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { Some(&self.rule) }
    fn project_id(&self) -> &str { &self.project_id }
    fn as_any(&self) -> &dyn Any { self }
}

impl ProjectEntity for Directive {}

impl StatefulEntity for Directive {
    fn current_status(&self) -> &str { &self.status }
    fn is_active(&self) -> bool { 
        matches!(self.get_status().unwrap_or(DirectiveStatus::Active), 
                DirectiveStatus::Active)
    }
    fn is_completed(&self) -> bool { false } // Directives are not "completed"
}

impl PrioritizedEntity for Directive {
    fn priority_level(&self) -> &str { &self.priority }
}

impl ValidatedEntity for Directive {}
impl SearchableEntity for Directive {
    fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        // Search in ID, title, rule, context, rationale, category
        self.id().to_lowercase().contains(&query_lower)
            || self.title().to_lowercase().contains(&query_lower)
            || self.rule.to_lowercase().contains(&query_lower)
            || self.context.as_ref().map_or(false, |c| c.to_lowercase().contains(&query_lower))
            || self.rationale.as_ref().map_or(false, |r| r.to_lowercase().contains(&query_lower))
            || self.category.as_ref().map_or(false, |cat| cat.to_lowercase().contains(&query_lower))
    }
    
    fn searchable_content(&self) -> String {
        let mut content = vec![self.id(), self.title(), &self.rule];
        
        if let Some(context) = &self.context {
            content.push(context);
        }
        if let Some(rationale) = &self.rationale {
            content.push(rationale);
        }
        if let Some(category) = &self.category {
            content.push(category);
        }
        
        content.join(" ")
    }
    
    fn search_keywords(&self) -> Vec<String> {
        let mut keywords = vec![
            self.entity_type().as_str().to_string(),
            self.id().to_string(),
            self.priority.clone(),
            self.status.clone(),
        ];
        
        if let Some(category) = &self.category {
            keywords.push(category.clone());
        }
        
        keywords
    }
}
impl ComparableEntity for Directive {}

// Additional trait implementations for JSON data structures

impl Entity for CompletedTask {
    fn id(&self) -> &str { &self.task_id }
    fn entity_type(&self) -> EntityType { EntityType::Task }
    fn created_at(&self) -> DateTime<Utc> { Utc::now() } // Not tracked for JSON structs
    fn updated_at(&self) -> DateTime<Utc> { Utc::now() }
    fn title(&self) -> &str { &self.description }
    fn project_id(&self) -> &str { "" } // Not directly tracked
    fn as_any(&self) -> &dyn Any { self }
}

impl SearchableEntity for CompletedTask {}

// Entity utility functions and helpers

/// Entity utilities for working with multiple entity types
pub struct EntityUtils;

impl EntityUtils {
    /// Check if ID matches any valid entity pattern
    pub fn validate_any_entity_id(id: &str) -> bool {
        IdValidator::validate_project_id(id)
            || IdValidator::validate_feature_id(id)
            || IdValidator::validate_task_id(id)
            || IdValidator::validate_session_id(id)
            || IdValidator::validate_directive_id(id)
    }
    
    /// Determine entity type from ID pattern
    pub fn determine_entity_type_from_id(id: &str) -> Option<EntityType> {
        if IdValidator::validate_project_id(id) {
            Some(EntityType::Project)
        } else if IdValidator::validate_feature_id(id) {
            Some(EntityType::Feature)
        } else if IdValidator::validate_task_id(id) {
            Some(EntityType::Task)
        } else if IdValidator::validate_session_id(id) {
            Some(EntityType::Session)
        } else if IdValidator::validate_directive_id(id) {
            Some(EntityType::Directive)
        } else {
            None
        }
    }
    
    /// Get entity type prefix (P, F, T, S, D)
    pub fn get_entity_prefix(entity_type: EntityType) -> &'static str {
        match entity_type {
            EntityType::Project => "P",
            EntityType::Feature => "F",
            EntityType::Task => "T",
            EntityType::Session => "S",
            EntityType::Directive => "D",
        }
    }
    
    /// Generate next ID for entity type (requires current max ID)
    pub fn generate_next_id(entity_type: EntityType, current_max: u32) -> String {
        match entity_type {
            EntityType::Project => format!("P{:03}", current_max + 1),
            EntityType::Feature => format!("F{:05}", current_max + 1),
            EntityType::Task => format!("T{:06}", current_max + 1),
            EntityType::Session => format!("S{:06}", current_max + 1),
            EntityType::Directive => format!("D{:03}", current_max + 1),
        }
    }
    
    /// Parse numeric part from entity ID
    pub fn parse_id_number(id: &str) -> Option<u32> {
        if id.len() < 2 {
            return None;
        }
        
        let number_part = &id[1..]; // Skip first character (prefix)
        number_part.parse().ok()
    }
}

#[cfg(test)]
mod entity_trait_tests {
    use super::*;

    #[test]
    fn test_project_entity_traits() {
        let project = Project::new(
            "P001".to_string(),
            "Test Project".to_string(),
            "A test project".to_string(),
        ).unwrap();

        // Test Entity trait
        assert_eq!(project.id(), "P001");
        assert_eq!(project.entity_type(), EntityType::Project);
        assert_eq!(project.title(), "Test Project");
        assert_eq!(project.description(), Some("A test project"));
        assert_eq!(project.project_id(), "P001"); // Self-reference
        assert!(project.validate().is_ok());

        // Test StatefulEntity trait
        assert_eq!(project.current_status(), "active");
        assert!(project.is_active());
        assert!(!project.is_completed());

        // Test SearchableEntity trait
        assert!(project.matches_search("Test"));
        assert!(project.matches_search("project"));
        assert!(project.matches_search("P001"));
        assert!(!project.matches_search("nonexistent"));
    }

    #[test]
    fn test_feature_entity_traits() {
        let feature = Feature::new(
            "F00001".to_string(),
            "P001".to_string(),
            "F00001".to_string(), // code
            "Test Feature".to_string(),
            "A test feature".to_string(),
            None, // category
        ).unwrap();

        // Test Entity trait
        assert_eq!(feature.id(), "F00001");
        assert_eq!(feature.entity_type(), EntityType::Feature);
        assert_eq!(feature.title(), "Test Feature");
        assert_eq!(feature.project_id(), "P001");

        // Test ProjectEntity trait
        assert!(feature.belongs_to_project("P001"));
        assert!(!feature.belongs_to_project("P002"));

        // Test StatefulEntity trait
        assert_eq!(feature.current_status(), "not_implemented");
        assert!(!feature.is_active());
        assert!(!feature.is_completed());
        assert!(feature.status_display().contains("Not Implemented"));

        // Test NotableEntity trait
        assert!(!feature.has_notes());
    }

    #[test]
    fn test_task_entity_traits() {
        let task = Task::new(
            "T000001".to_string(),
            "P001".to_string(),
            "F00001".to_string(),
            "Test task".to_string(),
            "feature".to_string(),
        ).unwrap();

        // Test Entity trait
        assert_eq!(task.id(), "T000001");
        assert_eq!(task.entity_type(), EntityType::Task);
        assert_eq!(task.title(), "Test task");
        assert_eq!(task.project_id(), "P001");

        // Test PrioritizedEntity trait
        assert_eq!(task.priority_level(), "medium");
        assert!(!task.is_high_priority());
        assert_eq!(task.priority_sort_order(), 2);

        // Test StatefulEntity trait
        assert_eq!(task.current_status(), "pending");
        assert!(!task.is_active());
        assert!(!task.is_completed());
    }

    #[test]
    fn test_directive_entity_search() {
        let directive = Directive::new(
            "D001".to_string(),
            "P001".to_string(),
            "Testing Rule".to_string(),
            "Always test before committing code".to_string(),
        ).unwrap();

        // Test enhanced search functionality
        assert!(directive.matches_search("Testing"));
        assert!(directive.matches_search("test"));
        assert!(directive.matches_search("committing"));
        assert!(directive.matches_search("D001"));
        
        let content = directive.searchable_content();
        assert!(content.contains("Testing Rule"));
        assert!(content.contains("Always test"));
        
        let keywords = directive.search_keywords();
        assert!(keywords.contains(&"directive".to_string()));
        assert!(keywords.contains(&"D001".to_string()));
        assert!(keywords.contains(&"medium".to_string()));
    }

    #[test]
    fn test_entity_utils() {
        assert!(EntityUtils::validate_any_entity_id("P001"));
        assert!(EntityUtils::validate_any_entity_id("F00001"));
        assert!(EntityUtils::validate_any_entity_id("T000001"));
        assert!(!EntityUtils::validate_any_entity_id("INVALID"));

        assert_eq!(
            EntityUtils::determine_entity_type_from_id("P001"),
            Some(EntityType::Project)
        );
        assert_eq!(
            EntityUtils::determine_entity_type_from_id("F00001"),
            Some(EntityType::Feature)
        );
        assert_eq!(
            EntityUtils::determine_entity_type_from_id("INVALID"),
            None
        );

        assert_eq!(EntityUtils::get_entity_prefix(EntityType::Project), "P");
        assert_eq!(EntityUtils::get_entity_prefix(EntityType::Feature), "F");

        assert_eq!(EntityUtils::generate_next_id(EntityType::Project, 5), "P006");
        assert_eq!(EntityUtils::generate_next_id(EntityType::Feature, 10), "F00011");

        assert_eq!(EntityUtils::parse_id_number("P001"), Some(1));
        assert_eq!(EntityUtils::parse_id_number("F00005"), Some(5));
        assert_eq!(EntityUtils::parse_id_number("INVALID"), None);
    }
}
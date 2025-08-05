// Entity Models for Workspace Project Management System

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// All entities use string IDs with prefix + counter pattern
// No UUIDs in the system


pub use super::{
    DirectiveCategory, Entity, EntityType, FeatureState, NoteType, Priority, SessionState,
    TaskStatus,
};

/// Project entity - root container for all workspace data
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
    pub metadata: Option<String>, // JSON metadata
}

impl Entity for Project {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Project }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Feature entity - central tracking for all project capabilities
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feature {
    pub id: String,
    pub project_id: String,
    pub code: String,           // F0001, F0002, etc.
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub state: FeatureState,
    pub test_status: String,    // detailed test information
    pub priority: Priority,
    pub implementation_notes: Option<String>,
    pub test_evidence: Option<String>,  // links to test results
    pub dependencies: Option<String>,   // JSON array of feature IDs
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_effort: Option<i32>,  // hours
    pub actual_effort: Option<i32>,     // hours
    pub metadata: Option<String>,       // JSON metadata
}

impl Entity for Feature {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Feature }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { Some(&self.description) }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Task entity - work items with feature integration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub code: String,           // TASK-001, TASK-002, etc.
    pub title: String,
    pub description: String,
    pub category: String,       // feature, bug, refactor, etc.
    pub status: TaskStatus,
    pub priority: Priority,
    pub feature_ids: Option<String>,  // JSON array of linked feature IDs
    pub depends_on: Option<String>,   // JSON array of task dependencies
    pub acceptance_criteria: Option<String>, // JSON array
    pub validation_steps: Option<String>,    // JSON array
    pub evidence: Option<String>,            // validation evidence
    pub session_id: Option<String>,         // current/last session working on this
    pub assigned_to: Option<String>,        // assignee if any
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_effort: Option<i32>,       // hours
    pub actual_effort: Option<i32>,          // hours
    pub tags: Option<String>,                // JSON array of tags
    pub metadata: Option<String>,            // JSON metadata
}

impl Entity for Task {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Task }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { Some(&self.description) }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Session entity - development sessions with comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub state: SessionState,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
    pub achievements: Option<String>,        // JSON array of accomplishments
    pub files_modified: Option<String>,      // JSON array of file changes
    pub features_worked: Option<String>,     // JSON array of feature IDs
    pub tasks_completed: Option<String>,     // JSON array of task IDs
    pub next_priority: Option<String>,
    pub reminder: Option<String>,
    pub validation_evidence: Option<String>,
    pub context_remaining: Option<f64>,      // percentage of context remaining
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,          // JSON metadata
}

impl Entity for Session {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Session }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Directive entity - persistent rules that override AI behavior
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Directive {
    pub id: String,
    pub project_id: String,
    pub code: String,           // DEV-001, ARCH-001, etc.
    pub title: String,
    pub rule: String,           // the actual rule text
    pub category: DirectiveCategory,
    pub priority: Priority,
    pub context: Option<String>,        // when rule applies
    pub rationale: Option<String>,      // why rule exists
    pub examples: Option<String>,       // JSON array of examples
    pub violations: Option<String>,     // what happens if broken
    pub override_behavior: Option<String>, // what AI behavior this changes
    pub active: bool,
    pub compliance_checked: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

impl Entity for Directive {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Directive }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { Some(&self.rule) }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Template entity - Tera templates for project automation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,                // Tera template content
    pub output_path: Option<String>,    // where to render template
    pub enabled: bool,
    pub variables: Option<String>,      // JSON object of template variables
    pub last_rendered: Option<DateTime<Utc>>,
    pub render_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,          // JSON metadata
}

impl Entity for Template {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Template }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Test entity - test results linked to features
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Test {
    pub id: String,
    pub project_id: String,
    pub feature_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub test_type: String,              // unit, integration, e2e, etc.
    pub file_path: String,
    pub function_name: Option<String>,
    pub passed: bool,
    pub output: Option<String>,         // test output
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub is_tautological: bool,          // detected as fake test
    pub coverage_percentage: Option<f64>,
    pub run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,          // JSON metadata
}

impl Entity for Test {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Test }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Dependency entity - relationships between entities
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dependency {
    pub id: String,
    pub project_id: String,
    pub from_entity_id: String,
    pub from_entity_type: EntityType,
    pub to_entity_id: String,
    pub to_entity_type: EntityType,
    pub dependency_type: String,        // blocking, soft, resource, sequential
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Entity for Dependency {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Dependency }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.created_at } // No separate updated_at for dependencies
    fn title(&self) -> &str { "Dependency" }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Milestone entity - project milestones with feature linkage
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Milestone {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub target_date: Option<DateTime<Utc>>,
    pub achieved_date: Option<DateTime<Utc>>,
    pub status: String,                     // planned, in_progress, achieved, missed
    pub feature_ids: Option<String>,        // JSON array of linked feature IDs
    pub success_criteria: Option<String>,   // JSON array of criteria
    pub completion_percentage: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,          // JSON metadata
}

impl Entity for Milestone {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Milestone }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { Some(&self.description) }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Note entity - attachable to any other entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: String,
    pub project_id: String,
    pub entity_id: Option<String>,     // if attached to specific entity
    pub entity_type: Option<EntityType>, // type of entity this is attached to
    pub note_type: NoteType,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,           // JSON array of tags
    pub author: Option<String>,         // who created the note
    pub is_project_wide: bool,          // project-wide architecture notes
    pub is_pinned: bool,                // pinned for importance
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,          // JSON metadata
}

impl Entity for Note {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::Note }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn title(&self) -> &str { &self.title }
    fn description(&self) -> Option<&str> { Some(&self.content) }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Entity reference for linking notes and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReference {
    pub id: String,
    pub entity_type: EntityType,
    pub title: String,
    pub description: Option<String>,
}

/// Feature state transition with validation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FeatureStateTransition {
    pub feature_id: String,
    pub from_state: FeatureState,
    pub to_state: FeatureState,
    pub evidence: Option<String>,
    pub notes: Option<String>,
    pub triggered_by: String,           // what caused the transition
    pub timestamp: DateTime<Utc>,
}

/// Task creation with automatic feature detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreationRequest {
    pub title: String,
    pub description: String,
    pub category: String,
    pub priority: Priority,
    pub feature_detection: bool,        // whether to detect new features
    pub suggested_features: Vec<String>, // auto-detected feature suggestions
}

/// Feature creation suggestion from task analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSuggestion {
    pub suggested_code: String,         // F0090, etc.
    pub name: String,
    pub description: String,
    pub category: String,
    pub confidence: f64,                // confidence in suggestion
    pub reasoning: String,              // why this was suggested
    pub task_id: String,                // originating task
}

/// Comprehensive entity query filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityQueryFilter {
    pub entity_types: Option<Vec<EntityType>>,
    pub project_id: Option<String>,
    pub search_text: Option<String>,
    pub tags: Option<Vec<String>>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub states: Option<Vec<String>>,    // feature states, task statuses, etc.
    pub priorities: Option<Vec<Priority>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Universal audit trail for all entity state changes - F0131 Entity State Tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityAuditTrail {
    pub id: String,                     // audit-{counter}
    pub entity_id: String,              // ID of entity that changed
    pub entity_type: EntityType,        // Type of entity
    pub project_id: String,             // Project context
    pub operation_type: String,         // create, update, delete, state_change
    pub field_changed: Option<String>,  // Specific field that changed
    pub old_value: Option<String>,      // Previous value (JSON)
    pub new_value: Option<String>,      // New value (JSON)
    pub change_reason: Option<String>,  // Why the change was made
    pub triggered_by: String,           // Who/what caused the change
    pub session_id: Option<String>,     // Session context
    pub timestamp: DateTime<Utc>,       // When change occurred
    pub metadata: Option<String>,       // Additional context (JSON)
}

impl Entity for EntityAuditTrail {
    fn id(&self) -> &str { &self.id }
    fn entity_type(&self) -> EntityType { EntityType::AuditTrail }
    fn created_at(&self) -> DateTime<Utc> { self.timestamp }
    fn updated_at(&self) -> DateTime<Utc> { self.timestamp }
    fn title(&self) -> &str { 
        "Entity Audit Record"
    }
    fn description(&self) -> Option<&str> { self.change_reason.as_deref() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Audit trail query filters for F0131
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailQuery {
    pub entity_id: Option<String>,
    pub entity_type: Option<EntityType>,
    pub project_id: Option<String>,
    pub operation_type: Option<String>,
    pub triggered_by: Option<String>,
    pub field_changed: Option<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
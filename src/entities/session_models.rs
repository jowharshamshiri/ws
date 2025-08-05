// Enhanced Session and Operational Models for Comprehensive Project Management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// No UUID usage - all IDs are strings
use super::models::{Entity, EntityType};

/// Session types based on historical patterns
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "session_type", rename_all = "snake_case")]
pub enum SessionType {
    FeatureImplementation,    // Implementing new features
    BugFix,                  // Fixing issues and bugs
    TestingValidation,       // Testing and validation work
    Refactoring,             // Code refactoring and cleanup
    DocumentationUpdate,     // Documentation work
    ArchitecturalDesign,     // Design and architecture work
    Integration,             // Integration and deployment
    MaintenanceCleanup,      // General maintenance
}

/// Session state management
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "session_state", rename_all = "snake_case")]
pub enum SessionState {
    Active,
    Completed,
    Interrupted,
    Archived,
}

/// File modification types
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "modification_type", rename_all = "snake_case")]
pub enum ModificationType {
    Created,
    Modified,
    Deleted,
    Renamed,
    Moved,
}

/// Evidence types for validation
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "evidence_type", rename_all = "snake_case")]
pub enum EvidenceType {
    TestPass,
    TestFail,
    CompilationSuccess,
    CompilationError,
    ManualVerification,
    AutomatedValidation,
    BenchmarkResult,
    CodeReview,
}

/// Message types for conversation history
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "message_type", rename_all = "snake_case")]
pub enum MessageType {
    UserInput,
    ClaudeResponse,
    SystemMessage,
    ToolResult,
    ErrorMessage,
}

impl MessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::UserInput => "user_input",
            MessageType::ClaudeResponse => "claude_response",
            MessageType::SystemMessage => "system_message",
            MessageType::ToolResult => "tool_result",
            MessageType::ErrorMessage => "error_message",
        }
    }
}

/// Enhanced Session entity with comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub session_type: SessionType,
    pub title: String,
    pub description: String,
    pub state: SessionState,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub focus_areas: Option<String>,        // JSON array of areas worked on
    pub accomplishments: Option<String>,    // JSON array of achievements
    pub blockers_encountered: Option<String>, // JSON array of issues
    pub next_session_priorities: Option<String>, // JSON array of next steps
    pub commit_id: Option<String>,          // Git commit ID for this session
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
    fn description(&self) -> Option<&str> { Some(&self.description) }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// File modification tracking for precise change attribution
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileModification {
    pub id: String,
    pub session_id: String,
    pub file_path: String,
    pub modification_type: ModificationType,
    pub purpose: String,
    pub lines_added: Option<i32>,
    pub lines_removed: Option<i32>,
    pub lines_modified: Option<i32>,
    pub line_ranges: Option<String>,        // JSON array of line ranges
    pub diff_content: Option<String>,       // Git-style diff if available
    pub timestamp: DateTime<Utc>,
}

/// Evidence tracking for feature validation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Evidence {
    pub id: String,
    pub entity_type: String,                // feature, task, session
    pub entity_id: String,
    pub evidence_type: EvidenceType,
    pub title: String,
    pub description: String,
    pub file_references: Option<String>,    // JSON array of file paths
    pub test_results: Option<String>,       // Test execution results
    pub validation_command: Option<String>, // Command used for validation
    pub output_log: Option<String>,         // Command output
    pub created_at: DateTime<Utc>,
}

/// Issue resolution tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IssueResolution {
    pub id: String,
    pub session_id: String,
    pub issue_title: String,
    pub issue_description: String,
    pub root_cause: String,
    pub solution_description: String,
    pub solution_steps: Option<String>,     // JSON array of steps taken
    pub files_modified: Option<String>,     // JSON array of files changed
    pub validation_method: String,
    pub validation_evidence: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Conversation message storage
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationMessage {
    pub id: String,
    pub session_id: String,
    pub message_type: MessageType,
    pub content: String,
    pub metadata: Option<String>,           // JSON metadata (tool calls, context, etc.)
    pub timestamp: DateTime<Utc>,
    pub sequence_number: i32,               // Order within session
}

/// Session metrics and analytics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SessionMetrics {
    pub id: String,
    pub session_id: String,
    pub features_created: i32,
    pub features_completed: i32,
    pub tasks_created: i32,
    pub tasks_completed: i32,
    pub files_modified: i32,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub tests_written: i32,
    pub tests_passing: i32,
    pub compilation_errors_fixed: i32,
    pub issues_resolved: i32,
    pub created_at: DateTime<Utc>,
}

/// Feature change tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FeatureChange {
    pub id: String,
    pub session_id: String,
    pub feature_id: String,
    pub change_type: String,                // created, state_change, updated, completed
    pub previous_state: Option<String>,     // Previous feature state
    pub new_state: String,                  // New feature state
    pub reason: String,                     // Why the change was made
    pub evidence_id: Option<String>,    // Link to evidence
    pub timestamp: DateTime<Utc>,
}

/// Task change tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskChange {
    pub id: String,
    pub session_id: String,
    pub task_id: String,
    pub change_type: String,                // created, status_change, updated, completed
    pub previous_status: Option<String>,    // Previous task status
    pub new_status: String,                 // New task status
    pub reason: String,                     // Why the change was made
    pub evidence_id: Option<String>,    // Link to evidence
    pub timestamp: DateTime<Utc>,
}

/// Project snapshot for historical tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectSnapshot {
    pub id: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub snapshot_type: String,              // session_start, session_end, milestone, backup
    pub title: String,
    pub total_features: i32,
    pub implemented_features: i32,
    pub tested_features: i32,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub code_metrics: Option<String>,       // JSON with LOC, test coverage, etc.
    pub created_at: DateTime<Utc>,
}

/// Development patterns and insights
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevelopmentPattern {
    pub id: String,
    pub project_id: String,
    pub pattern_type: String,               // common_operation, error_pattern, success_pattern
    pub title: String,
    pub description: String,
    pub frequency: i32,                     // How often this pattern occurs
    pub success_rate: Option<f32>,          // Success rate for this pattern
    pub related_features: Option<String>,   // JSON array of feature types
    pub recommended_approach: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API operation tracking for methodology enforcement
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiOperation {
    pub id: String,
    pub session_id: String,
    pub operation_type: String,             // feature_create, task_update, etc.
    pub endpoint: String,                   // API endpoint called
    pub request_data: Option<String>,       // JSON request payload
    pub response_data: Option<String>,      // JSON response
    pub success: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub timestamp: DateTime<Utc>,
}
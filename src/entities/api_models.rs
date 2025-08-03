// API Request/Response Models for Comprehensive Project Management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::models::{FeatureState, TaskStatus, Priority};
use super::session_models::{SessionType, SessionState, ModificationType, EvidenceType, MessageType};

// ============================================================================
// Feature Management API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeatureRequest {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub priority: Priority,
    pub estimated_effort: Option<i32>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFeatureRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<FeatureState>,
    pub test_status: Option<String>,
    pub implementation_notes: Option<String>,
    pub test_evidence: Option<String>,
    pub actual_effort: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub state: FeatureState,
    pub test_status: String,
    pub priority: Priority,
    pub implementation_notes: Option<String>,
    pub test_evidence: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub linked_tasks: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_effort: Option<i32>,
    pub actual_effort: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureListQuery {
    pub state: Option<FeatureState>,
    pub category: Option<String>,
    pub priority: Option<Priority>,
    pub has_tests: Option<bool>,
    pub created_since: Option<DateTime<Utc>>,
    pub updated_since: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// ============================================================================
// Task Management API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub category: String,
    pub priority: Priority,
    pub feature_ids: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub estimated_effort: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub validation_steps: Option<Vec<String>>,
    pub evidence: Option<String>,
    pub actual_effort: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub code: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub feature_ids: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub validation_steps: Option<Vec<String>>,
    pub evidence: Option<String>,
    pub session_id: Option<String>,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_effort: Option<i32>,
    pub actual_effort: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskListQuery {
    pub status: Option<TaskStatus>,
    pub category: Option<String>,
    pub priority: Option<Priority>,
    pub feature_id: Option<String>,
    pub assigned_to: Option<String>,
    pub created_since: Option<DateTime<Utc>>,
    pub updated_since: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// ============================================================================
// Session Management API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub session_type: SessionType,
    pub title: String,
    pub description: String,
    pub focus_areas: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<SessionState>,
    pub accomplishments: Option<Vec<String>>,
    pub blockers_encountered: Option<Vec<String>>,
    pub next_session_priorities: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub session_type: SessionType,
    pub title: String,
    pub description: String,
    pub state: SessionState,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub focus_areas: Option<Vec<String>>,
    pub accomplishments: Option<Vec<String>>,
    pub blockers_encountered: Option<Vec<String>>,
    pub next_session_priorities: Option<Vec<String>>,
    pub file_modifications: Vec<FileModificationResponse>,
    pub features_modified: Vec<FeatureChangeResponse>,
    pub tasks_completed: Vec<String>,
    pub metrics: Option<SessionMetricsResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// File Modification API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordFileModificationRequest {
    pub file_path: String,
    pub modification_type: ModificationType,
    pub purpose: String,
    pub lines_added: Option<i32>,
    pub lines_removed: Option<i32>,
    pub lines_modified: Option<i32>,
    pub line_ranges: Option<Vec<String>>,
    pub diff_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileModificationResponse {
    pub id: String,
    pub session_id: String,
    pub file_path: String,
    pub modification_type: ModificationType,
    pub purpose: String,
    pub lines_added: Option<i32>,
    pub lines_removed: Option<i32>,
    pub lines_modified: Option<i32>,
    pub line_ranges: Option<Vec<String>>,
    pub diff_content: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Evidence and Validation API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEvidenceRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub evidence_type: EvidenceType,
    pub title: String,
    pub description: String,
    pub file_references: Option<Vec<String>>,
    pub test_results: Option<String>,
    pub validation_command: Option<String>,
    pub output_log: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceResponse {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub evidence_type: EvidenceType,
    pub title: String,
    pub description: String,
    pub file_references: Option<Vec<String>>,
    pub test_results: Option<String>,
    pub validation_command: Option<String>,
    pub output_log: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Issue Resolution API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordIssueResolutionRequest {
    pub issue_title: String,
    pub issue_description: String,
    pub root_cause: String,
    pub solution_description: String,
    pub solution_steps: Option<Vec<String>>,
    pub files_modified: Option<Vec<String>>,
    pub validation_method: String,
    pub validation_evidence: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueResolutionResponse {
    pub id: String,
    pub session_id: String,
    pub issue_title: String,
    pub issue_description: String,
    pub root_cause: String,
    pub solution_description: String,
    pub solution_steps: Option<Vec<String>>,
    pub files_modified: Option<Vec<String>>,
    pub validation_method: String,
    pub validation_evidence: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Conversation History API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordMessageRequest {
    pub message_type: MessageType,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMessageResponse {
    pub id: String,
    pub session_id: String,
    pub message_type: MessageType,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub sequence_number: i32,
}

// ============================================================================
// Project Status and Metrics API Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatusResponse {
    pub project: ProjectInfo,
    pub feature_metrics: FeatureMetrics,
    pub task_summary: TaskSummary,
    pub session_summary: SessionSummary,
    pub recent_activity: Vec<RecentActivity>,
    pub health_indicators: HealthIndicators,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureMetrics {
    pub total_features: i32,
    pub implemented_features: i32,
    pub tested_features: i32,
    pub implementation_percentage: f32,
    pub test_coverage_percentage: f32,
    pub quality_score: f32,
    pub by_state: std::collections::HashMap<String, i32>,
    pub by_category: std::collections::HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSummary {
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub in_progress_tasks: i32,
    pub pending_tasks: i32,
    pub blocked_tasks: i32,
    pub completion_percentage: f32,
    pub by_priority: std::collections::HashMap<String, i32>,
    pub by_category: std::collections::HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    pub total_sessions: i32,
    pub active_sessions: i32,
    pub completed_sessions: i32,
    pub average_duration_minutes: Option<f32>,
    pub recent_sessions: Vec<SessionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentActivity {
    pub activity_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthIndicators {
    pub build_status: String,
    pub test_status: String,
    pub coverage_trend: String,
    pub velocity_trend: String,
    pub blocker_count: i32,
    pub last_successful_build: Option<DateTime<Utc>>,
}

// ============================================================================
// Feature Change Tracking Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordFeatureChangeRequest {
    pub feature_id: String,
    pub change_type: String,
    pub previous_state: Option<String>,
    pub new_state: String,
    pub reason: String,
    pub evidence_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureChangeResponse {
    pub id: String,
    pub session_id: String,
    pub feature_id: String,
    pub change_type: String,
    pub previous_state: Option<String>,
    pub new_state: String,
    pub reason: String,
    pub evidence_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Session Metrics Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMetricsResponse {
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
}

// ============================================================================
// Search and Query Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub entity_types: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub filters: Option<std::collections::HashMap<String, String>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub description: String,
    pub relevance_score: f32,
    pub matched_fields: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: i32,
    pub page_info: PageInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    pub current_page: i32,
    pub page_size: i32,
    pub total_pages: i32,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

// ============================================================================
// API Response Wrappers
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(code: String, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code,
                message,
                details: None,
            }),
            timestamp: Utc::now(),
        }
    }
}
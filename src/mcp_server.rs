use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::entities::{
    models::{Feature, Task, Session, Note, Project, Milestone, Dependency, NoteLink, NoteLinkQuery},
    validation::{ValidationResult, ValidationError, ValidationWarning, OperationRequest, EntityValidator, ValidationContext},
    git_integration::{GitManager, GitCommit, FileChange},
    relationships, note_links,
    EntityManager, EntityType,
};

/// MCP Server state
#[derive(Clone)]
pub struct McpServerState {
    pub entity_manager: Arc<EntityManager>,
    pub debug: bool,
}

/// Feature management request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AddFeatureRequest {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFeatureRequest {
    pub state: Option<String>,
    pub test_status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListFeaturesQuery {
    pub state: Option<String>,
    pub category: Option<String>,
    pub recent: Option<bool>,
    pub test_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindFeaturesQuery {
    pub state: Option<String>,
    pub test_status: Option<String>,
    pub added_since: Option<String>,
    pub added_by: Option<String>,
    pub notes_contain: Option<String>,
    pub category: Option<String>,
}

/// Task management request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AddTaskRequest {
    pub title: String,
    pub description: String,
    pub feature_id: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub notes: Option<String>,
}

/// Directive management request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AddDirectiveRequest {
    pub title: String,
    pub rule: String,
    pub category: String,
    pub priority: Option<String>,
    pub context: Option<String>,
    pub rationale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDirectiveRequest {
    pub title: Option<String>,
    pub rule: Option<String>,
    pub category: Option<String>,
    pub priority: Option<String>,
    pub context: Option<String>,
    pub rationale: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDirectivesQuery {
    pub category: Option<String>,
    pub priority: Option<String>,
    pub active: Option<bool>,
}

/// Relationship management request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRelationshipRequest {
    pub from_entity_id: String,
    pub from_entity_type: String,
    pub to_entity_id: String,
    pub to_entity_type: String,
    pub relationship_type: String,
    pub description: Option<String>,
}

/// Milestone management request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AddMilestoneRequest {
    pub title: String,
    pub description: String,
    pub target_date: Option<DateTime<Utc>>,
    pub feature_ids: Option<Vec<String>>,
    pub success_criteria: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMilestoneRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub target_date: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub completion_percentage: Option<f64>,
    pub feature_ids: Option<Vec<String>>,
    pub success_criteria: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMilestonesQuery {
    pub status: Option<String>,
    pub upcoming: Option<bool>,
    pub achieved: Option<bool>,
}

/// Note link management request/response types for F0137
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteLinkRequest {
    pub source_note_id: String,
    pub target_type: String,
    pub target_id: String,
    pub target_entity_type: Option<String>,
    pub link_type: String,
    pub auto_detected: Option<bool>,
    pub detection_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListNoteLinksQuery {
    pub source_note_id: Option<String>,
    pub target_id: Option<String>,
    pub target_type: Option<String>,
    pub link_type: Option<String>,
    pub auto_detected: Option<bool>,
    pub incoming: Option<bool>,
    pub outgoing: Option<bool>,
}

/// Note management request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AddNoteRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub content: String,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteSearchParams {
    pub content: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddProjectNoteRequest {
    pub title: String,
    pub content: String,
    pub category: String,
}

/// Project status response
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatusResponse {
    pub project: Project,
    pub feature_metrics: FeatureMetrics,
    pub task_summary: TaskSummary,
    pub recent_activity: Vec<ActivityItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureMetrics {
    pub total: usize,
    pub implemented: usize,
    pub tested: usize,
    pub implementation_percentage: f64,
    pub test_coverage_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSummary {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub blocked: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityItem {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub activity_type: String,
    pub description: String,
    pub entity_type: String,
    pub entity_id: String,
}

/// Session metrics structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionDetails {
    pub session: Session,
    pub metrics: SessionMetrics,
    pub conversation_history: Vec<ConversationMessage>,
    pub file_modifications: Vec<FileModification>,
    pub feature_changes: Vec<FeatureChange>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SessionMetrics {
    pub session_id: String,
    pub session_duration_seconds: u64,
    pub total_messages: u64,
    pub tool_calls: u64,
    pub context_usage_tokens: u64,
    pub average_response_time_ms: u64,
    pub peak_response_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    // Additional fields for comprehensive session tracking
    pub total_tool_calls: u64,
    pub total_response_time_ms: u64,
    pub context_used: u64,
    pub session_duration_ms: u64,
    pub features_created: u64,
    pub features_updated: u64,
    pub tasks_created: u64,
    pub tasks_completed: u64,
    pub files_modified: u64,
    pub issues_resolved: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMetricsResponse {
    pub metrics: SessionMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMetricsRequest {
    pub session_id: String,
    pub metrics: SessionMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMetricsResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationMessage {
    pub id: String,
    pub session_id: String,
    pub message_type: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileModification {
    pub id: String,
    pub session_id: String,
    pub file_path: String,
    pub modification_type: String,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureChange {
    pub id: String,
    pub session_id: String,
    pub feature_id: String,
    pub old_state: String,
    pub new_state: String,
    pub change_reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Validation error response for API endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    pub error: String,
    pub validation_errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub entity_type: String,
    pub entity_id: Option<String>,
}

/// Successful validation response
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationSuccessResponse {
    pub message: String,
    pub warnings: Vec<ValidationWarning>,
    pub entity_type: String,
    pub entity_id: Option<String>,
}

/// Constraint enforcement request
#[derive(Debug, Serialize, Deserialize)]
pub struct ConstraintEnforcementRequest {
    pub operation: OperationRequest,
    pub context: Option<ValidationContext>,
}

/// Timeline item for project history
#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineItem {
    pub timestamp: DateTime<Utc>,
    pub item_type: String, // "session", "commit", "milestone"
    pub title: String,
    pub description: Option<String>,
    pub session_id: Option<String>,
    pub commit_hash: Option<String>, 
    pub commit_info: Option<GitCommit>,
}

/// Project timeline response
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectTimeline {
    pub items: Vec<TimelineItem>,
}

/// Generic API error response with validation support
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub error_code: Option<String>,
    pub details: Option<String>,
    pub validation_errors: Option<Vec<ValidationError>>,
    pub suggestions: Option<Vec<String>>,
}

impl From<ValidationResult> for ValidationErrorResponse {
    fn from(result: ValidationResult) -> Self {
        Self {
            error: "Validation failed".to_string(),
            validation_errors: result.errors,
            warnings: result.warnings,
            entity_type: result.entity_type,
            entity_id: result.entity_id,
        }
    }
}

/// Start the MCP server
pub async fn start_mcp_server(port: u16, debug: bool) -> Result<()> {
    // Initialize entity manager with persistent database
    let db_path = std::env::current_dir()?.join(".ws").join("project.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    
    // Always initialize database to ensure all tables and columns exist
    let pool = crate::entities::database::initialize_database(&db_path).await?;
    
    let entity_manager = EntityManager::new(pool)
;
    
    let state = McpServerState {
        entity_manager: Arc::new(entity_manager),
        debug,
    };
    
    let app = create_router(state.clone());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    if debug {
        println!("Starting MCP server on http://{}", addr);
    }
    
    axum::Server::bind(&addr)
        .serve(app.with_state(state.clone()).into_make_service())
        .await
        .context("Failed to start MCP server")?;
    
    Ok(())
}

/// Create the router with all endpoints
fn create_router(_state: McpServerState) -> Router<McpServerState> {
    Router::new()
        // Feature management endpoints
        .route("/api/features", get(list_features).post(add_feature))
        .route("/api/features/:id", get(get_feature).put(update_feature))
        .route("/api/features/find", get(find_features))
        
        // Task management endpoints
        .route("/api/tasks", get(list_tasks).post(add_task))
        .route("/api/tasks/:id", get(get_task).put(update_task))
        
        // Directive management endpoints
        .route("/api/directives", get(list_directives).post(add_directive))
        .route("/api/directives/:id", get(get_directive).put(update_directive).delete(delete_directive))
        
        // Relationship management endpoints
        .route("/api/relationships", get(list_relationships).post(create_relationship))
        .route("/api/relationships/:id", delete(delete_relationship))
        .route("/api/relationships/:id/resolve", post(resolve_relationship))
        .route("/api/relationships/stats", get(get_relationship_stats))
        .route("/api/entities/:entity_id/relationships", get(get_entity_relationships))
        
        // Project management endpoints
        .route("/api/project/status", get(get_project_status))
        .route("/api/project/setup", post(setup_project))
        
        // Constraint enforcement endpoints
        .route("/api/validate/operation", post(validate_operation))
        .route("/api/enforce/constraints", post(enforce_constraints))
        
        // Milestone management endpoints
        .route("/api/milestones", get(list_milestones).post(add_milestone))
        .route("/api/milestones/:id", get(get_milestone).put(update_milestone).delete(delete_milestone))
        .route("/api/milestones/:id/achieve", post(achieve_milestone))
        
        // Note management endpoints
        .route("/api/notes", get(list_notes).post(add_note))
        .route("/api/notes/project", post(add_project_note))
        .route("/api/notes/search", get(search_notes))
        
        // Note link management endpoints for F0137
        .route("/api/note-links", get(list_note_links_api).post(create_note_link_api))
        .route("/api/note-links/:id", delete(delete_note_link_api))
        .route("/api/notes/:note_id/links", get(get_note_links_api))
        .route("/api/entities/:entity_id/note-links", get(get_entity_note_links_api))
        .route("/api/note-links/detect", post(detect_note_links_api))
        
        // Session management endpoints
        .route("/api/sessions", get(list_sessions).post(start_session))
        .route("/api/sessions/:id", get(get_session_details))
        .route("/api/sessions/:id/end", post(end_session))
        .route("/api/sessions/:id/metrics", get(get_session_metrics))
        .route("/api/sessions/:id/commits", get(get_session_commits))
        .route("/api/sessions/:id/files", get(get_session_files))
        .route("/api/sessions/:id/diff", get(get_session_diff))
        
        // Git history and timeline endpoints
        .route("/api/git/commits", get(list_git_commits))
        .route("/api/git/commits/:hash", get(get_commit_details))
        .route("/api/git/files/:commit_hash/*path", get(get_file_at_commit))
        .route("/api/git/diff/:from_commit/:to_commit", get(get_git_diff))
        .route("/api/git/timeline", get(get_project_timeline))
        
        // Metrics storage endpoints
        .route("/api/metrics/store", post(store_session_metrics_endpoint))
        
        // Health check
        .route("/health", get(health_check))
        
        // Serve static dashboard files
        .route("/", get(serve_dashboard))
        .route("/dashboard/*path", get(serve_static))
        
        .layer(CorsLayer::permissive())
}

/// Feature management handlers
async fn add_feature(
    State(state): State<McpServerState>,
    Json(request): Json<AddFeatureRequest>,
) -> Result<Json<Feature>, StatusCode> {
    let feature = state.entity_manager
        .create_feature(request.name, request.description)
        .await
        .map_err(|e| {
            eprintln!("Error creating feature: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(feature))
}

async fn list_features(
    State(state): State<McpServerState>,
    Query(params): Query<ListFeaturesQuery>,
) -> Result<Json<Vec<Feature>>, StatusCode> {
    let features = state.entity_manager
        .list_features()
        .await
        .map_err(|e| {
            eprintln!("Error listing features: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Apply filters based on query parameters
    let filtered = apply_feature_filters(features, &params);
    
    Ok(Json(filtered))
}

async fn get_feature(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<Feature>, StatusCode> {
    let feature = state.entity_manager
        .get_feature(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(feature))
}

async fn update_feature(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateFeatureRequest>,
) -> Result<Json<Feature>, StatusCode> {
    let mut feature = state.entity_manager
        .get_feature(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    if let Some(state) = request.state {
        feature.implementation_notes = Some(state);
    }
    
    if let Some(test_status) = request.test_status {
        feature.test_status = test_status;
    }
    
    if let Some(notes) = request.notes {
        feature.implementation_notes = Some(notes);
    }
    
    let updated = state.entity_manager
        .update_feature(feature)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(updated))
}

async fn find_features(
    State(state): State<McpServerState>,
    Query(params): Query<FindFeaturesQuery>,
) -> Result<Json<Vec<Feature>>, StatusCode> {
    let features = state.entity_manager
        .find_features(&crate::mcp_server::ListFeaturesQuery {
            state: params.state,
            category: params.category,
            recent: None,
            test_status: params.test_status,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(features))
}

/// Task management handlers
async fn add_task(
    State(state): State<McpServerState>,
    Json(request): Json<AddTaskRequest>,
) -> Result<Json<Task>, StatusCode> {
    let task = state.entity_manager
        .create_task(request.title, request.description)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(task))
}

async fn list_tasks(
    State(state): State<McpServerState>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks = state.entity_manager
        .list_tasks()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(tasks))
}

async fn get_task(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<Task>, StatusCode> {
    let task = state.entity_manager
        .get_task(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(task))
}

async fn update_task(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, StatusCode> {
    let mut task = state.entity_manager
        .get_task(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    if let Some(status) = request.status {
        // Parse status string to TaskStatus enum
        task.status = match status.as_str() {
            "pending" => crate::entities::TaskStatus::Pending,
            "in_progress" => crate::entities::TaskStatus::InProgress,
            "blocked" => crate::entities::TaskStatus::Blocked,
            "completed" => crate::entities::TaskStatus::Completed,
            "cancelled" => crate::entities::TaskStatus::Cancelled,
            _ => task.status, // Keep existing status if invalid
        };
    }
    
    if let Some(priority) = request.priority {
        // Parse priority string to Priority enum
        task.priority = match priority.as_str() {
            "high" => crate::entities::Priority::High,
            "medium" => crate::entities::Priority::Medium,
            "low" => crate::entities::Priority::Low,
            _ => task.priority, // Keep existing priority if invalid
        };
    }
    
    if let Some(notes) = request.notes {
        task.evidence = Some(notes);
    }
    
    let updated = state.entity_manager
        .update_task(task)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(updated))
}

/// Directive management handlers
async fn add_directive(
    State(state): State<McpServerState>,
    Json(request): Json<AddDirectiveRequest>,
) -> Result<Json<crate::entities::models::Directive>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse category and priority
    let category = match request.category.as_str() {
        "development" => crate::entities::DirectiveCategory::Development,
        "testing" => crate::entities::DirectiveCategory::Testing,
        "security" => crate::entities::DirectiveCategory::Security,
        "architecture" => crate::entities::DirectiveCategory::Architecture,
        "workflow" => crate::entities::DirectiveCategory::Workflow,
        "deployment" => crate::entities::DirectiveCategory::Deployment,
        "communication" => crate::entities::DirectiveCategory::Communication,
        _ => crate::entities::DirectiveCategory::Development,
    };

    let priority = match request.priority.as_deref().unwrap_or("medium") {
        "high" => crate::entities::Priority::High,
        "medium" => crate::entities::Priority::Medium,
        "low" => crate::entities::Priority::Low,
        _ => crate::entities::Priority::Medium,
    };

    let directive = state.entity_manager
        .create_directive(&project.id, request.title, request.rule, category, priority, request.context)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(directive))
}

async fn list_directives(
    State(state): State<McpServerState>,
    Query(params): Query<ListDirectivesQuery>,
) -> Result<Json<Vec<crate::entities::models::Directive>>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let directives = if params.active.unwrap_or(true) {
        state.entity_manager
            .get_active_directives(&project.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        state.entity_manager
            .get_directives(&project.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    
    Ok(Json(directives))
}

async fn get_directive(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<crate::entities::models::Directive>, StatusCode> {
    let directive = state.entity_manager
        .get_directive(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(directive))
}

async fn update_directive(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateDirectiveRequest>,
) -> Result<Json<crate::entities::models::Directive>, StatusCode> {
    let mut directive = state.entity_manager
        .get_directive(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Update fields if provided
    if let Some(title) = request.title {
        directive.title = title;
    }
    
    if let Some(rule) = request.rule {
        directive.rule = rule;
    }
    
    if let Some(category) = request.category {
        directive.category = match category.as_str() {
            "development" => crate::entities::DirectiveCategory::Development,
            "testing" => crate::entities::DirectiveCategory::Testing,
            "security" => crate::entities::DirectiveCategory::Security,
            "architecture" => crate::entities::DirectiveCategory::Architecture,
            "workflow" => crate::entities::DirectiveCategory::Workflow,
            "deployment" => crate::entities::DirectiveCategory::Deployment,
            "communication" => crate::entities::DirectiveCategory::Communication,
            _ => directive.category,
        };
    }
    
    if let Some(priority) = request.priority {
        directive.priority = match priority.as_str() {
            "high" => crate::entities::Priority::High,
            "medium" => crate::entities::Priority::Medium,
            "low" => crate::entities::Priority::Low,
            _ => directive.priority,
        };
    }
    
    if let Some(context) = request.context {
        directive.context = Some(context);
    }
    
    if let Some(rationale) = request.rationale {
        directive.rationale = Some(rationale);
    }
    
    if let Some(active) = request.active {
        directive.active = active;
    }
    
    let updated = state.entity_manager
        .update_directive(directive)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(updated))
}

async fn delete_directive(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.entity_manager
        .delete_directive(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"success": true})))
}

/// Relationship management handlers
async fn create_relationship(
    State(state): State<McpServerState>,
    Json(request): Json<CreateRelationshipRequest>,
) -> Result<Json<Dependency>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let from_entity_type = parse_entity_type_api(&request.from_entity_type)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let to_entity_type = parse_entity_type_api(&request.to_entity_type)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let dependency = relationships::create_dependency(
        &state.entity_manager.get_pool(),
        &project.id,
        &request.from_entity_id,
        from_entity_type,
        &request.to_entity_id,
        to_entity_type,
        request.relationship_type,
        request.description,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(dependency))
}

async fn list_relationships(
    State(state): State<McpServerState>,
) -> Result<Json<Vec<Dependency>>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|e| {
            eprintln!("Error getting current project for relationships: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let dependencies = relationships::get_project_dependencies(
        state.entity_manager.get_pool(),
        &project.id
    ).await
    .map_err(|e| {
        eprintln!("Error getting project dependencies: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(dependencies))
}

async fn get_entity_relationships(
    State(state): State<McpServerState>,
    Path(entity_id): Path<String>,
) -> Result<Json<std::collections::HashMap<EntityType, Vec<String>>>, StatusCode> {
    let relationships = relationships::get_relationships(
        state.entity_manager.get_pool(),
        &entity_id
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(relationships))
}

async fn delete_relationship(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    sqlx::query("DELETE FROM dependencies WHERE id = ?")
        .bind(&id)
        .execute(state.entity_manager.get_pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"success": true})))
}

async fn resolve_relationship(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    relationships::resolve_dependency(state.entity_manager.get_pool(), &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"success": true})))
}

async fn get_relationship_stats(
    State(state): State<McpServerState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let dependencies = relationships::get_project_dependencies(
        state.entity_manager.get_pool(),
        &project.id
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let stats = serde_json::json!({
        "total_relationships": dependencies.len(),
        "active_relationships": dependencies.iter().filter(|d| d.resolved_at.is_none()).count(),
        "resolved_relationships": dependencies.iter().filter(|d| d.resolved_at.is_some()).count(),
    });
    
    Ok(Json(stats))
}

/// Milestone management handlers
async fn add_milestone(
    State(state): State<McpServerState>,
    Json(request): Json<AddMilestoneRequest>,
) -> Result<Json<Milestone>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|e| {
            eprintln!("Error getting current project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let milestone = state.entity_manager
        .create_milestone(
            &project.id,
            &request.title,
            &request.description,
            request.target_date,
            request.feature_ids,
            request.success_criteria,
        )
        .await
        .map_err(|e| {
            eprintln!("Error creating milestone: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(milestone))
}

async fn list_milestones(
    State(state): State<McpServerState>,
    Query(query): Query<ListMilestonesQuery>,
) -> Result<Json<Vec<Milestone>>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|e| {
            eprintln!("Error getting current project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let milestones = if let Some(status) = query.status {
        state.entity_manager
            .get_milestones_by_status(&project.id, &status)
            .await
    } else {
        state.entity_manager
            .get_milestones_by_project(&project.id)
            .await
    }.map_err(|e| {
        eprintln!("Error listing milestones: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(milestones))
}

async fn get_milestone(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<Milestone>, StatusCode> {
    let milestone = state.entity_manager
        .get_milestone(&id)
        .await
        .map_err(|e| {
            eprintln!("Error getting milestone: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(milestone))
}

async fn update_milestone(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateMilestoneRequest>,
) -> Result<Json<Milestone>, StatusCode> {
    let milestone = state.entity_manager
        .update_milestone(
            &id,
            request.title.as_deref(),
            request.description.as_deref(),
            request.target_date.map(Some).or(Some(None)),
            request.status.as_deref(),
            request.completion_percentage,
            request.feature_ids,
            request.success_criteria,
        )
        .await
        .map_err(|e| {
            eprintln!("Error updating milestone: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(milestone))
}

async fn achieve_milestone(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<Milestone>, StatusCode> {
    let milestone = state.entity_manager
        .achieve_milestone(&id)
        .await
        .map_err(|e| {
            eprintln!("Error achieving milestone: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(milestone))
}

async fn delete_milestone(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.entity_manager
        .delete_milestone(&id)
        .await
        .map_err(|e| {
            eprintln!("Error deleting milestone: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(serde_json::json!({"success": true})))
}

/// Project management handlers
async fn get_project_status(
    State(state): State<McpServerState>,
) -> Result<Json<ProjectStatusResponse>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let features = state.entity_manager
        .list_features()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let tasks = state.entity_manager
        .list_tasks()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let feature_metrics = calculate_feature_metrics(&features);
    let task_summary = calculate_task_summary(&tasks);
    let recent_activity = get_recent_activity(&state).await?;
    
    let response = ProjectStatusResponse {
        project,
        feature_metrics,
        task_summary,
        recent_activity,
    };
    
    Ok(Json(response))
}

async fn setup_project(
    State(state): State<McpServerState>,
    Json(request): Json<HashMap<String, String>>,
) -> Result<Json<Project>, StatusCode> {
    let name = request.get("name")
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let project = state.entity_manager
        .create_project(name.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(project))
}

/// Note management handlers
async fn add_note(
    State(state): State<McpServerState>,
    Json(request): Json<AddNoteRequest>,
) -> Result<Json<Note>, StatusCode> {
    let note = state.entity_manager
        .create_note(
            request.entity_type,
            request.entity_id,
            request.content,
            request.category,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(note))
}

async fn add_project_note(
    State(state): State<McpServerState>,
    Json(request): Json<AddProjectNoteRequest>,
) -> Result<Json<Note>, StatusCode> {
    let note = state.entity_manager
        .create_project_note(
            request.title,
            request.content,
            request.category,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(note))
}

async fn list_notes(
    State(state): State<McpServerState>,
) -> Result<Json<Vec<Note>>, StatusCode> {
    let notes = state.entity_manager
        .list_notes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(notes))
}

async fn search_notes(
    State(state): State<McpServerState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Note>>, StatusCode> {
    let notes = state.entity_manager
        .search_notes(&NoteSearchParams {
            content: params.get("content").cloned(),
            category: params.get("category").cloned(),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(notes))
}

/// Session management handlers
async fn list_sessions(
    State(state): State<McpServerState>,
) -> Result<Json<Vec<Session>>, StatusCode> {
    let sessions = state.entity_manager
        .list_sessions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(sessions))
}

async fn start_session(
    State(state): State<McpServerState>,
    Json(request): Json<HashMap<String, String>>,
) -> Result<Json<Session>, StatusCode> {
    let description = request.get("description")
        .cloned()
        .unwrap_or_else(|| "Auto-started session".to_string());
    
    let session = state.entity_manager
        .start_session(description)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(session))
}

async fn end_session(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, StatusCode> {
    let session = state.entity_manager
        .end_session(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(session))
}

/// Session metrics handlers
async fn get_session_details(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<SessionDetails>, StatusCode> {
    let session = state.entity_manager
        .get_session(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Get session metrics from database
    let metrics = state.entity_manager
        .get_session_metrics(&id)
        .await
        .unwrap_or_default();
    
    let details = SessionDetails {
        session,
        metrics,
        conversation_history: Vec::new(), // TODO: Implement conversation history
        file_modifications: Vec::new(),   // TODO: Implement file tracking
        feature_changes: Vec::new(),      // TODO: Implement feature change tracking
    };
    
    Ok(Json(details))
}

async fn get_session_metrics(
    State(state): State<McpServerState>,
    Path(id): Path<String>,
) -> Result<Json<SessionMetricsResponse>, StatusCode> {
    let metrics = state.entity_manager
        .get_session_metrics(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(SessionMetricsResponse { metrics }))
}

async fn store_session_metrics_endpoint(
    State(state): State<McpServerState>,
    Json(request): Json<StoreMetricsRequest>,
) -> Result<Json<StoreMetricsResponse>, StatusCode> {
    state.entity_manager
        .store_session_metrics(&request.session_id, &request.metrics)
        .await
        .map_err(|e| {
            eprintln!("Error storing session metrics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(StoreMetricsResponse { 
        success: true,
        message: "Session metrics stored successfully".to_string(),
    }))
}

/// Health check handler
async fn health_check() -> Json<HashMap<&'static str, &'static str>> {
    let mut response = HashMap::new();
    response.insert("status", "healthy");
    response.insert("service", "ws-mcp-server");
    Json(response)
}

/// Dashboard handlers
async fn serve_dashboard() -> Result<axum::response::Html<&'static str>, StatusCode> {
    Ok(axum::response::Html(include_str!("static/dashboard.html")))
}

async fn serve_static(Path(path): Path<String>) -> Result<(axum::http::HeaderMap, String), StatusCode> {
    // Basic static file serving - in production, use proper static file server
    let (content, content_type) = match path.as_str() {
        "app.js" => (include_str!("static/app.js"), "application/javascript"),
        "style.css" => (include_str!("static/style.css"), "text/css"),
        _ => return Err(StatusCode::NOT_FOUND),
    };
    
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", content_type.parse().unwrap());
    
    Ok((headers, content.to_string()))
}

/// Helper functions
fn apply_feature_filters(features: Vec<Feature>, params: &ListFeaturesQuery) -> Vec<Feature> {
    features.into_iter()
        .filter(|f| {
            if let Some(state) = &params.state {
                if f.state.to_string() != *state {
                    return false;
                }
            }
            
            if let Some(category) = &params.category {
                if f.category.as_ref() != Some(category) {
                    return false;
                }
            }
            
            if let Some(test_status) = &params.test_status {
                if f.test_status != *test_status {
                    return false;
                }
            }
            
            // Add more filters as needed
            true
        })
        .collect()
}

fn calculate_feature_metrics(features: &[Feature]) -> FeatureMetrics {
    let total = features.len();
    
    // Count implemented features (🟢 TestedPassing + 🟠 Implemented)
    let implemented = features.iter()
        .filter(|f| matches!(f.state, 
            crate::entities::FeatureState::TestedPassing |
            crate::entities::FeatureState::Implemented
        ))
        .count();
    
    // Count tested features (🟢 TestedPassing)
    let tested = features.iter()
        .filter(|f| matches!(f.state, crate::entities::FeatureState::TestedPassing))
        .count();
    
    let implementation_percentage = if total > 0 {
        (implemented as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    
    let test_coverage_percentage = if total > 0 {
        (tested as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    
    FeatureMetrics {
        total,
        implemented,
        tested,
        implementation_percentage,
        test_coverage_percentage,
    }
}

fn calculate_task_summary(tasks: &[Task]) -> TaskSummary {
    let total = tasks.len();
    let pending = tasks.iter().filter(|t| t.status.as_str() == "pending").count();
    let in_progress = tasks.iter().filter(|t| t.status.as_str() == "in_progress").count();
    let completed = tasks.iter().filter(|t| t.status.as_str() == "completed").count();
    let blocked = tasks.iter().filter(|t| t.status.as_str() == "blocked").count();
    
    TaskSummary {
        total,
        pending,
        in_progress,
        completed,
        blocked,
    }
}

async fn get_recent_activity(state: &McpServerState) -> Result<Vec<ActivityItem>, StatusCode> {
    // Get recent features and tasks for activity feed
    let features = state.entity_manager
        .list_features()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let tasks = state.entity_manager
        .list_tasks()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut activity = Vec::new();
    
    // Add recent feature updates (last 10)
    for feature in features.iter().take(10) {
        activity.push(ActivityItem {
            timestamp: feature.updated_at,
            activity_type: "feature_update".to_string(),
            description: format!("Feature {} updated: {}", feature.code, feature.name),
            entity_type: "feature".to_string(),
            entity_id: feature.code.clone(),
        });
    }
    
    // Add recent task updates (last 10)
    for task in tasks.iter().take(10) {
        activity.push(ActivityItem {
            timestamp: task.updated_at,
            activity_type: "task_update".to_string(),
            description: format!("Task updated: {}", task.title),
            entity_type: "task".to_string(),
            entity_id: task.id.to_string(),
        });
    }
    
    // Sort by timestamp descending (most recent first)
    activity.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // Limit to 20 most recent items
    activity.truncate(20);
    
    Ok(activity)
}

/// Git integration handlers

/// Get commits associated with a specific session
async fn get_session_commits(
    State(state): State<McpServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<GitCommit>>, StatusCode> {
    let current_dir = std::env::current_dir()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let git_manager = GitManager::new(&current_dir);
    
    if !git_manager.is_git_repo() {
        return Ok(Json(Vec::new()));
    }
    
    let pattern = format!("Session {}", session_id);
    let commits = git_manager.get_session_commits(&pattern)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(commits))
}

/// Get files modified in a session
async fn get_session_files(
    State(state): State<McpServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<String>>, StatusCode> {
    // Get session from database first
    let session = state.entity_manager
        .get_session(&session_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Extract commit ID from session metadata
    let commit_hash = if let Some(metadata) = &session.metadata {
        // Parse JSON metadata to get commit_id
        serde_json::from_str::<serde_json::Value>(metadata)
            .ok()
            .and_then(|v| v.get("commit_id")?.as_str().map(|s| s.to_string()))
    } else {
        None
    };
    
    if let Some(commit_hash) = commit_hash {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let git_manager = GitManager::new(&current_dir);
        
        let files = git_manager.list_files_at_commit(&commit_hash)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Json(files))
    } else {
        Ok(Json(Vec::new()))
    }
}

/// Get diff for a session's changes
async fn get_session_diff(
    State(state): State<McpServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<FileChange>>, StatusCode> {
    let session = state.entity_manager
        .get_session(&session_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Extract commit ID from session metadata
    let commit_hash = if let Some(metadata) = &session.metadata {
        serde_json::from_str::<serde_json::Value>(metadata)
            .ok()
            .and_then(|v| v.get("commit_id")?.as_str().map(|s| s.to_string()))
    } else {
        None
    };
    
    if let Some(commit_hash) = commit_hash {
        let current_dir = std::env::current_dir()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let git_manager = GitManager::new(&current_dir);
        
        // Get diff from previous commit to this commit
        let diff = git_manager.get_diff(Some(&format!("{}~1", commit_hash)), Some(&commit_hash))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Json(diff))
    } else {
        Ok(Json(Vec::new()))
    }
}

/// List all git commits in the repository
async fn list_git_commits(
    State(_state): State<McpServerState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<GitCommit>>, StatusCode> {
    let current_dir = std::env::current_dir()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let git_manager = GitManager::new(&current_dir);
    
    if !git_manager.is_git_repo() {
        return Ok(Json(Vec::new()));
    }
    
    // For now, get recent commits (this could be expanded with pagination)
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    
    // Use git log to get recent commits
    let output = std::process::Command::new("git")
        .args(&["log", "--format=%H", &format!("-{}", limit)])
        .current_dir(&current_dir)
        .output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !output.status.success() {
        return Ok(Json(Vec::new()));
    }
    
    let commit_hashes = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    
    for hash in commit_hashes.lines() {
        if let Ok(commit) = git_manager.get_commit_info(hash.trim()) {
            commits.push(commit);
        }
    }
    
    Ok(Json(commits))
}

/// Get detailed information about a specific commit
async fn get_commit_details(
    State(_state): State<McpServerState>,
    Path(hash): Path<String>,
) -> Result<Json<GitCommit>, StatusCode> {
    let current_dir = std::env::current_dir()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let git_manager = GitManager::new(&current_dir);
    
    let commit = git_manager.get_commit_info(&hash)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(commit))
}

/// Get file content at a specific commit
async fn get_file_at_commit(
    State(_state): State<McpServerState>,
    Path((commit_hash, file_path)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let current_dir = std::env::current_dir()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let git_manager = GitManager::new(&current_dir);
    
    let content = git_manager.get_file_at_commit(&file_path, &commit_hash)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(serde_json::json!({
        "commit_hash": commit_hash,
        "file_path": file_path,
        "content": content
    })))
}

/// Get diff between two commits
async fn get_git_diff(
    State(_state): State<McpServerState>,
    Path((from_commit, to_commit)): Path<(String, String)>,
) -> Result<Json<Vec<FileChange>>, StatusCode> {
    let current_dir = std::env::current_dir()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let git_manager = GitManager::new(&current_dir);
    
    let diff = git_manager.get_diff(Some(&from_commit), Some(&to_commit))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(diff))
}

/// Get project timeline showing sessions and their commits
async fn get_project_timeline(
    State(state): State<McpServerState>,
) -> Result<Json<ProjectTimeline>, StatusCode> {
    let sessions = state.entity_manager
        .list_sessions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let current_dir = std::env::current_dir()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let git_manager = GitManager::new(&current_dir);
    
    let mut timeline_items = Vec::new();
    
    for session in sessions {
        let commit_hash = if let Some(metadata) = &session.metadata {
            serde_json::from_str::<serde_json::Value>(metadata)
                .ok()
                .and_then(|v| v.get("commit_id")?.as_str().map(|s| s.to_string()))
        } else {
            None
        };
        
        let commit_info = if let Some(hash) = &commit_hash {
            git_manager.get_commit_info(hash).ok()
        } else {
            None
        };
        
        timeline_items.push(TimelineItem {
            timestamp: session.started_at,
            item_type: "session".to_string(),
            title: session.title.clone(),
            description: session.description.clone(),
            session_id: Some(session.id.clone()),
            commit_hash,
            commit_info,
        });
    }
    
    // Sort by timestamp descending
    timeline_items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(Json(ProjectTimeline {
        items: timeline_items,
    }))
}

/// Constraint enforcement API handlers

/// Validate operation request without executing it
async fn validate_operation(
    State(state): State<McpServerState>,
    Json(request): Json<ConstraintEnforcementRequest>,
) -> Result<Json<ValidationResult>, Json<ValidationErrorResponse>> {
    let validator = EntityValidator::new();
    let context = request.context.unwrap_or_default();
    
    let result = validator.validate_operation(&request.operation, &context);
    
    if result.is_valid {
        Ok(Json(result))
    } else {
        Err(Json(ValidationErrorResponse::from(result)))
    }
}

/// Enforce constraints and block invalid operations
async fn enforce_constraints(
    State(state): State<McpServerState>,
    Json(request): Json<ConstraintEnforcementRequest>,
) -> Result<Json<ValidationSuccessResponse>, Json<ValidationErrorResponse>> {
    let validator = EntityValidator::new();
    let context = request.context.unwrap_or_default();
    
    match validator.enforce_constraints(&request.operation, &context) {
        Ok(()) => Ok(Json(ValidationSuccessResponse {
            message: "Operation validated and allowed".to_string(),
            warnings: Vec::new(),
            entity_type: request.operation.entity_type,
            entity_id: request.operation.entity_id,
        })),
        Err(e) => {
            let result = validator.validate_operation(&request.operation, &context);
            Err(Json(ValidationErrorResponse::from(result)))
        }
    }
}

/// Utility functions for relationship management
fn parse_entity_type_api(type_str: &str) -> Result<EntityType, anyhow::Error> {
    match type_str.to_lowercase().as_str() {
        "feature" => Ok(EntityType::Feature),
        "task" => Ok(EntityType::Task),
        "session" => Ok(EntityType::Session),
        "project" => Ok(EntityType::Project),
        "directive" => Ok(EntityType::Directive),
        "note" => Ok(EntityType::Note),
        "template" => Ok(EntityType::Template),
        "dependency" => Ok(EntityType::Dependency),
        "milestone" => Ok(EntityType::Milestone),
        "test" => Ok(EntityType::Test),
        _ => Err(anyhow::anyhow!("Unknown entity type: {}", type_str)),
    }
}

/// Note link management API handlers for F0137

async fn create_note_link_api(
    State(state): State<McpServerState>,
    Json(request): Json<CreateNoteLinkRequest>,
) -> Result<Json<NoteLink>, StatusCode> {
    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|e| {
            eprintln!("Error getting current project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let link = state.entity_manager
        .create_note_link(
            &project.id,
            &request.source_note_id,
            &request.target_type,
            &request.target_id,
            request.target_entity_type.as_deref(),
            &request.link_type,
            request.auto_detected.unwrap_or(false),
            request.detection_reason.as_deref(),
        )
        .await
        .map_err(|e| {
            eprintln!("Error creating note link: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(link))
}

async fn list_note_links_api(
    State(state): State<McpServerState>,
    Query(query): Query<ListNoteLinksQuery>,
) -> Result<Json<Vec<NoteLink>>, StatusCode> {
    let note_link_query = NoteLinkQuery {
        source_note_id: query.source_note_id,
        target_id: query.target_id,
        target_type: query.target_type,
        link_type: query.link_type,
        auto_detected: query.auto_detected,
        project_id: None, // Will be filtered by current project
        limit: Some(100),
        offset: None,
    };

    let links = state.entity_manager
        .query_note_links(&note_link_query)
        .await
        .map_err(|e| {
            eprintln!("Error querying note links: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(links))
}

async fn delete_note_link_api(
    State(state): State<McpServerState>,
    Path(link_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let removed = state.entity_manager
        .remove_note_link(&link_id)
        .await
        .map_err(|e| {
            eprintln!("Error removing note link: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if removed {
        Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Note link {} removed", link_id)
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_note_links_api(
    State(state): State<McpServerState>,
    Path(note_id): Path<String>,
) -> Result<Json<Vec<NoteLink>>, StatusCode> {
    let links = state.entity_manager
        .get_note_links(&note_id)
        .await
        .map_err(|e| {
            eprintln!("Error getting note links: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(links))
}

async fn get_entity_note_links_api(
    State(state): State<McpServerState>,
    Path(entity_id): Path<String>,
    Query(query): Query<ListNoteLinksQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let entity_type = if entity_id.starts_with("note-") {
        Some("note")
    } else if entity_id.starts_with("F") {
        Some("entity")
    } else {
        Some("entity")
    };

    let (outgoing_links, incoming_links) = state.entity_manager
        .get_bidirectional_links(&entity_id, entity_type)
        .await
        .map_err(|e| {
            eprintln!("Error getting bidirectional links: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let show_incoming = query.incoming.unwrap_or(true);
    let show_outgoing = query.outgoing.unwrap_or(true);

    Ok(Json(serde_json::json!({
        "entity_id": entity_id,
        "outgoing_links": if show_outgoing { outgoing_links } else { Vec::<NoteLink>::new() },
        "incoming_links": if show_incoming { incoming_links } else { Vec::<NoteLink>::new() }
    })))
}

async fn detect_note_links_api(
    State(state): State<McpServerState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<Vec<note_links::DetectedLink>>, StatusCode> {
    let content = request.get("content")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let project = state.entity_manager
        .get_current_project()
        .await
        .map_err(|e| {
            eprintln!("Error getting current project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let detected_links = state.entity_manager
        .detect_note_links(&project.id, content)
        .await
        .map_err(|e| {
            eprintln!("Error detecting note links: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(detected_links))
}
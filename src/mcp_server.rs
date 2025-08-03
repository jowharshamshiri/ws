use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::entities::{
    models::{Feature, Task, Session, Note, Project},
    EntityManager,
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

/// Start the MCP server
pub async fn start_mcp_server(port: u16, debug: bool) -> Result<()> {
    // Initialize entity manager with persistent database
    let db_path = std::env::current_dir()?.join(".ws").join("project.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    
    let pool = if db_path.exists() {
        // Use existing database
        sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?
    } else {
        // Initialize new database
        crate::entities::database::initialize_database(&db_path).await?
    };
    
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
        
        // Project management endpoints
        .route("/api/project/status", get(get_project_status))
        .route("/api/project/setup", post(setup_project))
        
        // Note management endpoints
        .route("/api/notes", get(list_notes).post(add_note))
        .route("/api/notes/project", post(add_project_note))
        .route("/api/notes/search", get(search_notes))
        
        // Session management endpoints
        .route("/api/sessions", get(list_sessions).post(start_session))
        .route("/api/sessions/:id/end", post(end_session))
        
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
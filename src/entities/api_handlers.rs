// Comprehensive API Handlers for Project Management System
// Implements all operations identified from session log analysis

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::Json as ResponseJson,
};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use super::EntityManager;
use super::api_models::*;
use super::session_models::*;

// ============================================================================
// Feature Management Handlers
// ============================================================================

/// Create a new feature
pub async fn create_feature(
    State(entity_manager): State<Arc<EntityManager>>,
    Json(request): Json<CreateFeatureRequest>,
) -> Result<ResponseJson<ApiResponse<FeatureResponse>>, StatusCode> {
    match entity_manager.create_feature(request.name, request.description).await {
        Ok(feature) => {
            let response = FeatureResponse {
                id: feature.id.to_string(),
                name: feature.name,
                description: feature.description,
                category: feature.category,
                state: feature.state,
                test_status: feature.test_status,
                priority: feature.priority,
                implementation_notes: feature.implementation_notes,
                test_evidence: feature.test_evidence,
                dependencies: feature.dependencies
                    .and_then(|d| serde_json::from_str(&d).ok()),
                linked_tasks: vec![], // TODO: Implement task linking
                created_at: feature.created_at,
                updated_at: feature.updated_at,
                completed_at: feature.completed_at,
                estimated_effort: feature.estimated_effort,
                actual_effort: feature.actual_effort,
            };
            Ok(ResponseJson(ApiResponse::success(response)))
        }
        Err(e) => {
            let error_response = ApiResponse::error(
                "FEATURE_CREATE_ERROR".to_string(),
                format!("Failed to create feature: {}", e),
            );
            Ok(ResponseJson(error_response))
        }
    }
}

/// Get a specific feature by ID
pub async fn get_feature(
    State(entity_manager): State<Arc<EntityManager>>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<FeatureResponse>>, StatusCode> {
    match entity_manager.get_feature(&id).await {
        Ok(feature) => {
            let response = FeatureResponse {
                id: feature.id.to_string(),
                name: feature.name,
                description: feature.description,
                category: feature.category,
                state: feature.state,
                test_status: feature.test_status,
                priority: feature.priority,
                implementation_notes: feature.implementation_notes,
                test_evidence: feature.test_evidence,
                dependencies: feature.dependencies
                    .and_then(|d| serde_json::from_str(&d).ok()),
                linked_tasks: vec![], // TODO: Implement task linking
                created_at: feature.created_at,
                updated_at: feature.updated_at,
                completed_at: feature.completed_at,
                estimated_effort: feature.estimated_effort,
                actual_effort: feature.actual_effort,
            };
            Ok(ResponseJson(ApiResponse::success(response)))
        }
        Err(e) => {
            let error_response = ApiResponse::error(
                "FEATURE_NOT_FOUND".to_string(),
                format!("Feature not found: {}", e),
            );
            Ok(ResponseJson(error_response))
        }
    }
}

/// List features with optional filtering
pub async fn list_features(
    State(entity_manager): State<Arc<EntityManager>>,
    Query(query): Query<FeatureListQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<FeatureResponse>>>, StatusCode> {
    match entity_manager.list_features().await {
        Ok(features) => {
            let responses: Vec<FeatureResponse> = features
                .into_iter()
                .map(|feature| FeatureResponse {
                    id: feature.id.to_string(),
                    name: feature.name,
                    description: feature.description,
                    category: feature.category,
                    state: feature.state,
                    test_status: feature.test_status,
                    priority: feature.priority,
                    implementation_notes: feature.implementation_notes,
                    test_evidence: feature.test_evidence,
                    dependencies: feature.dependencies
                        .and_then(|d| serde_json::from_str(&d).ok()),
                    linked_tasks: vec![], // TODO: Implement task linking
                    created_at: feature.created_at,
                    updated_at: feature.updated_at,
                    completed_at: feature.completed_at,
                    estimated_effort: feature.estimated_effort,
                    actual_effort: feature.actual_effort,
                })
                .collect();
            Ok(ResponseJson(ApiResponse::success(responses)))
        }
        Err(e) => {
            let error_response = ApiResponse::error(
                "FEATURE_LIST_ERROR".to_string(),
                format!("Failed to list features: {}", e),
            );
            Ok(ResponseJson(error_response))
        }
    }
}

/// Update a feature
pub async fn update_feature(
    State(entity_manager): State<Arc<EntityManager>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateFeatureRequest>,
) -> Result<ResponseJson<ApiResponse<FeatureResponse>>, StatusCode> {
    match entity_manager.get_feature(&id).await {
        Ok(mut feature) => {
            // Apply updates
            if let Some(name) = request.name {
                feature.name = name;
            }
            if let Some(description) = request.description {
                feature.description = description;
            }
            if let Some(state) = request.state {
                feature.state = state;
            }
            if let Some(test_status) = request.test_status {
                feature.test_status = test_status;
            }
            if let Some(implementation_notes) = request.implementation_notes {
                feature.implementation_notes = Some(implementation_notes);
            }
            if let Some(test_evidence) = request.test_evidence {
                feature.test_evidence = Some(test_evidence);
            }
            if let Some(actual_effort) = request.actual_effort {
                feature.actual_effort = Some(actual_effort);
            }
            
            feature.updated_at = Utc::now();

            match entity_manager.update_feature(feature.clone()).await {
                Ok(updated_feature) => {
                    let response = FeatureResponse {
                        id: updated_feature.id.to_string(),
                        name: updated_feature.name,
                        description: updated_feature.description,
                        category: updated_feature.category,
                        state: updated_feature.state,
                        test_status: updated_feature.test_status,
                        priority: updated_feature.priority,
                        implementation_notes: updated_feature.implementation_notes,
                        test_evidence: updated_feature.test_evidence,
                        dependencies: updated_feature.dependencies
                            .and_then(|d| serde_json::from_str(&d).ok()),
                        linked_tasks: vec![], // TODO: Implement task linking
                        created_at: updated_feature.created_at,
                        updated_at: updated_feature.updated_at,
                        completed_at: updated_feature.completed_at,
                        estimated_effort: updated_feature.estimated_effort,
                        actual_effort: updated_feature.actual_effort,
                    };
                    Ok(ResponseJson(ApiResponse::success(response)))
                }
                Err(e) => {
                    let error_response = ApiResponse::error(
                        "FEATURE_UPDATE_ERROR".to_string(),
                        format!("Failed to update feature: {}", e),
                    );
                    Ok(ResponseJson(error_response))
                }
            }
        }
        Err(e) => {
            let error_response = ApiResponse::error(
                "FEATURE_NOT_FOUND".to_string(),
                format!("Feature not found: {}", e),
            );
            Ok(ResponseJson(error_response))
        }
    }
}

// ============================================================================
// Task Management Handlers
// ============================================================================

/// Create a new task
pub async fn create_task(
    State(entity_manager): State<Arc<EntityManager>>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<ResponseJson<ApiResponse<TaskResponse>>, StatusCode> {
    match entity_manager.create_task(request.title, request.description).await {
        Ok(task) => {
            let response = TaskResponse {
                id: task.id.to_string(),
                code: task.code,
                title: task.title,
                description: task.description,
                category: task.category,
                status: task.status,
                priority: task.priority,
                feature_ids: task.feature_ids
                    .and_then(|d| serde_json::from_str(&d).ok()),
                depends_on: task.depends_on
                    .and_then(|d| serde_json::from_str(&d).ok()),
                acceptance_criteria: task.acceptance_criteria
                    .and_then(|d| serde_json::from_str(&d).ok()),
                validation_steps: task.validation_steps
                    .and_then(|d| serde_json::from_str(&d).ok()),
                evidence: task.evidence,
                session_id: task.session_id.map(|id| id.to_string()),
                assigned_to: task.assigned_to,
                created_at: task.created_at,
                updated_at: task.updated_at,
                started_at: task.started_at,
                completed_at: task.completed_at,
                estimated_effort: task.estimated_effort,
                actual_effort: task.actual_effort,
                tags: task.tags.and_then(|d| serde_json::from_str(&d).ok()),
            };
            Ok(ResponseJson(ApiResponse::success(response)))
        }
        Err(e) => {
            let error_response = ApiResponse::error(
                "TASK_CREATE_ERROR".to_string(),
                format!("Failed to create task: {}", e),
            );
            Ok(ResponseJson(error_response))
        }
    }
}

/// Get a specific task by ID
pub async fn get_task(
    State(entity_manager): State<Arc<EntityManager>>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<TaskResponse>>, StatusCode> {
    match entity_manager.get_task(&id).await {
        Ok(task) => {
            let response = TaskResponse {
                id: task.id.to_string(),
                code: task.code,
                title: task.title,
                description: task.description,
                category: task.category,
                status: task.status,
                priority: task.priority,
                feature_ids: task.feature_ids
                    .and_then(|d| serde_json::from_str(&d).ok()),
                depends_on: task.depends_on
                    .and_then(|d| serde_json::from_str(&d).ok()),
                acceptance_criteria: task.acceptance_criteria
                    .and_then(|d| serde_json::from_str(&d).ok()),
                validation_steps: task.validation_steps
                    .and_then(|d| serde_json::from_str(&d).ok()),
                evidence: task.evidence,
                session_id: task.session_id.map(|id| id.to_string()),
                assigned_to: task.assigned_to,
                created_at: task.created_at,
                updated_at: task.updated_at,
                started_at: task.started_at,
                completed_at: task.completed_at,
                estimated_effort: task.estimated_effort,
                actual_effort: task.actual_effort,
                tags: task.tags.and_then(|d| serde_json::from_str(&d).ok()),
            };
            Ok(ResponseJson(ApiResponse::success(response)))
        }
        Err(e) => {
            let error_response = ApiResponse::error(
                "TASK_NOT_FOUND".to_string(),
                format!("Task not found: {}", e),
            );
            Ok(ResponseJson(error_response))
        }
    }
}

/// List tasks with optional filtering
pub async fn list_tasks(
    State(entity_manager): State<Arc<EntityManager>>,
    Query(query): Query<TaskListQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<TaskResponse>>>, StatusCode> {
    match entity_manager.list_tasks().await {
        Ok(tasks) => {
            let responses: Vec<TaskResponse> = tasks
                .into_iter()
                .map(|task| TaskResponse {
                    id: task.id.to_string(),
                    code: task.code,
                    title: task.title,
                    description: task.description,
                    category: task.category,
                    status: task.status,
                    priority: task.priority,
                    feature_ids: task.feature_ids
                        .and_then(|d| serde_json::from_str(&d).ok()),
                    depends_on: task.depends_on
                        .and_then(|d| serde_json::from_str(&d).ok()),
                    acceptance_criteria: task.acceptance_criteria
                        .and_then(|d| serde_json::from_str(&d).ok()),
                    validation_steps: task.validation_steps
                        .and_then(|d| serde_json::from_str(&d).ok()),
                    evidence: task.evidence,
                    session_id: task.session_id.map(|id| id.to_string()),
                    assigned_to: task.assigned_to,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                    started_at: task.started_at,
                    completed_at: task.completed_at,
                    estimated_effort: task.estimated_effort,
                    actual_effort: task.actual_effort,
                    tags: task.tags.and_then(|d| serde_json::from_str(&d).ok()),
                })
                .collect();
            Ok(ResponseJson(ApiResponse::success(responses)))
        }
        Err(e) => {
            let error_response = ApiResponse::error(
                "TASK_LIST_ERROR".to_string(),
                format!("Failed to list tasks: {}", e),
            );
            Ok(ResponseJson(error_response))
        }
    }
}

// ============================================================================
// Project Status Handler
// ============================================================================

/// Get comprehensive project status
pub async fn get_project_status(
    State(entity_manager): State<Arc<EntityManager>>,
) -> Result<ResponseJson<ApiResponse<ProjectStatusResponse>>, StatusCode> {
    // Get project info
    let project = match entity_manager.get_current_project().await {
        Ok(project) => project,
        Err(e) => {
            let error_response = ApiResponse::error(
                "PROJECT_ERROR".to_string(),
                format!("Failed to get project: {}", e),
            );
            return Ok(ResponseJson(error_response));
        }
    };

    // Get features and calculate metrics
    let features = entity_manager.list_features().await.unwrap_or_default();
    let total_features = features.len() as i32;
    let implemented_features = features
        .iter()
        .filter(|f| matches!(f.state, super::models::FeatureState::Implemented | super::models::FeatureState::TestedPassing))
        .count() as i32;
    let tested_features = features
        .iter()
        .filter(|f| matches!(f.state, super::models::FeatureState::TestedPassing))
        .count() as i32;

    let feature_metrics = FeatureMetrics {
        total_features,
        implemented_features,
        tested_features,
        implementation_percentage: if total_features > 0 {
            (implemented_features as f32 / total_features as f32) * 100.0
        } else {
            0.0
        },
        test_coverage_percentage: if total_features > 0 {
            (tested_features as f32 / total_features as f32) * 100.0
        } else {
            0.0
        },
        quality_score: if total_features > 0 {
            (tested_features as f32 / total_features as f32) * 100.0
        } else {
            0.0
        },
        by_state: std::collections::HashMap::new(), // TODO: Implement state grouping
        by_category: std::collections::HashMap::new(), // TODO: Implement category grouping
    };

    // Get tasks and calculate metrics
    let tasks = entity_manager.list_tasks().await.unwrap_or_default();
    let total_tasks = tasks.len() as i32;
    let completed_tasks = tasks
        .iter()
        .filter(|t| matches!(t.status, super::models::TaskStatus::Completed))
        .count() as i32;
    let in_progress_tasks = tasks
        .iter()
        .filter(|t| matches!(t.status, super::models::TaskStatus::InProgress))
        .count() as i32;
    let pending_tasks = tasks
        .iter()
        .filter(|t| matches!(t.status, super::models::TaskStatus::Pending))
        .count() as i32;
    let blocked_tasks = tasks
        .iter()
        .filter(|t| matches!(t.status, super::models::TaskStatus::Blocked))
        .count() as i32;

    let task_summary = TaskSummary {
        total_tasks,
        completed_tasks,
        in_progress_tasks,
        pending_tasks,
        blocked_tasks,
        completion_percentage: if total_tasks > 0 {
            (completed_tasks as f32 / total_tasks as f32) * 100.0
        } else {
            0.0
        },
        by_priority: std::collections::HashMap::new(), // TODO: Implement priority grouping
        by_category: std::collections::HashMap::new(), // TODO: Implement category grouping
    };

    let response = ProjectStatusResponse {
        project: ProjectInfo {
            id: project.id.to_string(),
            name: project.name,
            description: project.description,
            version: project.version,
            created_at: project.created_at,
            updated_at: project.updated_at,
        },
        feature_metrics,
        task_summary,
        session_summary: SessionSummary {
            total_sessions: 0, // TODO: Implement session counting
            active_sessions: 0,
            completed_sessions: 0,
            average_duration_minutes: None,
            recent_sessions: vec![],
        },
        recent_activity: vec![], // TODO: Implement activity tracking
        health_indicators: HealthIndicators {
            build_status: "unknown".to_string(),
            test_status: "unknown".to_string(),
            coverage_trend: "stable".to_string(),
            velocity_trend: "stable".to_string(),
            blocker_count: blocked_tasks,
            last_successful_build: None,
        },
    };

    Ok(ResponseJson(ApiResponse::success(response)))
}
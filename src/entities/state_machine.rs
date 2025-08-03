// State Machine Management for Entity Workflows

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::models::{Feature, FeatureStateTransition};
use super::{FeatureState, TaskStatus, SessionState};

/// Feature state machine transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStateMachine;

impl FeatureStateMachine {
    /// Validate if a state transition is allowed
    pub fn can_transition(from: &FeatureState, to: &FeatureState) -> bool {
        use FeatureState::*;
        
        match (from, to) {
            // From NotImplemented
            (NotImplemented, Implemented) => true,
            (NotImplemented, CriticalIssue) => true,
            
            // From Implemented 
            (Implemented, TestedPassing) => true,
            (Implemented, TestedFailing) => true,
            (Implemented, TautologicalTest) => true,
            (Implemented, CriticalIssue) => true,
            (Implemented, NotImplemented) => true, // rollback
            
            // From TestedPassing
            (TestedPassing, TestedFailing) => true, // tests started failing
            (TestedPassing, TautologicalTest) => true, // detected fake tests
            (TestedPassing, CriticalIssue) => true,
            
            // From TestedFailing
            (TestedFailing, TestedPassing) => true, // tests fixed
            (TestedFailing, TautologicalTest) => true, // detected fake tests
            (TestedFailing, Implemented) => true, // remove tests
            (TestedFailing, CriticalIssue) => true,
            
            // From TautologicalTest
            (TautologicalTest, TestedPassing) => true, // fixed tests
            (TautologicalTest, TestedFailing) => true, // real tests added
            (TautologicalTest, Implemented) => true, // removed fake tests
            (TautologicalTest, CriticalIssue) => true,
            
            // From CriticalIssue - can transition to any state after fixing
            (CriticalIssue, _) => true,
            
            // Same state (no-op)
            (state1, state2) if state1 == state2 => true,
            
            // All other transitions not allowed
            _ => false,
        }
    }
    
    /// Get next possible states from current state
    pub fn next_states(current: &FeatureState) -> Vec<FeatureState> {
        use FeatureState::*;
        
        match current {
            NotImplemented => vec![Implemented, CriticalIssue],
            Implemented => vec![
                TestedPassing, 
                TestedFailing, 
                TautologicalTest, 
                CriticalIssue, 
                NotImplemented
            ],
            TestedPassing => vec![TestedFailing, TautologicalTest, CriticalIssue],
            TestedFailing => vec![
                TestedPassing, 
                TautologicalTest, 
                Implemented, 
                CriticalIssue
            ],
            TautologicalTest => vec![
                TestedPassing, 
                TestedFailing, 
                Implemented, 
                CriticalIssue
            ],
            CriticalIssue => vec![
                NotImplemented,
                Implemented,
                TestedPassing,
                TestedFailing,
                TautologicalTest,
            ],
            // Legacy states - allow transition to modern equivalents
            Completed => vec![TestedPassing, CriticalIssue],
            InProgress => vec![Implemented, TestedPassing, TestedFailing, CriticalIssue],
            Testing => vec![TestedPassing, TestedFailing, TautologicalTest, CriticalIssue],
        }
    }
    
    /// Get state transition reason/description
    pub fn transition_reason(from: &FeatureState, to: &FeatureState) -> String {
        use FeatureState::*;
        
        match (from, to) {
            (NotImplemented, Implemented) => "Feature implemented, needs tests".to_string(),
            (Implemented, TestedPassing) => "Tests added and passing".to_string(),
            (Implemented, TestedFailing) => "Tests added but failing".to_string(),
            (Implemented, TautologicalTest) => "Fake/tautological tests detected".to_string(),
            (TestedPassing, TestedFailing) => "Tests started failing".to_string(),
            (TestedFailing, TestedPassing) => "Tests fixed and now passing".to_string(),
            (TautologicalTest, TestedPassing) => "Proper tests implemented".to_string(),
            (TautologicalTest, Implemented) => "Fake tests removed".to_string(),
            (_, CriticalIssue) => "Critical issue identified".to_string(),
            (CriticalIssue, _) => "Critical issue resolved".to_string(),
            _ => "State transition".to_string(),
        }
    }
}

/// Task status state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStateMachine;

impl TaskStateMachine {
    /// Validate if a status transition is allowed
    pub fn can_transition(from: &TaskStatus, to: &TaskStatus) -> bool {
        use TaskStatus::*;
        
        match (from, to) {
            // From Pending
            (Pending, InProgress) => true,
            (Pending, Cancelled) => true,
            
            // From InProgress
            (InProgress, Completed) => true,
            (InProgress, Blocked) => true,
            (InProgress, Pending) => true, // pause work
            (InProgress, Cancelled) => true,
            
            // From Blocked
            (Blocked, InProgress) => true, // unblocked
            (Blocked, Pending) => true, // reset
            (Blocked, Cancelled) => true,
            
            // From Completed (rare but allowed for corrections)
            (Completed, InProgress) => true, // reopen
            
            // Same status (no-op)
            (status1, status2) if status1 == status2 => true,
            
            // Cancelled is terminal
            (Cancelled, _) => false,
            
            _ => false,
        }
    }
    
    /// Get next possible statuses from current status
    pub fn next_statuses(current: &TaskStatus) -> Vec<TaskStatus> {
        use TaskStatus::*;
        
        match current {
            Pending => vec![InProgress, Cancelled],
            InProgress => vec![Completed, Blocked, Pending, Cancelled],
            Blocked => vec![InProgress, Pending, Cancelled],
            Completed => vec![InProgress], // rare but allows reopening
            Cancelled => vec![], // terminal state
        }
    }
}

/// Session state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStateMachine;

impl SessionStateMachine {
    /// Validate if a state transition is allowed
    pub fn can_transition(from: &SessionState, to: &SessionState) -> bool {
        use SessionState::*;
        
        match (from, to) {
            // From Active
            (Active, Completed) => true,
            (Active, Interrupted) => true,
            
            // Terminal states cannot transition
            (Completed, _) => false,
            (Interrupted, _) => false,
            
            // Same state (no-op)
            (state1, state2) if state1 == state2 => false, // Invalid transition
            
            // All other transitions are invalid
            _ => false,
        }
    }
}

/// Transition a feature to a new state with validation and audit
pub async fn transition_feature_state(
    pool: &SqlitePool,
    feature_id: Uuid,
    to_state: FeatureState,
    evidence: Option<String>,
    notes: Option<String>,
    triggered_by: String,
) -> Result<FeatureStateTransition> {
    // Get current feature state
    let feature: Feature = sqlx::query_as(r#"
        SELECT id, project_id, code, name, description, category, state, test_status, priority,
               implementation_notes, test_evidence, dependencies, created_at, updated_at,
               completed_at, estimated_effort, actual_effort
        FROM features WHERE id = ?
    "#)
    .bind(feature_id.to_string())
    .fetch_one(pool)
    .await?;
    
    // Validate transition
    if !FeatureStateMachine::can_transition(&feature.state, &to_state) {
        return Err(anyhow!(
            "Invalid state transition from {:?} to {:?} for feature {}",
            feature.state,
            to_state,
            feature.code
        ));
    }
    
    let now = Utc::now();
    
    // Update feature state
    let completed_at = if matches!(to_state, FeatureState::TestedPassing) {
        Some(now)
    } else {
        None
    };

    sqlx::query(r#"
        UPDATE features 
        SET state = ?, updated_at = ?, completed_at = ?, implementation_notes = ?
        WHERE id = ?
    "#)
    .bind(format!("{:?}", to_state).to_lowercase())
    .bind(now.to_rfc3339())
    .bind(completed_at.map(|dt| dt.to_rfc3339()))
    .bind(&notes)
    .bind(feature_id.to_string())
    .execute(pool)
    .await?;

    // Create transition record
    let transition_id = Uuid::new_v4();
    let transition = FeatureStateTransition {
        feature_id: feature_id.into(),
        from_state: feature.state.clone(),
        to_state: to_state.clone(),
        evidence: evidence.clone(),
        notes: notes.clone(),
        triggered_by: triggered_by.clone(),
        timestamp: now,
    };

    sqlx::query(r#"
        INSERT INTO feature_state_transitions (
            id, feature_id, from_state, to_state, evidence, notes, triggered_by, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#)
    .bind(transition_id.to_string())
    .bind(feature_id.to_string())
    .bind(format!("{:?}", feature.state).to_lowercase())
    .bind(format!("{:?}", to_state).to_lowercase())
    .bind(&evidence)
    .bind(&notes)
    .bind(&triggered_by)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(transition)
}

/// Transition a task to a new status with validation
pub async fn transition_task_status(
    pool: &SqlitePool,
    task_id: Uuid,
    to_status: TaskStatus,
    evidence: Option<String>,
    session_id: Option<Uuid>,
) -> Result<()> {
    // Get current task status
    let current_status: String = sqlx::query_scalar(
        "SELECT status FROM tasks WHERE id = ?"
    )
    .bind(task_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let current_task_status = parse_task_status(&current_status)?;
    
    // Validate transition
    if !TaskStateMachine::can_transition(&current_task_status, &to_status) {
        return Err(anyhow!(
            "Invalid status transition from {:?} to {:?}",
            current_task_status,
            to_status
        ));
    }
    
    let now = Utc::now();
    
    let (started_at, completed_at) = match to_status {
        TaskStatus::InProgress => (Some(now), None),
        TaskStatus::Completed => (None, Some(now)),
        _ => (None, None),
    };

    sqlx::query(r#"
        UPDATE tasks 
        SET status = ?, evidence = ?, session_id = ?, updated_at = ?,
            started_at = COALESCE(started_at, ?), completed_at = ?
        WHERE id = ?
    "#)
    .bind(format!("{:?}", to_status).to_lowercase())
    .bind(&evidence)
    .bind(session_id.map(|id| id.to_string()))
    .bind(now.to_rfc3339())
    .bind(started_at.map(|dt| dt.to_rfc3339()))
    .bind(completed_at.map(|dt| dt.to_rfc3339()))
    .bind(task_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Auto-transition features based on task completion
pub async fn auto_transition_features_from_task(
    pool: &SqlitePool,
    task_id: Uuid,
    task_status: TaskStatus,
) -> Result<Vec<FeatureStateTransition>> {
    // Get linked features for this task
    let feature_ids_json: Option<String> = sqlx::query_scalar(
        "SELECT feature_ids FROM tasks WHERE id = ?"
    )
    .bind(task_id.to_string())
    .fetch_optional(pool)
    .await?;
    
    let mut transitions = Vec::new();
    
    if let Some(json) = feature_ids_json {
        let feature_id_strings: Vec<String> = serde_json::from_str(&json)?;
        
        for feature_id_str in feature_id_strings {
            let feature_id = Uuid::parse_str(&feature_id_str)?;
            
            // Get current feature state
            let current_state: String = sqlx::query_scalar(
                "SELECT state FROM features WHERE id = ?"
            )
            .bind(feature_id.to_string())
            .fetch_one(pool)
            .await?;
            
            let feature_state = parse_feature_state(&current_state)?;
            
            // Determine if feature should transition based on task completion
            let new_state = match (&feature_state, &task_status) {
                // Task implementing feature is completed -> feature is implemented
                (FeatureState::NotImplemented, TaskStatus::Completed) => {
                    Some(FeatureState::Implemented)
                }
                _ => None,
            };
            
            if let Some(to_state) = new_state {
                let transition = transition_feature_state(
                    pool,
                    feature_id,
                    to_state,
                    Some(format!("Task {} completed", task_id)),
                    Some("Auto-transition from task completion".to_string()),
                    "task_completion".to_string(),
                ).await?;
                
                transitions.push(transition);
            }
        }
    }
    
    Ok(transitions)
}

/// Auto-transition features based on test results
pub async fn auto_transition_features_from_test(
    pool: &SqlitePool,
    feature_id: Uuid,
    test_passed: bool,
    is_tautological: bool,
) -> Result<Option<FeatureStateTransition>> {
    // Get current feature state
    let current_state: String = sqlx::query_scalar(
        "SELECT state FROM features WHERE id = ?"
    )
    .bind(feature_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let feature_state = parse_feature_state(&current_state)?;
    
    // Determine new state based on test results
    let new_state = match (&feature_state, test_passed, is_tautological) {
        // Tautological test detected
        (_, _, true) => Some(FeatureState::TautologicalTest),
        
        // Test passed
        (FeatureState::Implemented, true, false) => Some(FeatureState::TestedPassing),
        (FeatureState::TestedFailing, true, false) => Some(FeatureState::TestedPassing),
        
        // Test failed
        (FeatureState::Implemented, false, false) => Some(FeatureState::TestedFailing),
        (FeatureState::TestedPassing, false, false) => Some(FeatureState::TestedFailing),
        
        _ => None,
    };
    
    if let Some(to_state) = new_state {
        let evidence = if is_tautological {
            Some("Tautological test detected".to_string())
        } else {
            Some(format!("Test result: {}", if test_passed { "PASSED" } else { "FAILED" }))
        };
        
        let transition = transition_feature_state(
            pool,
            feature_id,
            to_state,
            evidence,
            Some("Auto-transition from test results".to_string()),
            "test_result".to_string(),
        ).await?;
        
        Ok(Some(transition))
    } else {
        Ok(None)
    }
}

/// Get state transition history for a feature
pub async fn get_feature_transition_history(
    pool: &SqlitePool,
    feature_id: Uuid,
) -> Result<Vec<FeatureStateTransition>> {
    let transitions = sqlx::query_as::<_, FeatureStateTransition>(r#"
        SELECT feature_id, from_state, to_state, evidence, notes, triggered_by, timestamp
        FROM feature_state_transitions
        WHERE feature_id = ?
        ORDER BY timestamp DESC
    "#)
    .bind(feature_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(transitions)
}

/// Parse feature state from string
fn parse_feature_state(state_str: &str) -> Result<FeatureState> {
    match state_str {
        "not_implemented" => Ok(FeatureState::NotImplemented),
        "implemented" => Ok(FeatureState::Implemented),
        "tested_passing" => Ok(FeatureState::TestedPassing),
        "tested_failing" => Ok(FeatureState::TestedFailing),
        "tautological_test" => Ok(FeatureState::TautologicalTest),
        "critical_issue" => Ok(FeatureState::CriticalIssue),
        _ => Err(anyhow!("Unknown feature state: {}", state_str)),
    }
}

/// Parse task status from string
fn parse_task_status(status_str: &str) -> Result<TaskStatus> {
    match status_str {
        "pending" => Ok(TaskStatus::Pending),
        "in_progress" => Ok(TaskStatus::InProgress),
        "blocked" => Ok(TaskStatus::Blocked),
        "completed" => Ok(TaskStatus::Completed),
        "cancelled" => Ok(TaskStatus::Cancelled),
        _ => Err(anyhow!("Unknown task status: {}", status_str)),
    }
}

/// Validate all feature states in project for consistency
pub async fn validate_feature_states(pool: &SqlitePool, project_id: Uuid) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    
    // Get all features with their test results (using query_as instead of query! macro)
    let features = sqlx::query_as::<_, (String, String, String, String, i64, i64, i64)>(r#"
        SELECT f.id, f.code, f.state, f.test_status,
               COUNT(t.id) as test_count,
               COALESCE(SUM(CASE WHEN t.passed = 1 THEN 1 ELSE 0 END), 0) as passed_tests,
               COALESCE(SUM(CASE WHEN t.is_tautological = 1 THEN 1 ELSE 0 END), 0) as tautological_tests
        FROM features f
        LEFT JOIN tests t ON f.id = t.feature_id
        WHERE f.project_id = ?
        GROUP BY f.id, f.code, f.state, f.test_status
    "#)
    .bind(project_id.to_string())
    .fetch_all(pool)
    .await?;
    
    for feature in features {
        let (_id, code, state_str, _test_status, test_count, passed_tests, tautological_tests) = feature;
        let state = parse_feature_state(&state_str)?;
        
        // Validate state consistency with test results
        match state {
            FeatureState::TestedPassing => {
                if test_count == 0 {
                    issues.push(format!(
                        "Feature {} is marked as TestedPassing but has no tests",
                        code
                    ));
                } else if passed_tests == 0 {
                    issues.push(format!(
                        "Feature {} is marked as TestedPassing but has no passing tests",
                        code
                    ));
                }
            }
            FeatureState::TestedFailing => {
                if test_count == 0 {
                    issues.push(format!(
                        "Feature {} is marked as TestedFailing but has no tests",
                        code
                    ));
                } else if passed_tests == test_count {
                    issues.push(format!(
                        "Feature {} is marked as TestedFailing but all tests pass",
                        code
                    ));
                }
            }
            FeatureState::TautologicalTest => {
                if tautological_tests == 0 {
                    issues.push(format!(
                        "Feature {} is marked as TautologicalTest but has no tautological tests",
                        code
                    ));
                }
            }
            FeatureState::Implemented => {
                if test_count > 0 {
                    issues.push(format!(
                        "Feature {} is marked as Implemented but has tests - should be TestedPassing/TestedFailing",
                        code
                    ));
                }
            }
            _ => {}
        }
    }
    
    Ok(issues)
}
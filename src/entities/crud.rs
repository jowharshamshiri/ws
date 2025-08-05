// CRUD Operations for Workspace Entities

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::models::*;
use super::{FeatureState, Priority, TaskStatus, SessionState, DirectiveCategory, NoteType, EntityType};
use super::audit;

/// Project CRUD operations
pub mod projects {
    use super::*;

    pub async fn create(
        pool: &SqlitePool,
        name: String,
        description: Option<String>,
        repository_url: Option<String>,
    ) -> Result<Project> {
        let id = format!("proj-{}", uuid::Uuid::new_v4().to_string()[..8].to_lowercase());
        let now = Utc::now();
        
        let project = Project {
            id: id.clone(),
            name: name.clone(),
            description: description.clone(),
            repository_url: repository_url.clone(),
            version: "0.1.0".to_string(),
            created_at: now,
            updated_at: now,
            archived: false,
            metadata: None,
        };

        sqlx::query(r#"
            INSERT INTO projects (id, name, description, repository_url, version, created_at, updated_at, archived)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&name)
        .bind(&description)
        .bind(&repository_url)
        .bind(&project.version)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(false)
        .execute(pool)
        .await?;

        Ok(project)
    }

    pub async fn get(pool: &SqlitePool, id: &str) -> Result<Project> {
        let project = sqlx::query_as::<_, Project>(r#"
            SELECT id, name, description, repository_url, version, created_at, updated_at, archived, metadata
            FROM projects WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(project)
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Project>> {
        let projects = sqlx::query_as::<_, Project>(r#"
            SELECT id, name, description, repository_url, version, created_at, updated_at, archived, metadata
            FROM projects WHERE archived = FALSE
            ORDER BY created_at DESC
        "#)
        .fetch_all(pool)
        .await?;

        Ok(projects)
    }

    pub async fn update_version(pool: &SqlitePool, id: Uuid, version: String) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query("UPDATE projects SET version = ?, updated_at = ? WHERE id = ?")
            .bind(&version)
            .bind(now.to_rfc3339())
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Feature CRUD operations with state machine
pub mod features {
    use super::*;

    /// List all features
    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Feature>> {
        let features = sqlx::query_as::<_, Feature>(r#"
            SELECT * FROM features ORDER BY created_at DESC
        "#)
        .fetch_all(pool)
        .await?;
        
        Ok(features)
    }

    /// Get feature by ID
    pub async fn get(pool: &SqlitePool, id: &str) -> Result<Feature> {
        let feature = sqlx::query_as::<_, Feature>(r#"
            SELECT * FROM features WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(pool)
        .await?;
        
        Ok(feature)
    }

    /// Update feature with audit trail recording
    pub async fn update(pool: &SqlitePool, feature: Feature) -> Result<Feature> {
        // Get current feature for audit trail comparison
        let old_feature = get(pool, &feature.id).await?;
        let now = Utc::now();
        
        sqlx::query(r#"
            UPDATE features 
            SET name = ?, description = ?, category = ?, state = ?, test_status = ?, 
                priority = ?, implementation_notes = ?, updated_at = ?
            WHERE id = ?
        "#)
        .bind(&feature.name)
        .bind(&feature.description)
        .bind(&feature.category)
        .bind(&feature.state)
        .bind(&feature.test_status)  
        .bind(&feature.priority)
        .bind(&feature.implementation_notes)
        .bind(now.to_rfc3339())
        .bind(feature.id.to_string())
        .execute(pool)
        .await?;
        
        // Record audit trail for changes
        let _audit_records = audit::record_entity_changes(
            pool,
            &feature.id,
            EntityType::Feature,
            &feature.project_id,
            &old_feature,
            &feature,
            "update".to_string(),
            "system".to_string(), // TODO: Get actual user/trigger source
            None, // TODO: Get current session ID
            None, // TODO: Add change reason parameter
        ).await?;
        
        // Return updated feature
        get(pool, &feature.id).await
    }

    /// Update feature with explicit audit context
    pub async fn update_with_audit(
        pool: &SqlitePool, 
        feature: Feature,
        triggered_by: String,
        session_id: Option<String>,
        change_reason: Option<String>,
    ) -> Result<Feature> {
        // Get current feature for audit trail comparison
        let old_feature = get(pool, &feature.id).await?;
        let now = Utc::now();
        
        sqlx::query(r#"
            UPDATE features 
            SET name = ?, description = ?, category = ?, state = ?, test_status = ?, 
                priority = ?, implementation_notes = ?, updated_at = ?
            WHERE id = ?
        "#)
        .bind(&feature.name)
        .bind(&feature.description)
        .bind(&feature.category)
        .bind(&feature.state)
        .bind(&feature.test_status)
        .bind(&feature.priority)
        .bind(&feature.implementation_notes)
        .bind(now.to_rfc3339())  
        .bind(feature.id.to_string())
        .execute(pool)
        .await?;
        
        // Record audit trail for changes
        let _audit_records = audit::record_entity_changes(
            pool,
            &feature.id,
            EntityType::Feature,
            &feature.project_id,
            &old_feature,
            &feature,
            "update".to_string(),
            triggered_by,
            session_id,
            change_reason,
        ).await?;
        
        // Return updated feature
        get(pool, &feature.id).await
    }

    pub async fn create(
        pool: &SqlitePool,
        project_id: &str,
        name: String,
        description: String,
        category: Option<String>,
        priority: Priority,
    ) -> Result<Feature> {
        // ID will be generated as feature code
        let now = Utc::now();
        
        // Generate next feature code
        let code = generate_next_feature_code(pool, project_id).await?;
        
        let feature = Feature {
            id: code.clone(),
            project_id: project_id.to_string(),
            code: code.clone(),
            name: name.clone(),
            description: description.clone(),
            category: category.clone(),
            state: FeatureState::NotImplemented,
            test_status: "not_tested".to_string(),
            priority: priority.clone(),
            implementation_notes: None,
            test_evidence: None,
            dependencies: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            estimated_effort: None,
            actual_effort: None,
            metadata: None,
        };

        sqlx::query(r#"
            INSERT INTO features (
                id, project_id, code, name, description, category, state, test_status, priority,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&code)
        .bind(project_id)
        .bind(&code)
        .bind(&name)
        .bind(&description)
        .bind(&category)
        .bind("not_implemented")
        .bind("not_tested")
        .bind(&priority)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        // Record creation audit trail
        let entity_data = serde_json::to_string(&feature)?;
        let _audit_record = audit::record_operation_audit(
            pool,
            &code,
            EntityType::Feature,
            project_id,
            "create".to_string(),
            "system".to_string(), // TODO: Get actual user/trigger source
            None, // TODO: Get current session ID
            Some("Feature created via CRUD operation".to_string()),
            Some(entity_data),
        ).await?;

        Ok(feature)
    }


    pub async fn get_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Feature>> {
        let features = sqlx::query_as::<_, Feature>(r#"
            SELECT id, project_id, code, name, description, category, state, test_status, priority,
                   implementation_notes, test_evidence, dependencies, created_at, updated_at,
                   completed_at, estimated_effort, actual_effort
            FROM features WHERE project_id = ?
            ORDER BY code
        "#)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(features)
    }

    pub async fn update_state(
        pool: &SqlitePool,
        id: &str,
        new_state: FeatureState,
        evidence: Option<String>,
        notes: Option<String>,
        triggered_by: String,
    ) -> Result<()> {
        let now = Utc::now();
        
        // Get current state for transition tracking
        let current = get(pool, id).await?;
        
        // Update feature state
        let completed_at = if matches!(new_state, FeatureState::TestedPassing) {
            Some(now)
        } else {
            None
        };

        sqlx::query(r#"
            UPDATE features 
            SET state = ?, updated_at = ?, completed_at = ?, implementation_notes = ?
            WHERE id = ?
        "#)
        .bind(&new_state)
        .bind(now.to_rfc3339())
        .bind(completed_at.map(|dt| dt.to_rfc3339()))
        .bind(&notes)
        .bind(id)
        .execute(pool)
        .await?;

        // Record state transition
        record_state_transition(
            pool,
            id,
            current.state,
            new_state,
            evidence,
            notes,
            triggered_by,
        ).await?;

        Ok(())
    }

    pub async fn update_test_status(
        pool: &SqlitePool,
        id: Uuid,
        test_status: String,
        test_evidence: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query(r#"
            UPDATE features 
            SET test_status = ?, test_evidence = ?, updated_at = ?
            WHERE id = ?
        "#)
        .bind(&test_status)
        .bind(&test_evidence)
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn generate_next_feature_code(pool: &SqlitePool, project_id: &str) -> Result<String> {
        let max_code: Option<String> = sqlx::query_scalar(r#"
            SELECT code FROM features 
            WHERE project_id = ? AND code LIKE 'F%'
            ORDER BY CAST(SUBSTR(code, 2) AS INTEGER) DESC 
            LIMIT 1
        "#)
        .bind(project_id)
        .fetch_optional(pool)
        .await?;

        let next_num = if let Some(code) = max_code {
            let num_str = code.trim_start_matches('F');
            num_str.parse::<i32>().unwrap_or(0) + 1
        } else {
            1
        };

        Ok(format!("F{:04}", next_num))
    }

    async fn record_state_transition(
        pool: &SqlitePool,
        feature_id: &str,
        from_state: FeatureState,
        to_state: FeatureState,
        evidence: Option<String>,
        notes: Option<String>,
        triggered_by: String,
    ) -> Result<()> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(r#"
            INSERT INTO feature_state_transitions (
                id, feature_id, from_state, to_state, evidence, notes, triggered_by, timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(id)
        .bind(feature_id)
        .bind(&from_state)
        .bind(&to_state)
        .bind(&evidence)
        .bind(&notes)
        .bind(&triggered_by)
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_state_transitions(pool: &SqlitePool, feature_id: Uuid) -> Result<Vec<FeatureStateTransition>> {
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
}

/// Task CRUD operations with feature integration
pub mod tasks {
    use super::*;

    /// List all tasks
    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(r#"
            SELECT * FROM tasks ORDER BY created_at DESC
        "#)
        .fetch_all(pool)
        .await?;
        
        Ok(tasks)
    }

    /// Get task by ID
    pub async fn get(pool: &SqlitePool, id: &str) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(r#"
            SELECT * FROM tasks WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(pool)
        .await?;
        
        Ok(task)
    }

    pub async fn create(
        pool: &SqlitePool,
        project_id: &str,
        title: String,
        description: String,
        category: String,
        priority: Priority,
        feature_ids: Option<Vec<String>>,
    ) -> Result<Task> {
        let id = format!("task-{}", uuid::Uuid::new_v4().to_string()[..12].to_lowercase());
        let now = Utc::now();
        
        // Generate next task code
        let code = generate_next_task_code(pool, project_id).await?;
        
        let feature_ids_json = if let Some(ids) = &feature_ids {
            Some(serde_json::to_string(ids)?)
        } else {
            None
        };

        let task = Task {
            id: id.clone(),
            project_id: project_id.to_string(),
            code: code.clone(),
            title: title.clone(),
            description: description.clone(),
            category: category.clone(),
            status: TaskStatus::Pending,
            priority: priority.clone(),
            feature_ids: feature_ids_json.clone(),
            depends_on: None,
            acceptance_criteria: None,
            validation_steps: None,
            evidence: None,
            session_id: None,
            assigned_to: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            estimated_effort: None,
            actual_effort: None,
            tags: None,
            metadata: None,
        };

        sqlx::query(r#"
            INSERT INTO tasks (
                id, project_id, code, title, description, category, status, priority,
                feature_ids, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(project_id)
        .bind(&code)
        .bind(&title)
        .bind(&description)
        .bind(&category)
        .bind("pending")
        .bind(&priority)
        .bind(&feature_ids_json)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        Ok(task)
    }


    pub async fn get_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(r#"
            SELECT id, project_id, code, title, description, category, status, priority,
                   feature_ids, depends_on, acceptance_criteria, validation_steps, evidence,
                   session_id, assigned_to, created_at, updated_at, started_at, completed_at,
                   estimated_effort, actual_effort, tags
            FROM tasks WHERE project_id = ?
            ORDER BY priority DESC, created_at ASC
        "#)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(tasks)
    }

    /// Update task
    pub async fn update(pool: &SqlitePool, task: Task) -> Result<Task> {
        let now = Utc::now();
        
        let (started_at, completed_at) = match task.status {
            TaskStatus::InProgress => (task.started_at.or(Some(now)), task.completed_at),
            TaskStatus::Completed => (task.started_at, Some(task.completed_at.unwrap_or(now))),
            _ => (task.started_at, task.completed_at),
        };

        sqlx::query(r#"
            UPDATE tasks 
            SET title = ?, description = ?, category = ?, status = ?, priority = ?,
                feature_ids = ?, depends_on = ?, acceptance_criteria = ?, validation_steps = ?,
                evidence = ?, session_id = ?, assigned_to = ?, updated_at = ?,
                started_at = ?, completed_at = ?, estimated_effort = ?, actual_effort = ?, tags = ?
            WHERE id = ?
        "#)
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.category)
        .bind(&task.status)
        .bind(&task.priority)
        .bind(task.feature_ids.as_ref().and_then(|ids| serde_json::to_string(ids).ok()))
        .bind(task.depends_on.as_ref().and_then(|ids| serde_json::to_string(ids).ok()))
        .bind(&task.acceptance_criteria)
        .bind(&task.validation_steps)
        .bind(&task.evidence)
        .bind(task.session_id.as_ref().map(|id| id.to_string()))
        .bind(&task.assigned_to)
        .bind(now.to_rfc3339())
        .bind(started_at.map(|dt| dt.to_rfc3339()))
        .bind(completed_at.map(|dt| dt.to_rfc3339()))
        .bind(&task.estimated_effort)
        .bind(&task.actual_effort)
        .bind(task.tags.as_ref().and_then(|tags| serde_json::to_string(tags).ok()))
        .bind(task.id.to_string())
        .execute(pool)
        .await?;

        let updated_task = Task {
            updated_at: now,
            started_at,
            completed_at,
            ..task
        };

        Ok(updated_task)
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: &str,
        new_status: TaskStatus,
        evidence: Option<String>,
        session_id: Option<Uuid>,
    ) -> Result<()> {
        let now = Utc::now();
        
        let (started_at, completed_at) = match new_status {
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
        .bind(&new_status)
        .bind(&evidence)
        .bind(session_id.map(|id| id.to_string()))
        .bind(now.to_rfc3339())
        .bind(started_at.map(|dt| dt.to_rfc3339()))
        .bind(completed_at.map(|dt| dt.to_rfc3339()))
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_feature(pool: &SqlitePool, feature_id: &str) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(r#"
            SELECT id, project_id, code, title, description, category, status, priority,
                   feature_ids, depends_on, acceptance_criteria, validation_steps, evidence,
                   session_id, assigned_to, created_at, updated_at, started_at, completed_at,
                   estimated_effort, actual_effort, tags
            FROM tasks 
            WHERE feature_ids LIKE '%' || ? || '%'
            ORDER BY priority DESC, created_at ASC
        "#)
        .bind(feature_id)
        .fetch_all(pool)
        .await?;

        Ok(tasks)
    }

    async fn generate_next_task_code(pool: &SqlitePool, project_id: &str) -> Result<String> {
        let max_code: Option<String> = sqlx::query_scalar(r#"
            SELECT code FROM tasks 
            WHERE project_id = ? AND code LIKE 'TASK-%'
            ORDER BY CAST(SUBSTR(code, 6) AS INTEGER) DESC 
            LIMIT 1
        "#)
        .bind(project_id)
        .fetch_optional(pool)
        .await?;

        let next_num = if let Some(code) = max_code {
            let num_str = code.trim_start_matches("TASK-");
            num_str.parse::<i32>().unwrap_or(0) + 1
        } else {
            1
        };

        Ok(format!("TASK-{:03}", next_num))
    }
}

/// Session metrics CRUD operations
pub mod session_metrics {
    use super::*;
    use sqlx::Row;

    pub async fn store(pool: &SqlitePool, metrics: &crate::mcp_server::SessionMetrics) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query(r#"
            INSERT INTO session_metrics (
                id, session_id, session_duration_seconds, total_messages, tool_calls,
                context_usage_tokens, average_response_time_ms, peak_response_time_ms,
                total_tool_calls, total_response_time_ms, context_used, session_duration_ms,
                features_created, features_updated, tasks_created, tasks_completed,
                files_modified, issues_resolved, timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&metrics.session_id)
        .bind(metrics.session_duration_seconds as i64)
        .bind(metrics.total_messages as i64)
        .bind(metrics.tool_calls as i64)
        .bind(metrics.context_usage_tokens as i64)
        .bind(metrics.average_response_time_ms as i64)
        .bind(metrics.peak_response_time_ms as i64)
        .bind(metrics.total_tool_calls as i64)
        .bind(metrics.total_response_time_ms as i64)
        .bind(metrics.context_used as i64)
        .bind(metrics.session_duration_ms as i64)
        .bind(metrics.features_created as i64)
        .bind(metrics.features_updated as i64)
        .bind(metrics.tasks_created as i64)
        .bind(metrics.tasks_completed as i64)
        .bind(metrics.files_modified as i64)
        .bind(metrics.issues_resolved as i64)
        .bind(metrics.timestamp.to_rfc3339())
        .execute(pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_by_session_id(pool: &SqlitePool, session_id: &str) -> Result<Vec<crate::mcp_server::SessionMetrics>> {
        let rows = sqlx::query(r#"
            SELECT session_id, session_duration_seconds, total_messages, tool_calls,
                   context_usage_tokens, average_response_time_ms, peak_response_time_ms,
                   total_tool_calls, total_response_time_ms, context_used, session_duration_ms,
                   features_created, features_updated, tasks_created, tasks_completed,
                   files_modified, issues_resolved, timestamp
            FROM session_metrics WHERE session_id = ?
            ORDER BY timestamp DESC
        "#)
        .bind(session_id)
        .fetch_all(pool)
        .await?;

        let mut metrics = Vec::new();
        for row in rows {
            let session_id: String = row.get("session_id");
            let session_duration_seconds: i64 = row.get("session_duration_seconds");
            let total_messages: i64 = row.get("total_messages");
            let tool_calls: i64 = row.get("tool_calls");
            let context_usage_tokens: i64 = row.get("context_usage_tokens");
            let average_response_time_ms: i64 = row.get("average_response_time_ms");
            let peak_response_time_ms: i64 = row.get("peak_response_time_ms");
            let total_tool_calls: i64 = row.get("total_tool_calls");
            let total_response_time_ms: i64 = row.get("total_response_time_ms");
            let context_used: i64 = row.get("context_used");
            let session_duration_ms: i64 = row.get("session_duration_ms");
            let features_created: i64 = row.get("features_created");
            let features_updated: i64 = row.get("features_updated");
            let tasks_created: i64 = row.get("tasks_created");
            let tasks_completed: i64 = row.get("tasks_completed");
            let files_modified: i64 = row.get("files_modified");
            let issues_resolved: i64 = row.get("issues_resolved");
            let timestamp_str: String = row.get("timestamp");
            
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)?
                .with_timezone(&chrono::Utc);
            
            metrics.push(crate::mcp_server::SessionMetrics {
                session_id,
                session_duration_seconds: session_duration_seconds as u64,
                total_messages: total_messages as u64,
                tool_calls: tool_calls as u64,
                context_usage_tokens: context_usage_tokens as u64,
                average_response_time_ms: average_response_time_ms as u64,
                peak_response_time_ms: peak_response_time_ms as u64,
                total_tool_calls: total_tool_calls as u64,
                total_response_time_ms: total_response_time_ms as u64,
                context_used: context_used as u64,
                session_duration_ms: session_duration_ms as u64,
                features_created: features_created as u64,
                features_updated: features_updated as u64,
                tasks_created: tasks_created as u64,
                tasks_completed: tasks_completed as u64,
                files_modified: files_modified as u64,
                issues_resolved: issues_resolved as u64,
                timestamp,
            });
        }
        
        Ok(metrics)
    }

    pub async fn get_latest_by_session_id(pool: &SqlitePool, session_id: &str) -> Result<Option<crate::mcp_server::SessionMetrics>> {
        let metrics = get_by_session_id(pool, session_id).await?;
        Ok(metrics.into_iter().next())
    }
}

/// Session CRUD operations
pub mod sessions {
    use super::*;

    /// List all sessions
    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, Session>(r#"
            SELECT * FROM sessions ORDER BY created_at DESC
        "#)
        .fetch_all(pool)
        .await?;
        
        Ok(sessions)
    }

    pub async fn create(
        pool: &SqlitePool,
        project_id: &str,
        title: String,
        description: Option<String>,
    ) -> Result<Session> {
        let id = generate_next_session_id(pool, project_id).await?;
        let now = Utc::now();
        
        let session = Session {
            id: id.clone(),
            project_id: project_id.to_string(),
            title: title.clone(),
            description: description.clone(),
            state: SessionState::Active,
            started_at: now,
            ended_at: None,
            summary: None,
            achievements: None,
            files_modified: None,
            features_worked: None,
            tasks_completed: None,
            next_priority: None,
            reminder: None,
            validation_evidence: None,
            context_remaining: None,
            created_at: now,
            updated_at: now,
            metadata: None,
        };

        sqlx::query(r#"
            INSERT INTO sessions (
                id, project_id, title, description, state, started_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(project_id)
        .bind(&title)
        .bind(&description)
        .bind("active")
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        Ok(session)
    }

    async fn generate_next_session_id(pool: &SqlitePool, project_id: &str) -> Result<String> {
        let max_id: Option<String> = sqlx::query_scalar(r#"
            SELECT id FROM sessions 
            WHERE project_id = ? AND id LIKE 'SESS-%'
            ORDER BY CAST(SUBSTR(id, 6) AS INTEGER) DESC 
            LIMIT 1
        "#)
        .bind(project_id)
        .fetch_optional(pool)
        .await?;

        let next_num = if let Some(id) = max_id {
            let num_str = id.trim_start_matches("SESS-");
            num_str.parse::<i32>().unwrap_or(0) + 1
        } else {
            1
        };

        Ok(format!("SESS-{:04}", next_num))
    }

    pub async fn get(pool: &SqlitePool, id: &str) -> Result<Session> {
        let session = sqlx::query_as::<_, Session>(r#"
            SELECT id, project_id, title, description, state, started_at, ended_at, summary,
                   achievements, files_modified, features_worked, tasks_completed, next_priority,
                   reminder, validation_evidence, context_remaining, created_at, updated_at
            FROM sessions WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(session)
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Session> {
        get(pool, id).await
    }

    pub async fn get_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, Session>(r#"
            SELECT id, project_id, title, description, state, started_at, ended_at, summary,
                   achievements, files_modified, features_worked, tasks_completed, next_priority,
                   reminder, validation_evidence, context_remaining, created_at, updated_at, metadata
            FROM sessions WHERE project_id = ?
            ORDER BY started_at DESC
        "#)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(sessions)
    }

    pub async fn update(pool: &SqlitePool, session: Session) -> Result<Session> {
        let updated_at = Utc::now();
        
        sqlx::query(r#"
            UPDATE sessions 
            SET title = ?, description = ?, state = ?, ended_at = ?, summary = ?,
                achievements = ?, files_modified = ?, features_worked = ?, tasks_completed = ?,
                next_priority = ?, reminder = ?, validation_evidence = ?, context_remaining = ?,
                metadata = ?, updated_at = ?
            WHERE id = ?
        "#)
        .bind(&session.title)
        .bind(&session.description)
        .bind(session.state.as_str())
        .bind(session.ended_at.map(|dt| dt.to_rfc3339()))
        .bind(&session.summary)
        .bind(&session.achievements)
        .bind(&session.files_modified)
        .bind(&session.features_worked)
        .bind(&session.tasks_completed)
        .bind(&session.next_priority)
        .bind(&session.reminder)
        .bind(&session.validation_evidence)
        .bind(session.context_remaining)
        .bind(&session.metadata)
        .bind(updated_at.to_rfc3339())
        .bind(&session.id)
        .execute(pool)
        .await?;

        // Return updated session
        let mut updated_session = session;
        updated_session.updated_at = updated_at;
        Ok(updated_session)
    }

    pub async fn get_active(pool: &SqlitePool, project_id: Uuid) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(r#"
            SELECT id, project_id, title, description, state, started_at, ended_at, summary,
                   achievements, files_modified, features_worked, tasks_completed, next_priority,
                   reminder, validation_evidence, context_remaining, created_at, updated_at
            FROM sessions 
            WHERE project_id = ? AND state = 'active'
            ORDER BY started_at DESC
            LIMIT 1
        "#)
        .bind(project_id)
        .fetch_optional(pool)
        .await?;

        Ok(session)
    }

    pub async fn end_session(
        pool: &SqlitePool,
        id: Uuid,
        summary: Option<String>,
        achievements: Option<Vec<String>>,
        features_worked: Option<Vec<Uuid>>,
        tasks_completed: Option<Vec<Uuid>>,
        reminder: Option<String>,
        validation_evidence: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();
        
        let achievements_json = achievements.map(|a| serde_json::to_string(&a)).transpose()?;
        let features_json = features_worked.map(|f| {
            let ids: Vec<String> = f.iter().map(|id| id.to_string()).collect();
            serde_json::to_string(&ids)
        }).transpose()?;
        let tasks_json = tasks_completed.map(|t| {
            let ids: Vec<String> = t.iter().map(|id| id.to_string()).collect();
            serde_json::to_string(&ids)
        }).transpose()?;

        sqlx::query(r#"
            UPDATE sessions 
            SET state = 'completed', ended_at = ?, summary = ?, achievements = ?,
                features_worked = ?, tasks_completed = ?, reminder = ?, validation_evidence = ?,
                updated_at = ?
            WHERE id = ?
        "#)
        .bind(now.to_rfc3339())
        .bind(&summary)
        .bind(&achievements_json)
        .bind(&features_json)
        .bind(&tasks_json)
        .bind(&reminder)
        .bind(&validation_evidence)
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_context_remaining(
        pool: &SqlitePool,
        id: Uuid,
        context_remaining: f64,
    ) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query("UPDATE sessions SET context_remaining = ?, updated_at = ? WHERE id = ?")
            .bind(context_remaining)
            .bind(now.to_rfc3339())
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Directive CRUD operations
pub mod directives {
    use super::*;

    pub async fn create(
        pool: &SqlitePool,
        project_id: &str,
        title: String,
        rule: String,
        category: DirectiveCategory,
        priority: Priority,
        context: Option<String>,
    ) -> Result<Directive> {
        let id = format!("dir-{}", uuid::Uuid::new_v4().to_string()[..12].to_lowercase());
        let now = Utc::now();
        
        // Generate next directive code
        let code = generate_next_directive_code(pool, project_id, &category).await?;
        
        let directive = Directive {
            id: id.clone(),
            project_id: project_id.to_string(),
            code: code.clone(),
            title: title.clone(),
            rule: rule.clone(),
            category: category.clone(),
            priority: priority.clone(),
            context: context.clone(),
            rationale: None,
            examples: None,
            violations: None,
            override_behavior: None,
            active: true,
            compliance_checked: None,
            created_at: now,
            updated_at: now,
            archived_at: None,
        };

        sqlx::query(r#"
            INSERT INTO directives (
                id, project_id, code, title, rule, category, priority, context, active,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(project_id)
        .bind(&code)
        .bind(&title)
        .bind(&rule)
        .bind(&category)
        .bind(&priority)
        .bind(&context)
        .bind(true)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        Ok(directive)
    }

    pub async fn get(pool: &SqlitePool, id: &str) -> Result<Directive> {
        let directive = sqlx::query_as::<_, Directive>(r#"
            SELECT id, project_id, code, title, rule, category, priority, context, rationale,
                   examples, violations, override_behavior, active, compliance_checked,
                   created_at, updated_at, archived_at
            FROM directives WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(directive)
    }

    pub async fn get_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Directive>> {
        let directives = sqlx::query_as::<_, Directive>(r#"
            SELECT id, project_id, code, title, rule, category, priority, context, rationale,
                   examples, violations, override_behavior, active, compliance_checked,
                   created_at, updated_at, archived_at
            FROM directives WHERE project_id = ? AND archived_at IS NULL
            ORDER BY priority DESC, category, code
        "#)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(directives)
    }

    pub async fn get_active(pool: &SqlitePool, project_id: &str) -> Result<Vec<Directive>> {
        let directives = sqlx::query_as::<_, Directive>(r#"
            SELECT id, project_id, code, title, rule, category, priority, context, rationale,
                   examples, violations, override_behavior, active, compliance_checked,
                   created_at, updated_at, archived_at
            FROM directives 
            WHERE project_id = ? AND active = TRUE AND archived_at IS NULL
            ORDER BY priority DESC, category, code
        "#)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(directives)
    }

    pub async fn update(pool: &SqlitePool, directive: Directive) -> Result<Directive> {
        let now = Utc::now();
        
        sqlx::query(r#"
            UPDATE directives SET
                title = ?, rule = ?, category = ?, priority = ?, context = ?,
                rationale = ?, examples = ?, violations = ?, override_behavior = ?,
                active = ?, updated_at = ?
            WHERE id = ?
        "#)
        .bind(&directive.title)
        .bind(&directive.rule)
        .bind(&directive.category)
        .bind(&directive.priority)
        .bind(&directive.context)
        .bind(&directive.rationale)
        .bind(&directive.examples)
        .bind(&directive.violations)
        .bind(&directive.override_behavior)
        .bind(directive.active)
        .bind(now.to_rfc3339())
        .bind(&directive.id)
        .execute(pool)
        .await?;

        // Return updated directive
        get(pool, &directive.id).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query(r#"
            UPDATE directives SET archived_at = ? WHERE id = ?
        "#)
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn generate_next_directive_code(
        pool: &SqlitePool, 
        project_id: &str, 
        category: &DirectiveCategory
    ) -> Result<String> {
        let prefix = match category {
            DirectiveCategory::Development => "DEV",
            DirectiveCategory::Testing => "TEST",
            DirectiveCategory::Deployment => "DEPLOY",
            DirectiveCategory::Security => "SEC",
            DirectiveCategory::Workflow => "WORK",
            DirectiveCategory::Architecture => "ARCH",
            DirectiveCategory::Communication => "COMM",
        };

        let max_code: Option<String> = sqlx::query_scalar(&format!(r#"
            SELECT code FROM directives 
            WHERE project_id = ? AND code LIKE '{}-%'
            ORDER BY CAST(SUBSTR(code, {}) AS INTEGER) DESC 
            LIMIT 1
        "#, prefix, prefix.len() + 2))
        .bind(project_id)
        .fetch_optional(pool)
        .await?;

        let next_num = if let Some(code) = max_code {
            let num_str = code.split('-').nth(1).unwrap_or("0");
            num_str.parse::<i32>().unwrap_or(0) + 1
        } else {
            1
        };

        Ok(format!("{}-{:03}", prefix, next_num))
    }
}

/// Milestone CRUD operations
pub mod milestones {
    use super::*;
    use chrono::{DateTime, Utc};

    pub async fn create(
        pool: &sqlx::SqlitePool,
        project_id: &str,
        title: &str,
        description: &str,
        target_date: Option<DateTime<Utc>>,
        feature_ids: Option<Vec<String>>,
        success_criteria: Option<Vec<String>>,
    ) -> Result<Milestone> {
        let id = generate_milestone_id(pool, project_id).await?;
        let now = Utc::now();
        
        let feature_ids_json = feature_ids.map(|ids| serde_json::to_string(&ids).unwrap());
        let success_criteria_json = success_criteria.map(|criteria| serde_json::to_string(&criteria).unwrap());

        let milestone = sqlx::query_as::<_, Milestone>(r#"
            INSERT INTO milestones (
                id, project_id, title, description, target_date, achieved_date, 
                status, feature_ids, success_criteria, completion_percentage, 
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, NULL, 'planned', ?, ?, 0.0, ?, ?)
            RETURNING *
        "#)
        .bind(&id)
        .bind(project_id)
        .bind(title)
        .bind(description)
        .bind(target_date)
        .bind(feature_ids_json)
        .bind(success_criteria_json)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(milestone)
    }

    pub async fn get_by_id(pool: &sqlx::SqlitePool, id: &str) -> Result<Option<Milestone>> {
        let milestone = sqlx::query_as::<_, Milestone>(
            "SELECT * FROM milestones WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(milestone)
    }

    pub async fn get_by_project(pool: &sqlx::SqlitePool, project_id: &str) -> Result<Vec<Milestone>> {
        let milestones = sqlx::query_as::<_, Milestone>(
            "SELECT * FROM milestones WHERE project_id = ? ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(milestones)
    }

    pub async fn update(
        pool: &sqlx::SqlitePool,
        id: &str,
        title: Option<&str>,
        description: Option<&str>,
        target_date: Option<Option<DateTime<Utc>>>,
        status: Option<&str>,
        completion_percentage: Option<f64>,
        feature_ids: Option<Vec<String>>,
        success_criteria: Option<Vec<String>>,
    ) -> Result<Milestone> {
        let milestone = get_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Milestone not found: {}", id))?;

        let updated_title = title.unwrap_or(&milestone.title);
        let updated_description = description.unwrap_or(&milestone.description);
        let updated_target_date = target_date.unwrap_or(milestone.target_date);
        let updated_status = status.unwrap_or(&milestone.status);
        let updated_completion = completion_percentage.unwrap_or(milestone.completion_percentage);
        
        let feature_ids_json = feature_ids.map(|ids| serde_json::to_string(&ids).unwrap())
            .or(milestone.feature_ids);
        let success_criteria_json = success_criteria.map(|criteria| serde_json::to_string(&criteria).unwrap())
            .or(milestone.success_criteria);

        let updated_milestone = sqlx::query_as::<_, Milestone>(r#"
            UPDATE milestones SET 
                title = ?, description = ?, target_date = ?, status = ?, 
                completion_percentage = ?, feature_ids = ?, success_criteria = ?, 
                updated_at = ?
            WHERE id = ?
            RETURNING *
        "#)
        .bind(updated_title)
        .bind(updated_description)
        .bind(updated_target_date)
        .bind(updated_status)
        .bind(updated_completion)
        .bind(feature_ids_json)
        .bind(success_criteria_json)
        .bind(Utc::now())
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(updated_milestone)
    }

    pub async fn mark_achieved(pool: &sqlx::SqlitePool, id: &str) -> Result<Milestone> {
        let now = Utc::now();
        
        let milestone = sqlx::query_as::<_, Milestone>(r#"
            UPDATE milestones SET 
                status = 'achieved', 
                achieved_date = ?, 
                completion_percentage = 100.0,
                updated_at = ?
            WHERE id = ?
            RETURNING *
        "#)
        .bind(now)
        .bind(now)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(milestone)
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM milestones WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_by_status(pool: &sqlx::SqlitePool, project_id: &str, status: &str) -> Result<Vec<Milestone>> {
        let milestones = sqlx::query_as::<_, Milestone>(
            "SELECT * FROM milestones WHERE project_id = ? AND status = ? ORDER BY created_at DESC"
        )
        .bind(project_id)
        .bind(status)
        .fetch_all(pool)
        .await?;

        Ok(milestones)
    }

    async fn generate_milestone_id(pool: &sqlx::SqlitePool, project_id: &str) -> Result<String> {
        let max_id: Option<String> = sqlx::query_scalar(r#"
            SELECT id FROM milestones 
            WHERE project_id = ? AND id LIKE 'mil-%'
            ORDER BY CAST(SUBSTR(id, 5) AS INTEGER) DESC 
            LIMIT 1
        "#)
        .bind(project_id)
        .fetch_optional(pool)
        .await?;

        let next_num = if let Some(id) = max_id {
            let num_str = id.split('-').nth(1).unwrap_or("0");
            num_str.parse::<i32>().unwrap_or(0) + 1
        } else {
            1
        };

        Ok(format!("mil-{:08}", next_num))
    }
}

/// Note CRUD operations - attachable to any entity
pub mod notes {
    use super::*;

    pub async fn create(
        pool: &SqlitePool,
        project_id: Uuid,
        title: String,
        content: String,
        note_type: NoteType,
        is_project_wide: bool,
    ) -> Result<Note> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let note = Note {
            id: id.into(),
            project_id: project_id.into(),
            entity_id: None,
            entity_type: None,
            note_type: note_type.clone(),
            title: title.clone(),
            content: content.clone(),
            tags: None,
            author: Some("claude".to_string()),
            is_project_wide,
            is_pinned: false,
            created_at: now,
            updated_at: now,
            metadata: None,
        };

        sqlx::query(r#"
            INSERT INTO notes (
                id, project_id, note_type, title, content, author, is_project_wide,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(id)
        .bind(project_id)
        .bind(&note_type)
        .bind(&title)
        .bind(&content)
        .bind("claude")
        .bind(is_project_wide)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        Ok(note)
    }

    pub async fn create_entity_note(
        pool: &SqlitePool,
        entity_id: Uuid,
        entity_type: EntityType,
        note_type: NoteType,
        title: String,
        content: String,
    ) -> Result<Note> {
        // Get project_id from entity
        let project_id = get_project_id_for_entity(pool, entity_id, &entity_type).await?;
        
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let note = Note {
            id: id.into(),
            project_id: project_id.into(),
            entity_id: Some(entity_id.to_string()),
            entity_type: Some(entity_type.clone()),
            note_type: note_type.clone(),
            title: title.clone(),
            content: content.clone(),
            tags: None,
            author: Some("claude".to_string()),
            is_project_wide: false,
            is_pinned: false,
            created_at: now,
            updated_at: now,
            metadata: None,
        };

        sqlx::query(r#"
            INSERT INTO notes (
                id, project_id, entity_id, entity_type, note_type, title, content, author,
                is_project_wide, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(id)
        .bind(project_id)
        .bind(entity_id.to_string())
        .bind(&entity_type)
        .bind(&note_type)
        .bind(&title)
        .bind(&content)
        .bind("claude")
        .bind(false)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        Ok(note)
    }

    pub async fn get_notes_for_entity(pool: &SqlitePool, entity_id: Uuid) -> Result<Vec<Note>> {
        let notes = sqlx::query_as::<_, Note>(r#"
            SELECT id, project_id, entity_id, entity_type, note_type, title, content, tags,
                   author, is_project_wide, is_pinned, created_at, updated_at
            FROM notes
            WHERE entity_id = ?
            ORDER BY is_pinned DESC, created_at DESC
        "#)
        .bind(entity_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(notes)
    }

    pub async fn get_project_notes(pool: &SqlitePool, project_id: Uuid) -> Result<Vec<Note>> {
        let notes = sqlx::query_as::<_, Note>(r#"
            SELECT id, project_id, entity_id, entity_type, note_type, title, content, tags,
                   author, is_project_wide, is_pinned, created_at, updated_at
            FROM notes
            WHERE project_id = ? AND is_project_wide = TRUE
            ORDER BY is_pinned DESC, created_at DESC
        "#)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(notes)
    }

    async fn get_project_id_for_entity(
        pool: &SqlitePool,
        entity_id: Uuid,
        entity_type: &EntityType,
    ) -> Result<Uuid> {
        let table = match entity_type {
            EntityType::Project => "projects",
            EntityType::Feature => "features",
            EntityType::Task => "tasks",
            EntityType::Session => "sessions",
            EntityType::Directive => "directives",
            EntityType::Template => "templates",
            EntityType::Test => "tests",
            EntityType::Dependency => "dependencies",
            EntityType::Milestone => "milestones",
            EntityType::Note => "notes",
            EntityType::AuditTrail => "entity_audit_trails",
        };

        let project_id_str: String = if table == "projects" {
            entity_id.to_string()
        } else {
            sqlx::query_scalar(&format!("SELECT project_id FROM {} WHERE id = ?", table))
                .bind(entity_id.to_string())
                .fetch_one(pool)
                .await?
        };

        Ok(Uuid::parse_str(&project_id_str)?)
    }
}
// F0131 Entity State Tracking - Audit Trail System
use anyhow::Result;
use chrono::Utc;
use serde_json;
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

use super::models::{EntityAuditTrail, AuditTrailQuery};
use super::EntityType;

/// Generate unique audit trail ID
pub fn generate_audit_id() -> String {
    let uuid_str = Uuid::new_v4().to_string();
    format!("audit-{}", &uuid_str[..12])
}

/// Universal audit trail recording for any entity changes
pub async fn record_entity_changes<T>(
    pool: &SqlitePool,
    entity_id: &str,
    entity_type: EntityType,
    project_id: &str,
    old_entity: &T,
    new_entity: &T,
    operation_type: String,
    triggered_by: String,
    session_id: Option<String>,
    change_reason: Option<String>,
) -> Result<Vec<EntityAuditTrail>>
where 
    T: serde::Serialize,
{
    let mut audit_records = Vec::new();
    
    // Serialize entities to compare changes
    let old_json = serde_json::to_value(old_entity)?;
    let new_json = serde_json::to_value(new_entity)?;
    
    // Compare fields and record changes
    if let (Some(old_obj), Some(new_obj)) = (old_json.as_object(), new_json.as_object()) {
        for (field, new_value) in new_obj {
            if let Some(old_value) = old_obj.get(field) {
                if old_value != new_value {
                    let audit_record = EntityAuditTrail {
                        id: generate_audit_id(),
                        entity_id: entity_id.to_string(),
                        entity_type: entity_type.clone(),
                        project_id: project_id.to_string(),
                        operation_type: operation_type.clone(),
                        field_changed: Some(field.clone()),
                        old_value: Some(serde_json::to_string(old_value)?),
                        new_value: Some(serde_json::to_string(new_value)?),
                        change_reason: change_reason.clone(),
                        triggered_by: triggered_by.clone(),
                        session_id: session_id.clone(),
                        timestamp: Utc::now(),
                        metadata: None,
                    };
                    
                    // Insert audit record
                    create_audit_record(pool, &audit_record).await?;
                    audit_records.push(audit_record);
                }
            }
        }
    }
    
    Ok(audit_records)
}

/// Record a single operation audit trail (for create/delete operations)
pub async fn record_operation_audit(
    pool: &SqlitePool,
    entity_id: &str,
    entity_type: EntityType,
    project_id: &str,
    operation_type: String,
    triggered_by: String,
    session_id: Option<String>,
    change_reason: Option<String>,
    entity_data: Option<String>,
) -> Result<EntityAuditTrail> {
    let audit_record = EntityAuditTrail {
        id: generate_audit_id(),
        entity_id: entity_id.to_string(),
        entity_type: entity_type.clone(),
        project_id: project_id.to_string(),
        operation_type: operation_type.clone(),
        field_changed: None,
        old_value: if operation_type == "delete" { entity_data.clone() } else { None },
        new_value: if operation_type == "create" { entity_data } else { None },
        change_reason,
        triggered_by,
        session_id,
        timestamp: Utc::now(),
        metadata: None,
    };
    
    create_audit_record(pool, &audit_record).await?;
    Ok(audit_record)
}

/// Create audit record in database
async fn create_audit_record(pool: &SqlitePool, audit: &EntityAuditTrail) -> Result<()> {
    sqlx::query(r#"
        INSERT INTO entity_audit_trails (
            id, entity_id, entity_type, project_id, operation_type,
            field_changed, old_value, new_value, change_reason, triggered_by, 
            session_id, timestamp, metadata
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#)
    .bind(&audit.id)
    .bind(&audit.entity_id)
    .bind(&audit.entity_type)
    .bind(&audit.project_id)
    .bind(&audit.operation_type)
    .bind(&audit.field_changed)
    .bind(&audit.old_value)
    .bind(&audit.new_value)
    .bind(&audit.change_reason)
    .bind(&audit.triggered_by)
    .bind(&audit.session_id)
    .bind(audit.timestamp.to_rfc3339())
    .bind(&audit.metadata)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Get audit trail for specific entity
pub async fn get_entity_audit_trail(
    pool: &SqlitePool,
    entity_id: &str,
    entity_type: Option<EntityType>,
) -> Result<Vec<EntityAuditTrail>> {
    let mut query = "SELECT * FROM entity_audit_trails WHERE entity_id = ?".to_string();
    query.push_str(" ORDER BY timestamp DESC");
    
    let audit_records = if let Some(etype) = entity_type {
        sqlx::query_as::<_, EntityAuditTrail>(&format!("{} AND entity_type = ?", query))
            .bind(entity_id)
            .bind(&etype)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, EntityAuditTrail>(&query)
            .bind(entity_id)
            .fetch_all(pool)
            .await?
    };
    Ok(audit_records)
}

/// Get audit trail with comprehensive filters
pub async fn query_audit_trail(
    pool: &SqlitePool,
    query: &AuditTrailQuery,
) -> Result<Vec<EntityAuditTrail>> {
    let mut sql = "SELECT * FROM entity_audit_trails WHERE 1=1".to_string();
    let mut bindings: Vec<String> = Vec::new();
    
    if let Some(entity_id) = &query.entity_id {
        sql.push_str(" AND entity_id = ?");
        bindings.push(entity_id.clone());
    }
    
    if let Some(entity_type) = &query.entity_type {
        sql.push_str(" AND entity_type = ?");
        bindings.push(entity_type.to_string());
    }
    
    if let Some(project_id) = &query.project_id {
        sql.push_str(" AND project_id = ?");
        bindings.push(project_id.clone());
    }
    
    if let Some(operation_type) = &query.operation_type {
        sql.push_str(" AND operation_type = ?");
        bindings.push(operation_type.clone());
    }
    
    if let Some(triggered_by) = &query.triggered_by {
        sql.push_str(" AND triggered_by = ?");
        bindings.push(triggered_by.clone());
    }
    
    if let Some(field_changed) = &query.field_changed {
        sql.push_str(" AND field_changed = ?");
        bindings.push(field_changed.clone());
    }
    
    if let Some((start, end)) = &query.date_range {
        sql.push_str(" AND timestamp BETWEEN ? AND ?");
        bindings.push(start.to_rfc3339());
        bindings.push(end.to_rfc3339());
    }
    
    sql.push_str(" ORDER BY timestamp DESC");
    
    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    
    if let Some(offset) = query.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }
    
    let mut query_builder = sqlx::query_as::<_, EntityAuditTrail>(&sql);
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }
    
    let audit_records = query_builder.fetch_all(pool).await?;
    Ok(audit_records)
}

/// Get audit trail statistics
pub async fn get_audit_statistics(
    pool: &SqlitePool,
    project_id: Option<&str>,
) -> Result<HashMap<String, i64>> {
    let mut stats = HashMap::new();
    
    let base_query = if let Some(pid) = project_id {
        format!("FROM entity_audit_trails WHERE project_id = '{}'", pid)
    } else {
        "FROM entity_audit_trails".to_string()
    };
    
    // Total audit records
    let total: (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) {}", base_query))
        .fetch_one(pool)
        .await?;
    stats.insert("total_audit_records".to_string(), total.0);
    
    // Records by entity type
    let entity_type_stats: Vec<(String, i64)> = sqlx::query_as(&format!(
        "SELECT entity_type, COUNT(*) {} GROUP BY entity_type", base_query
    ))
    .fetch_all(pool)
    .await?;
    
    for (entity_type, count) in entity_type_stats {
        stats.insert(format!("{}_audit_records", entity_type), count);
    }
    
    // Records by operation type
    let operation_stats: Vec<(String, i64)> = sqlx::query_as(&format!(
        "SELECT operation_type, COUNT(*) {} GROUP BY operation_type", base_query
    ))
    .fetch_all(pool)
    .await?;
    
    for (operation_type, count) in operation_stats {
        stats.insert(format!("{}_operations", operation_type), count);
    }
    
    Ok(stats)
}

/// Rollback capabilities - get entity state at specific timestamp
pub async fn get_entity_state_at_timestamp(
    pool: &SqlitePool,
    entity_id: &str,
    entity_type: EntityType,
    target_timestamp: chrono::DateTime<Utc>,
) -> Result<HashMap<String, String>> {
    // Get audit trail up to target timestamp
    let audit_records = sqlx::query_as::<_, EntityAuditTrail>(r#"
        SELECT * FROM entity_audit_trails 
        WHERE entity_id = ? AND entity_type = ? AND timestamp <= ?
        ORDER BY timestamp DESC
    "#)
    .bind(entity_id)
    .bind(&entity_type)
    .bind(target_timestamp.to_rfc3339())
    .fetch_all(pool)
    .await?;
    
    // Build rollback state from audit trail
    let mut rollback_values: HashMap<String, String> = HashMap::new();
    
    // Start with most recent and work backwards to build historical state
    for record in audit_records {
        if let Some(field) = &record.field_changed {
            if !rollback_values.contains_key(field) {
                if let Some(old_value) = &record.old_value {
                    rollback_values.insert(field.clone(), old_value.clone());
                }
            }
        }
    }
    
    Ok(rollback_values)
}

/// Delete old audit records (for cleanup/archival)
pub async fn cleanup_old_audit_records(
    pool: &SqlitePool,
    older_than_days: i64,
) -> Result<i64> {
    let cutoff_date = Utc::now() - chrono::Duration::days(older_than_days);
    
    let result = sqlx::query(r#"
        DELETE FROM entity_audit_trails 
        WHERE timestamp < ?
    "#)
    .bind(cutoff_date.to_rfc3339())
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected() as i64)
}
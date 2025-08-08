// Note Link Management System - F0137 Note Linking and References

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use super::models::{NoteLink, NoteLinkQuery};

/// Create a link between a note and another entity or note
pub async fn create_link(
    pool: &SqlitePool,
    project_id: &str,
    source_note_id: &str,
    target_type: &str,
    target_id: &str,
    target_entity_type: Option<&str>,
    link_type: &str,
    auto_detected: bool,
    detection_reason: Option<&str>,
) -> Result<NoteLink> {
    let id = format!("link-{}", chrono::Utc::now().timestamp_millis());
    let now = Utc::now();
    
    let note_link = NoteLink {
        id: id.clone(),
        project_id: project_id.to_string(),
        source_note_id: source_note_id.to_string(),
        target_type: target_type.to_string(),
        target_id: target_id.to_string(),
        target_entity_type: target_entity_type.map(|s| s.to_string()),
        link_type: link_type.to_string(),
        auto_detected,
        detection_reason: detection_reason.map(|s| s.to_string()),
        created_at: now,
        updated_at: now,
        metadata: None,
    };
    
    sqlx::query(r#"
        INSERT INTO note_links (
            id, project_id, source_note_id, target_type, target_id, target_entity_type,
            link_type, auto_detected, detection_reason, created_at, updated_at, metadata
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#)
    .bind(&note_link.id)
    .bind(&note_link.project_id)
    .bind(&note_link.source_note_id)
    .bind(&note_link.target_type)
    .bind(&note_link.target_id)
    .bind(&note_link.target_entity_type)
    .bind(&note_link.link_type)
    .bind(note_link.auto_detected)
    .bind(&note_link.detection_reason)
    .bind(note_link.created_at.to_rfc3339())
    .bind(note_link.updated_at.to_rfc3339())
    .bind(&note_link.metadata)
    .execute(pool)
    .await?;
    
    Ok(note_link)
}

/// Get all links for a specific note (outgoing links)
pub async fn get_links_from_note(pool: &SqlitePool, note_id: &str) -> Result<Vec<NoteLink>> {
    let links = sqlx::query_as::<_, NoteLink>(r#"
        SELECT id, project_id, source_note_id, target_type, target_id, target_entity_type,
               link_type, auto_detected, detection_reason, created_at, updated_at, metadata
        FROM note_links 
        WHERE source_note_id = ?
        ORDER BY created_at DESC
    "#)
    .bind(note_id)
    .fetch_all(pool)
    .await?;
    
    Ok(links)
}

/// Get all links pointing to a specific entity or note (incoming links)
pub async fn get_links_to_target(pool: &SqlitePool, target_id: &str, target_type: Option<&str>) -> Result<Vec<NoteLink>> {
    let mut query = "SELECT id, project_id, source_note_id, target_type, target_id, target_entity_type,
                           link_type, auto_detected, detection_reason, created_at, updated_at, metadata
                     FROM note_links WHERE target_id = ?".to_string();
    
    if let Some(_t_type) = target_type {
        query.push_str(" AND target_type = ?");
    }
    query.push_str(" ORDER BY created_at DESC");
    
    let mut query_builder = sqlx::query_as::<_, NoteLink>(&query).bind(target_id);
    
    if let Some(_t_type) = target_type {
        query_builder = query_builder.bind(_t_type);
    }
    
    let links = query_builder.fetch_all(pool).await?;
    Ok(links)
}

/// Get bidirectional links for an entity (both incoming and outgoing)
pub async fn get_bidirectional_links(pool: &SqlitePool, entity_id: &str, entity_type: Option<&str>) -> Result<(Vec<NoteLink>, Vec<NoteLink>)> {
    let outgoing = if entity_type == Some("note") {
        get_links_from_note(pool, entity_id).await?
    } else {
        Vec::new()
    };
    
    let incoming = get_links_to_target(pool, entity_id, entity_type).await?;
    
    Ok((outgoing, incoming))
}

/// Traverse links to find connected entities
pub async fn traverse_link_chain(pool: &SqlitePool, start_entity_id: &str, max_depth: usize) -> Result<Vec<LinkTraversalResult>> {
    let mut visited = std::collections::HashSet::new();
    let mut results = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    
    queue.push_back((start_entity_id.to_string(), 0));
    visited.insert(start_entity_id.to_string());
    
    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        
        // Get all outgoing links from current entity
        let (outgoing_links, _) = get_bidirectional_links(pool, &current_id, None).await?;
        
        for link in outgoing_links {
            if !visited.contains(&link.target_id) {
                visited.insert(link.target_id.clone());
                results.push(LinkTraversalResult {
                    entity_id: link.target_id.clone(),
                    entity_type: link.target_entity_type.clone(),
                    depth: depth + 1,
                    path_from_start: format!("{} -> {}", current_id, link.target_id),
                    link_type: link.link_type.clone(),
                });
                
                // Add to queue for further traversal
                if depth + 1 < max_depth {
                    queue.push_back((link.target_id, depth + 1));
                }
            }
        }
    }
    
    Ok(results)
}

/// Find all entities connected to a given entity through note links
pub async fn find_connected_entities(pool: &SqlitePool, entity_id: &str) -> Result<Vec<ConnectedEntity>> {
    let mut connected = Vec::new();
    let mut processed = std::collections::HashSet::new();
    
    // Get direct bidirectional links
    let (outgoing_links, incoming_links) = get_bidirectional_links(pool, entity_id, None).await?;
    
    // Process outgoing links
    for link in outgoing_links {
        if !processed.contains(&link.target_id) {
            processed.insert(link.target_id.clone());
            connected.push(ConnectedEntity {
                entity_id: link.target_id,
                entity_type: link.target_entity_type,
                connection_type: "outgoing".to_string(),
                link_type: link.link_type,
                hop_count: 1,
            });
        }
    }
    
    // Process incoming links
    for link in incoming_links {
        if !processed.contains(&link.source_note_id) {
            processed.insert(link.source_note_id.clone());
            connected.push(ConnectedEntity {
                entity_id: link.source_note_id,
                entity_type: Some("note".to_string()),
                connection_type: "incoming".to_string(),
                link_type: link.link_type,
                hop_count: 1,
            });
        }
    }
    
    Ok(connected)
}

/// Auto-create links when notes are created or updated with detected patterns
pub async fn auto_create_detected_links(pool: &SqlitePool, project_id: &str, note_id: &str, content: &str) -> Result<Vec<NoteLink>> {
    let detected_links = detect_potential_links(pool, project_id, content).await?;
    let mut created_links = Vec::new();
    
    for detected in detected_links {
        let link = create_link(
            pool,
            project_id,
            note_id,
            &detected.target_type,
            &detected.target_id,
            detected.target_entity_type.as_deref(),
            &detected.link_type,
            true, // Auto-detected
            Some(&detected.detection_reason),
        ).await?;
        
        created_links.push(link);
    }
    
    Ok(created_links)
}

/// Remove a specific link
pub async fn remove_link(pool: &SqlitePool, link_id: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM note_links WHERE id = ?")
        .bind(link_id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}

/// Remove all links from a specific note
pub async fn remove_links_from_note(pool: &SqlitePool, note_id: &str) -> Result<u64> {
    let result = sqlx::query("DELETE FROM note_links WHERE source_note_id = ?")
        .bind(note_id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected())
}

/// Remove all links to a specific target
pub async fn remove_links_to_target(pool: &SqlitePool, target_id: &str, target_type: Option<&str>) -> Result<u64> {
    let mut query = "DELETE FROM note_links WHERE target_id = ?".to_string();
    
    if let Some(_t_type) = target_type {
        query.push_str(" AND target_type = ?");
    }
    
    let mut query_builder = sqlx::query(&query).bind(target_id);
    
    if let Some(_t_type) = target_type {
        query_builder = query_builder.bind(_t_type);
    }
    
    let result = query_builder.execute(pool).await?;
    Ok(result.rows_affected())
}

/// Query links with filters
pub async fn query_links(pool: &SqlitePool, query: &NoteLinkQuery) -> Result<Vec<NoteLink>> {
    let mut sql = "SELECT id, project_id, source_note_id, target_type, target_id, target_entity_type,
                          link_type, auto_detected, detection_reason, created_at, updated_at, metadata
                   FROM note_links WHERE 1=1".to_string();
    let mut params: Vec<String> = Vec::new();
    
    if let Some(source_id) = &query.source_note_id {
        sql.push_str(" AND source_note_id = ?");
        params.push(source_id.clone());
    }
    
    if let Some(target_id) = &query.target_id {
        sql.push_str(" AND target_id = ?");
        params.push(target_id.clone());
    }
    
    if let Some(target_type) = &query.target_type {
        sql.push_str(" AND target_type = ?");
        params.push(target_type.clone());
    }
    
    if let Some(link_type) = &query.link_type {
        sql.push_str(" AND link_type = ?");
        params.push(link_type.clone());
    }
    
    if let Some(auto_detected) = query.auto_detected {
        sql.push_str(" AND auto_detected = ?");
        params.push(auto_detected.to_string());
    }
    
    if let Some(project_id) = &query.project_id {
        sql.push_str(" AND project_id = ?");
        params.push(project_id.clone());
    }
    
    sql.push_str(" ORDER BY created_at DESC");
    
    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    
    if let Some(offset) = query.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }
    
    let mut query_builder = sqlx::query_as::<_, NoteLink>(&sql);
    for param in params {
        query_builder = query_builder.bind(param);
    }
    
    let links = query_builder.fetch_all(pool).await?;
    Ok(links)
}

/// Get link statistics for a project
pub async fn get_link_stats(pool: &SqlitePool, project_id: &str) -> Result<LinkStats> {
    let stats = sqlx::query_as::<_, LinkStatsRow>(r#"
        SELECT 
            COUNT(*) as total_links,
            COUNT(CASE WHEN auto_detected = TRUE THEN 1 END) as auto_detected_links,
            COUNT(CASE WHEN target_type = 'note' THEN 1 END) as note_to_note_links,
            COUNT(CASE WHEN target_type = 'entity' THEN 1 END) as note_to_entity_links,
            COUNT(DISTINCT source_note_id) as notes_with_outgoing_links,
            COUNT(DISTINCT target_id) as referenced_entities
        FROM note_links 
        WHERE project_id = ?
    "#)
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    
    Ok(LinkStats {
        total_links: stats.total_links,
        auto_detected_links: stats.auto_detected_links,
        note_to_note_links: stats.note_to_note_links,
        note_to_entity_links: stats.note_to_entity_links,
        notes_with_outgoing_links: stats.notes_with_outgoing_links,
        referenced_entities: stats.referenced_entities,
    })
}

#[derive(sqlx::FromRow)]
struct LinkStatsRow {
    total_links: i64,
    auto_detected_links: i64,
    note_to_note_links: i64,
    note_to_entity_links: i64,
    notes_with_outgoing_links: i64,
    referenced_entities: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkStats {
    pub total_links: i64,
    pub auto_detected_links: i64,
    pub note_to_note_links: i64,
    pub note_to_entity_links: i64,
    pub notes_with_outgoing_links: i64,
    pub referenced_entities: i64,
}

/// Detect potential links in note content
pub async fn detect_potential_links(pool: &SqlitePool, _project_id: &str, content: &str) -> Result<Vec<DetectedLink>> {
    let mut detected = Vec::new();
    
    // Detect entity ID patterns (F0001, task-123, proj-456, etc.)
    for line in content.lines() {
        if let Some(captures) = regex::Regex::new(r"\b([A-Z]\d{4}|[a-z]+-\d+)\b").unwrap().captures(line) {
            if let Some(entity_id) = captures.get(1) {
                let id = entity_id.as_str();
                
                // Determine entity type from ID prefix
                let (target_type, entity_type) = if id.starts_with("F") {
                    ("entity", Some("feature"))
                } else if id.starts_with("task-") {
                    ("entity", Some("task"))
                } else if id.starts_with("proj-") {
                    ("entity", Some("project"))
                } else if id.starts_with("note-") {
                    ("note", None)
                } else {
                    continue;
                };
                
                // Verify entity exists in database
                if let Ok(_) = verify_entity_exists(pool, id, target_type, entity_type).await {
                    detected.push(DetectedLink {
                        target_type: target_type.to_string(),
                        target_id: id.to_string(),
                        target_entity_type: entity_type.map(|s| s.to_string()),
                        link_type: "reference".to_string(),
                        detection_reason: format!("ID pattern '{}' found in content", id),
                    });
                }
            }
        }
    }
    
    Ok(detected)
}

/// Verify that an entity exists in the database
async fn verify_entity_exists(pool: &SqlitePool, entity_id: &str, target_type: &str, entity_type: Option<&str>) -> Result<bool> {
    if target_type == "note" {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes WHERE id = ?")
            .bind(entity_id)
            .fetch_one(pool)
            .await?;
        return Ok(count > 0);
    }
    
    let table = match entity_type {
        Some("feature") => "features",
        Some("task") => "tasks", 
        Some("project") => "projects",
        Some("session") => "sessions",
        Some("directive") => "directives",
        Some("milestone") => "milestones",
        _ => return Ok(false),
    };
    
    let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {} WHERE id = ?", table))
        .bind(entity_id)
        .fetch_one(pool)
        .await?;
    
    Ok(count > 0)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectedLink {
    pub target_type: String,
    pub target_id: String,
    pub target_entity_type: Option<String>,
    pub link_type: String,
    pub detection_reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkTraversalResult {
    pub entity_id: String,
    pub entity_type: Option<String>,
    pub depth: usize,
    pub path_from_start: String,
    pub link_type: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectedEntity {
    pub entity_id: String,
    pub entity_type: Option<String>,
    pub connection_type: String, // "incoming", "outgoing"
    pub link_type: String,
    pub hop_count: usize,
}
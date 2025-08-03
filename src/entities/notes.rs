// Note Management System - Attachable to any entity

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::models::Note;
use super::{EntityType, NoteType};

/// List all notes in the database  
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Note>> {
    let notes = sqlx::query_as::<_, Note>(r#"
        SELECT * FROM notes ORDER BY created_at DESC
    "#)
    .fetch_all(pool)
    .await?;
    
    Ok(notes)
}

/// Create a project-wide note (architecture, decisions, etc.)
pub async fn create_project_note(
    pool: &SqlitePool,
    project_id: &str,
    note_type: NoteType,
    title: String,
    content: String,
) -> Result<Note> {
    let id = format!("note-{}", uuid::Uuid::new_v4().to_string()[..8].to_lowercase());
    let now = Utc::now();
    
    let note = Note {
        id: id.clone(),
        project_id: project_id.to_string(),
        entity_id: None,
        entity_type: None,
        note_type: note_type.clone(),
        title: title.clone(),
        content: content.clone(),
        tags: None,
        author: Some("claude".to_string()),
        is_project_wide: true,
        is_pinned: false,
        created_at: now,
        updated_at: now,
    };

    sqlx::query(r#"
        INSERT INTO notes (
            id, project_id, note_type, title, content, author, is_project_wide,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#)
    .bind(id.to_string())
    .bind(project_id.to_string())
    .bind(format!("{:?}", note_type).to_lowercase())
    .bind(&title)
    .bind(&content)
    .bind("claude")
    .bind(true)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(note)
}

/// Create a note attached to a specific entity
pub async fn create_entity_note(
    pool: &SqlitePool,
    entity_id: Uuid,
    entity_type: EntityType,
    note_type: NoteType,
    title: String,
    content: String,
) -> Result<Note> {
    // Get project_id from the entity
    let project_id = get_project_id_for_entity(pool, entity_id, &entity_type).await?;
    
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    let note = Note {
        id: id.into(),
        project_id: project_id.into(),
        entity_id: Some(entity_id),
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
    };

    sqlx::query(r#"
        INSERT INTO notes (
            id, project_id, entity_id, entity_type, note_type, title, content, author,
            is_project_wide, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#)
    .bind(id.to_string())
    .bind(project_id.to_string())
    .bind(entity_id.to_string())
    .bind(format!("{:?}", entity_type).to_lowercase())
    .bind(format!("{:?}", note_type).to_lowercase())
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

/// Get all notes for a specific entity
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

/// Get all project-wide notes
pub async fn get_project_notes(pool: &SqlitePool, project_id: Uuid) -> Result<Vec<Note>> {
    let notes = sqlx::query_as::<_, Note>(r#"
        SELECT id, project_id, entity_id, entity_type, note_type, title, content, tags,
               author, is_project_wide, is_pinned, created_at, updated_at
        FROM notes
        WHERE project_id = ? AND is_project_wide = TRUE
        ORDER BY is_pinned DESC, note_type, created_at DESC
    "#)
    .bind(project_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(notes)
}

/// Get all notes by type across project
pub async fn get_notes_by_type(
    pool: &SqlitePool,
    project_id: Uuid,
    note_type: NoteType,
) -> Result<Vec<Note>> {
    let notes = sqlx::query_as::<_, Note>(r#"
        SELECT id, project_id, entity_id, entity_type, note_type, title, content, tags,
               author, is_project_wide, is_pinned, created_at, updated_at
        FROM notes
        WHERE project_id = ? AND note_type = ?
        ORDER BY is_pinned DESC, is_project_wide DESC, created_at DESC
    "#)
    .bind(project_id.to_string())
    .bind(format!("{:?}", note_type).to_lowercase())
    .fetch_all(pool)
    .await?;

    Ok(notes)
}

/// Search notes by content
pub async fn search_notes(
    pool: &SqlitePool,
    project_id: Uuid,
    query: &str,
) -> Result<Vec<Note>> {
    let search_pattern = format!("%{}%", query);
    
    let notes = sqlx::query_as::<_, Note>(r#"
        SELECT id, project_id, entity_id, entity_type, note_type, title, content, tags,
               author, is_project_wide, is_pinned, created_at, updated_at
        FROM notes
        WHERE project_id = ? AND (title LIKE ? OR content LIKE ?)
        ORDER BY is_pinned DESC, is_project_wide DESC, created_at DESC
    "#)
    .bind(project_id.to_string())
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;

    Ok(notes)
}

/// Update note content
pub async fn update_note(
    pool: &SqlitePool,
    note_id: Uuid,
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<()> {
    let now = Utc::now();
    
    // Build dynamic update query based on provided fields
    let mut query_parts = vec!["updated_at = ?"];
    let mut values: Vec<String> = vec![now.to_rfc3339()];
    
    if let Some(title) = title {
        query_parts.push("title = ?");
        values.push(title);
    }
    
    if let Some(content) = content {
        query_parts.push("content = ?");
        values.push(content);
    }
    
    if let Some(tags) = tags {
        query_parts.push("tags = ?");
        values.push(serde_json::to_string(&tags)?);
    }
    
    let query = format!(
        "UPDATE notes SET {} WHERE id = ?",
        query_parts.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    for value in values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(note_id.to_string());
    
    query_builder.execute(pool).await?;
    
    Ok(())
}

/// Pin/unpin a note for importance
pub async fn toggle_pin(pool: &SqlitePool, note_id: Uuid) -> Result<bool> {
    let now = Utc::now();
    
    // Get current pin status
    let is_pinned: bool = sqlx::query_scalar("SELECT is_pinned FROM notes WHERE id = ?")
        .bind(note_id.to_string())
        .fetch_one(pool)
        .await?;
    
    let new_pin_status = !is_pinned;
    
    sqlx::query("UPDATE notes SET is_pinned = ?, updated_at = ? WHERE id = ?")
        .bind(new_pin_status)
        .bind(now.to_rfc3339())
        .bind(note_id.to_string())
        .execute(pool)
        .await?;
    
    Ok(new_pin_status)
}

/// Delete a note
pub async fn delete_note(pool: &SqlitePool, note_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id.to_string())
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Get comprehensive note summary for dashboard
pub async fn get_note_summary(pool: &SqlitePool, project_id: Uuid) -> Result<NoteSummary> {
    // Count notes by type
    let architecture_count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM notes 
        WHERE project_id = ? AND note_type = 'architecture'
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let decision_count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM notes 
        WHERE project_id = ? AND note_type = 'decision'
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let observation_count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM notes 
        WHERE project_id = ? AND note_type = 'observation'
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let total_count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM notes WHERE project_id = ?
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let pinned_count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM notes 
        WHERE project_id = ? AND is_pinned = TRUE
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let project_wide_count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM notes 
        WHERE project_id = ? AND is_project_wide = TRUE
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    // Get recent notes
    let recent_notes = sqlx::query_as::<_, Note>(r#"
        SELECT id, project_id, entity_id, entity_type, note_type, title, content, tags,
               author, is_project_wide, is_pinned, created_at, updated_at
        FROM notes
        WHERE project_id = ?
        ORDER BY created_at DESC
        LIMIT 5
    "#)
    .bind(project_id.to_string())
    .fetch_all(pool)
    .await?;
    
    Ok(NoteSummary {
        total_count: total_count as usize,
        architecture_count: architecture_count as usize,
        decision_count: decision_count as usize,
        observation_count: observation_count as usize,
        pinned_count: pinned_count as usize,
        project_wide_count: project_wide_count as usize,
        recent_notes,
    })
}

/// Get project_id for any entity
async fn get_project_id_for_entity(
    pool: &SqlitePool,
    entity_id: Uuid,
    entity_type: &EntityType,
) -> Result<Uuid> {
    let table = match entity_type {
        EntityType::Project => return Ok(entity_id), // project_id is the entity_id
        EntityType::Feature => "features",
        EntityType::Task => "tasks",
        EntityType::Session => "sessions",
        EntityType::Directive => "directives",
        EntityType::Template => "templates",
        EntityType::Test => "tests",
        EntityType::Dependency => "dependencies",
        EntityType::Note => "notes",
    };

    let project_id_str: String = sqlx::query_scalar(&format!(
        "SELECT project_id FROM {} WHERE id = ?", table
    ))
    .bind(entity_id.to_string())
    .fetch_one(pool)
    .await?;

    Ok(Uuid::parse_str(&project_id_str)?)
}

/// Note summary for dashboard display
#[derive(Debug, Clone)]
pub struct NoteSummary {
    pub total_count: usize,
    pub architecture_count: usize,
    pub decision_count: usize,
    pub observation_count: usize,
    pub pinned_count: usize,
    pub project_wide_count: usize,
    pub recent_notes: Vec<Note>,
}

/// Note creation with suggested categorization
pub async fn create_note_with_suggestion(
    pool: &SqlitePool,
    project_id: Uuid,
    entity_id: Option<Uuid>,
    entity_type: Option<EntityType>,
    title: String,
    content: String,
) -> Result<(Note, NoteType)> {
    // Analyze content to suggest note type
    let suggested_type = suggest_note_type(&title, &content);
    
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    let note = Note {
        id: id.into(),
        project_id: project_id.into(),
        entity_id,
        entity_type: entity_type.clone(),
        note_type: suggested_type.clone(),
        title: title.clone(),
        content: content.clone(),
        tags: None,
        author: Some("claude".to_string()),
        is_project_wide: entity_id.is_none(),
        is_pinned: false,
        created_at: now,
        updated_at: now,
    };

    sqlx::query(r#"
        INSERT INTO notes (
            id, project_id, entity_id, entity_type, note_type, title, content, author,
            is_project_wide, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#)
    .bind(id.to_string())
    .bind(project_id.to_string())
    .bind(entity_id.map(|id| id.to_string()))
    .bind(entity_type.map(|t| format!("{:?}", t).to_lowercase()))
    .bind(format!("{:?}", suggested_type).to_lowercase())
    .bind(&title)
    .bind(&content)
    .bind("claude")
    .bind(entity_id.is_none())
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok((note, suggested_type))
}

/// Suggest note type based on content analysis
fn suggest_note_type(title: &str, content: &str) -> NoteType {
    let combined = format!("{} {}", title.to_lowercase(), content.to_lowercase());
    
    // Architecture keywords
    if combined.contains("architecture") || combined.contains("design pattern") 
        || combined.contains("system design") || combined.contains("component") 
        || combined.contains("module") || combined.contains("structure") {
        return NoteType::Architecture;
    }
    
    // Decision keywords
    if combined.contains("decision") || combined.contains("decided") 
        || combined.contains("choice") || combined.contains("option") 
        || combined.contains("approach") || combined.contains("strategy") {
        return NoteType::Decision;
    }
    
    // Evidence keywords
    if combined.contains("evidence") || combined.contains("proof") 
        || combined.contains("verified") || combined.contains("tested") 
        || combined.contains("validated") {
        return NoteType::Evidence;
    }
    
    // Issue keywords
    if combined.contains("issue") || combined.contains("problem") 
        || combined.contains("bug") || combined.contains("error") 
        || combined.contains("failure") {
        return NoteType::Issue;
    }
    
    // Reference keywords
    if combined.contains("reference") || combined.contains("link") 
        || combined.contains("see also") || combined.contains("documentation") 
        || combined.contains("spec") {
        return NoteType::Reference;
    }
    
    // Progress keywords
    if combined.contains("progress") || combined.contains("status") 
        || combined.contains("update") || combined.contains("completed") 
        || combined.contains("working on") {
        return NoteType::Progress;
    }
    
    // Reminder keywords
    if combined.contains("reminder") || combined.contains("remember") 
        || combined.contains("don't forget") || combined.contains("note") 
        || combined.contains("todo") {
        return NoteType::Reminder;
    }
    
    // Default to observation
    NoteType::Observation
}
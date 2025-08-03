// Entity Relationship Management

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

use super::models::Dependency;
use super::EntityType;

/// Create a dependency relationship between entities
pub async fn create_dependency(
    pool: &SqlitePool,
    project_id: Uuid,
    from_entity_id: Uuid,
    from_entity_type: EntityType,
    to_entity_id: Uuid,
    to_entity_type: EntityType,
    dependency_type: String,
    description: Option<String>,
) -> Result<Dependency> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    
    let dependency = Dependency {
        id: id.into(),
        project_id: project_id.into(),
        from_entity_id: from_entity_id.into(),
        from_entity_type: from_entity_type.clone(),
        to_entity_id: to_entity_id.into(),
        to_entity_type: to_entity_type.clone(),
        dependency_type: dependency_type.clone(),
        description: description.clone(),
        created_at: now,
        resolved_at: None,
    };

    sqlx::query(r#"
        INSERT INTO dependencies (
            id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type,
            dependency_type, description, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#)
    .bind(id.to_string())
    .bind(project_id.to_string())
    .bind(from_entity_id.to_string())
    .bind(format!("{:?}", from_entity_type).to_lowercase())
    .bind(to_entity_id.to_string())
    .bind(format!("{:?}", to_entity_type).to_lowercase())
    .bind(&dependency_type)
    .bind(&description)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(dependency)
}

/// Get all relationships for a specific entity
pub async fn get_relationships(
    pool: &SqlitePool,
    entity_id: Uuid,
) -> Result<HashMap<EntityType, Vec<Uuid>>> {
    let mut relationships: HashMap<EntityType, Vec<Uuid>> = HashMap::new();
    
    // Get entities this one depends on (outgoing dependencies)
    let outgoing = sqlx::query_as::<_, Dependency>(r#"
        SELECT id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type,
               dependency_type, description, created_at, resolved_at
        FROM dependencies
        WHERE from_entity_id = ? AND resolved_at IS NULL
    "#)
    .bind(entity_id.to_string())
    .fetch_all(pool)
    .await?;
    
    for dep in outgoing {
        relationships
            .entry(dep.to_entity_type)
            .or_insert_with(Vec::new)
            .push(dep.to_entity_id.into());
    }
    
    // Get entities that depend on this one (incoming dependencies)
    let incoming = sqlx::query_as::<_, Dependency>(r#"
        SELECT id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type,
               dependency_type, description, created_at, resolved_at
        FROM dependencies
        WHERE to_entity_id = ? AND resolved_at IS NULL
    "#)
    .bind(entity_id.to_string())
    .fetch_all(pool)
    .await?;
    
    for dep in incoming {
        relationships
            .entry(dep.from_entity_type)
            .or_insert_with(Vec::new)
            .push(dep.from_entity_id.into());
    }
    
    Ok(relationships)
}

/// Get all dependencies for a project
pub async fn get_project_dependencies(
    pool: &SqlitePool,
    project_id: Uuid,
) -> Result<Vec<Dependency>> {
    let dependencies = sqlx::query_as::<_, Dependency>(r#"
        SELECT id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type,
               dependency_type, description, created_at, resolved_at
        FROM dependencies
        WHERE project_id = ?
        ORDER BY created_at DESC
    "#)
    .bind(project_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(dependencies)
}

/// Get blocking dependencies for an entity
pub async fn get_blocking_dependencies(
    pool: &SqlitePool,
    entity_id: Uuid,
) -> Result<Vec<Dependency>> {
    let dependencies = sqlx::query_as::<_, Dependency>(r#"
        SELECT id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type,
               dependency_type, description, created_at, resolved_at
        FROM dependencies
        WHERE from_entity_id = ? AND dependency_type = 'blocking' AND resolved_at IS NULL
    "#)
    .bind(entity_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(dependencies)
}

/// Resolve a dependency (mark as completed)
pub async fn resolve_dependency(pool: &SqlitePool, dependency_id: Uuid) -> Result<()> {
    let now = Utc::now();
    
    sqlx::query("UPDATE dependencies SET resolved_at = ? WHERE id = ?")
        .bind(now.to_rfc3339())
        .bind(dependency_id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

/// Check if entity has unresolved blocking dependencies
pub async fn has_blocking_dependencies(pool: &SqlitePool, entity_id: Uuid) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM dependencies 
        WHERE from_entity_id = ? AND dependency_type = 'blocking' AND resolved_at IS NULL
    "#)
    .bind(entity_id.to_string())
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

/// Get dependency chain for an entity (recursive dependencies)
pub async fn get_dependency_chain(
    pool: &SqlitePool,
    entity_id: Uuid,
    max_depth: usize,
) -> Result<Vec<DependencyChain>> {
    let mut chains = Vec::new();
    let mut visited = std::collections::HashSet::new();
    
    get_dependency_chain_recursive(
        pool,
        entity_id,
        &mut chains,
        &mut visited,
        0,
        max_depth,
    ).await?;
    
    Ok(chains)
}

/// Recursive helper for dependency chain traversal
fn get_dependency_chain_recursive<'a>(
    pool: &'a SqlitePool,
    entity_id: Uuid,
    chains: &'a mut Vec<DependencyChain>,
    visited: &'a mut std::collections::HashSet<Uuid>,
    current_depth: usize,
    max_depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
    if current_depth >= max_depth || visited.contains(&entity_id) {
        return Ok(());
    }
    
    visited.insert(entity_id);
    
    // Get direct dependencies
    let dependencies = sqlx::query_as::<_, Dependency>(r#"
        SELECT id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type,
               dependency_type, description, created_at, resolved_at
        FROM dependencies
        WHERE from_entity_id = ? AND resolved_at IS NULL
    "#)
    .bind(entity_id.to_string())
    .fetch_all(pool)
    .await?;
    
    for dep in dependencies {
        chains.push(DependencyChain {
            from_entity: entity_id,
            to_entity: dep.to_entity_id.into(),
            dependency_type: dep.dependency_type.clone(),
            depth: current_depth,
            description: dep.description.clone(),
        });
        
        // Recurse into dependencies
        get_dependency_chain_recursive(
            pool,
            dep.to_entity_id.into(),
            chains,
            visited,
            current_depth + 1,
            max_depth,
        ).await?;
    }
    
    Ok(())
    })
}

/// Create automatic dependencies based on feature relationships
pub async fn create_feature_task_dependencies(
    pool: &SqlitePool,
    project_id: Uuid,
    feature_id: Uuid,
    task_id: Uuid,
) -> Result<()> {
    // Task depends on feature (task implements feature)
    create_dependency(
        pool,
        project_id,
        task_id,
        EntityType::Task,
        feature_id,
        EntityType::Feature,
        "implements".to_string(),
        Some("Task implements this feature".to_string()),
    ).await?;

    Ok(())
}

/// Create session-task relationships
pub async fn create_session_task_relationship(
    pool: &SqlitePool,
    project_id: Uuid,
    session_id: Uuid,
    task_id: Uuid,
) -> Result<()> {
    // Task worked on in session
    create_dependency(
        pool,
        project_id,
        task_id,
        EntityType::Task,
        session_id,
        EntityType::Session,
        "worked_in".to_string(),
        Some("Task worked on in this session".to_string()),
    ).await?;

    Ok(())
}

/// Get relationship statistics for dashboard
pub async fn get_relationship_stats(pool: &SqlitePool, project_id: Uuid) -> Result<RelationshipStats> {
    let total_dependencies: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM dependencies WHERE project_id = ?
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let active_dependencies: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM dependencies 
        WHERE project_id = ? AND resolved_at IS NULL
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    let blocking_dependencies: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM dependencies 
        WHERE project_id = ? AND dependency_type = 'blocking' AND resolved_at IS NULL
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    // Count entity relationships
    let feature_task_links: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(*) FROM dependencies 
        WHERE project_id = ? AND from_entity_type = 'task' AND to_entity_type = 'feature'
    "#)
    .bind(project_id.to_string())
    .fetch_one(pool)
    .await?;
    
    Ok(RelationshipStats {
        total_dependencies: total_dependencies as usize,
        active_dependencies: active_dependencies as usize,
        blocking_dependencies: blocking_dependencies as usize,
        feature_task_links: feature_task_links as usize,
    })
}

/// Find circular dependencies in the project
pub async fn find_circular_dependencies(
    pool: &SqlitePool,
    project_id: Uuid,
) -> Result<Vec<Vec<Uuid>>> {
    let dependencies = get_project_dependencies(pool, project_id).await?;
    let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    
    // Build dependency graph
    for dep in dependencies {
        if dep.resolved_at.is_none() {
            graph
                .entry(dep.from_entity_id.into())
                .or_insert_with(Vec::new)
                .push(dep.to_entity_id.into());
        }
    }
    
    // Find cycles using DFS
    let mut cycles = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();
    
    for &node in graph.keys() {
        if !visited.contains(&node) {
            find_cycles_dfs(
                &graph,
                node,
                &mut visited,
                &mut rec_stack,
                &mut Vec::new(),
                &mut cycles,
            );
        }
    }
    
    Ok(cycles)
}

/// DFS helper for cycle detection
fn find_cycles_dfs(
    graph: &HashMap<Uuid, Vec<Uuid>>,
    node: Uuid,
    visited: &mut std::collections::HashSet<Uuid>,
    rec_stack: &mut std::collections::HashSet<Uuid>,
    path: &mut Vec<Uuid>,
    cycles: &mut Vec<Vec<Uuid>>,
) {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);
    
    if let Some(neighbors) = graph.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                find_cycles_dfs(graph, neighbor, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(&neighbor) {
                // Found a cycle
                if let Some(cycle_start) = path.iter().position(|&x| x == neighbor) {
                    cycles.push(path[cycle_start..].to_vec());
                }
            }
        }
    }
    
    path.pop();
    rec_stack.remove(&node);
}

/// Dependency chain representation
#[derive(Debug, Clone)]
pub struct DependencyChain {
    pub from_entity: Uuid,
    pub to_entity: Uuid,
    pub dependency_type: String,
    pub depth: usize,
    pub description: Option<String>,
}

/// Relationship statistics for dashboard
#[derive(Debug, Clone)]
pub struct RelationshipStats {
    pub total_dependencies: usize,
    pub active_dependencies: usize,
    pub blocking_dependencies: usize,
    pub feature_task_links: usize,
}
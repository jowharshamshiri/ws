// EntityManager CRUD Operations Tests - New Schema-Based Architecture
// Tests T000026 - Comprehensive test suite for new entity CRUD operations

use anyhow::Result;
use tempfile::tempdir;
use workspace::entities::database::initialize_database;
use workspace::entities::{EntityManager, FeatureState, TaskStatus, SessionStatus, DirectiveCategory, Priority};

/// Test EntityManager project operations
#[tokio::test]
async fn test_entity_manager_project_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_entity_manager.db");
    
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Test create project
    let project = entity_manager.create_project(
        "Test Project".to_string(),
        "A test project for EntityManager validation".to_string(),
    ).await?;
    
    assert_eq!(project.id, "P001");
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.description, "A test project for EntityManager validation");
    
    // Test get project by ID
    let retrieved = entity_manager.get_project("P001").await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, "P001");
    assert_eq!(retrieved.name, "Test Project");
    
    // Test list active projects
    let active_projects = entity_manager.list_active_projects().await?;
    assert_eq!(active_projects.len(), 1);
    assert_eq!(active_projects[0].id, "P001");
    
    Ok(())
}

/// Test EntityManager feature operations
#[tokio::test]
async fn test_entity_manager_feature_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_entity_manager_features.db");
    
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Create a project first
    let project = entity_manager.create_project(
        "Feature Test Project".to_string(),
        "Project for testing features".to_string(),
    ).await?;
    
    // Test create feature with full parameters
    let feature = entity_manager.create_feature_full(
        project.id.clone(),
        "Test Feature".to_string(),
        "A test feature for validation".to_string(),
        Some("Core".to_string()),
    ).await?;
    
    assert_eq!(feature.project_id, project.id);
    assert_eq!(feature.name, "Test Feature");
    assert_eq!(feature.description, "A test feature for validation");
    assert_eq!(feature.category, Some("Core".to_string()));
    assert_eq!(feature.state, FeatureState::NotImplemented.as_str());
    
    // Test create feature with backward compatibility signature
    let feature2 = entity_manager.create_feature(
        "Second Feature".to_string(),
        "Another test feature".to_string(),
    ).await?;
    
    assert_eq!(feature2.name, "Second Feature");
    assert_eq!(feature2.description, "Another test feature");
    
    // Test get feature by ID
    let retrieved = entity_manager.get_feature(&feature.id).await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, feature.id);
    assert_eq!(retrieved.name, "Test Feature");
    
    // Test list features by project
    let project_features = entity_manager.list_features_by_project(&project.id).await?;
    assert_eq!(project_features.len(), 2);
    
    // Test list all features (backward compatibility)
    let all_features = entity_manager.list_features().await?;
    assert_eq!(all_features.len(), 2);
    
    // Test update feature state
    entity_manager.update_feature_state(&feature.id, FeatureState::ImplementedPassingTests).await?;
    let updated = entity_manager.get_feature(&feature.id).await?;
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().state, FeatureState::ImplementedPassingTests.as_str());
    
    Ok(())
}

/// Test EntityManager task operations
#[tokio::test]
async fn test_entity_manager_task_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_entity_manager_tasks.db");
    
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Create project and feature first
    let project = entity_manager.create_project(
        "Task Test Project".to_string(),
        "Project for testing tasks".to_string(),
    ).await?;
    
    let feature = entity_manager.create_feature_full(
        project.id.clone(),
        "Task Feature".to_string(),
        "Feature for task testing".to_string(),
        None,
    ).await?;
    
    // Test create task with full parameters
    let task = entity_manager.create_task_full(
        project.id.clone(),
        feature.id.clone(),
        "Test task implementation".to_string(),
        "feature".to_string(),
    ).await?;
    
    assert_eq!(task.project_id, project.id);
    assert_eq!(task.feature_id, feature.id);
    assert_eq!(task.task, "Test task implementation");
    assert_eq!(task.category, "feature");
    assert_eq!(task.status, TaskStatus::Pending.as_str());
    
    // Test create task with backward compatibility signature
    let task2 = entity_manager.create_task(
        "Second Task".to_string(),
        "Another test task".to_string(),
    ).await?;
    
    assert_eq!(task2.task, "Second Task");
    
    // Test get task by ID
    let retrieved = entity_manager.get_task(&task.id).await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, task.id);
    assert_eq!(retrieved.task, "Test task implementation");
    
    // Test list tasks by project
    let project_tasks = entity_manager.list_tasks_by_project(&project.id, None).await?;
    assert_eq!(project_tasks.len(), 2);
    
    // Test list tasks with status filter
    let pending_tasks = entity_manager.list_tasks_by_project(&project.id, Some(TaskStatus::Pending)).await?;
    assert_eq!(pending_tasks.len(), 2);
    
    // Test list all tasks (backward compatibility)
    let all_tasks = entity_manager.list_tasks().await?;
    assert_eq!(all_tasks.len(), 2);
    
    // Test update task status
    entity_manager.update_task_status(&task.id, TaskStatus::InProgress).await?;
    let updated = entity_manager.get_task(&task.id).await?;
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().status, TaskStatus::InProgress.as_str());
    
    // Test update task (backward compatibility)
    let mut task2_modified = task2.clone();
    task2_modified.status = TaskStatus::InProgress.as_str().to_string();
    entity_manager.update_task(task2_modified).await?;
    let updated2 = entity_manager.get_task(&task2.id).await?;
    assert!(updated2.is_some());
    assert_eq!(updated2.unwrap().status, TaskStatus::InProgress.as_str());
    
    Ok(())
}

/// Test EntityManager session operations
#[tokio::test]
async fn test_entity_manager_session_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_entity_manager_sessions.db");
    
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Create project first
    let project = entity_manager.create_project(
        "Session Test Project".to_string(),
        "Project for testing sessions".to_string(),
    ).await?;
    
    // Test create session
    let session = entity_manager.create_session(
        project.id.clone(),
        "Test Session".to_string(),
        "Session testing focus".to_string(),
    ).await?;
    
    assert_eq!(session.project_id, project.id);
    assert_eq!(session.title, "Test Session");
    assert_eq!(session.focus, "Session testing focus");
    assert_eq!(session.status, SessionStatus::Active.as_str());
    
    // Test get session by ID
    let retrieved = entity_manager.get_session(&session.id).await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, session.id);
    assert_eq!(retrieved.title, "Test Session");
    
    // Test complete session
    entity_manager.complete_session(&session.id, "Session completed successfully".to_string()).await?;
    let completed = entity_manager.get_session(&session.id).await?;
    assert!(completed.is_some());
    assert_eq!(completed.unwrap().status, SessionStatus::Completed.as_str());
    
    Ok(())
}

/// Test EntityManager directive operations
#[tokio::test]
async fn test_entity_manager_directive_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_entity_manager_directives.db");
    
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Create project first
    let project = entity_manager.create_project(
        "Directive Test Project".to_string(),
        "Project for testing directives".to_string(),
    ).await?;
    
    // Test create directive
    let directive = entity_manager.create_directive(
        project.id.clone(),
        "Test Directive".to_string(),
        "Always validate input parameters before processing".to_string(),
        DirectiveCategory::Architecture,
        Priority::High,
    ).await?;
    
    assert_eq!(directive.project_id, project.id);
    assert_eq!(directive.title, "Test Directive");
    assert_eq!(directive.rule, "Always validate input parameters before processing");
    assert_eq!(directive.category, Some(DirectiveCategory::Architecture.as_str().to_string()));
    assert_eq!(directive.priority, Priority::High.as_str());
    
    // Test get directive by ID
    let retrieved = entity_manager.get_directive(&directive.id).await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, directive.id);
    assert_eq!(retrieved.title, "Test Directive");
    
    // Test list active directives
    let active_directives = entity_manager.list_active_directives(&project.id).await?;
    assert_eq!(active_directives.len(), 1);
    assert_eq!(active_directives[0].id, directive.id);
    
    // Test deactivate directive
    entity_manager.deactivate_directive(&directive.id).await?;
    let deactivated_list = entity_manager.list_active_directives(&project.id).await?;
    assert_eq!(deactivated_list.len(), 0);
    
    Ok(())
}

/// Test EntityManager cross-entity relationships
#[tokio::test]
async fn test_entity_manager_relationships() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_entity_manager_relationships.db");
    
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Create project
    let project = entity_manager.create_project(
        "Relationship Test Project".to_string(),
        "Project for testing entity relationships".to_string(),
    ).await?;
    
    // Create multiple features
    let feature1 = entity_manager.create_feature_full(
        project.id.clone(),
        "Core Feature".to_string(),
        "Core functionality".to_string(),
        Some("Core".to_string()),
    ).await?;
    
    let feature2 = entity_manager.create_feature_full(
        project.id.clone(),
        "API Feature".to_string(),
        "API endpoints".to_string(),
        Some("API".to_string()),
    ).await?;
    
    // Create tasks for features
    let task1 = entity_manager.create_task_full(
        project.id.clone(),
        feature1.id.clone(),
        "Implement core logic".to_string(),
        "feature".to_string(),
    ).await?;
    
    let task2 = entity_manager.create_task_full(
        project.id.clone(),
        feature2.id.clone(),
        "Create API endpoints".to_string(),
        "api".to_string(),
    ).await?;
    
    // Create session
    let session = entity_manager.create_session(
        project.id.clone(),
        "Development Session".to_string(),
        "Working on core features".to_string(),
    ).await?;
    
    // Verify relationships
    let project_features = entity_manager.list_features_by_project(&project.id).await?;
    assert_eq!(project_features.len(), 2);
    
    let project_tasks = entity_manager.list_tasks_by_project(&project.id, None).await?;
    assert_eq!(project_tasks.len(), 2);
    
    // Verify task-feature relationships
    assert_eq!(task1.feature_id, feature1.id);
    assert_eq!(task2.feature_id, feature2.id);
    
    // Verify project relationships
    assert_eq!(feature1.project_id, project.id);
    assert_eq!(feature2.project_id, project.id);
    assert_eq!(task1.project_id, project.id);
    assert_eq!(task2.project_id, project.id);
    assert_eq!(session.project_id, project.id);
    
    Ok(())
}

/// Test EntityManager deletion operations with cascade behavior
#[tokio::test]
async fn test_entity_manager_deletion_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_entity_manager_deletions.db");
    
    let pool = initialize_database(&db_path).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Create project with features and tasks
    let project = entity_manager.create_project(
        "Deletion Test Project".to_string(),
        "Project for testing deletions".to_string(),
    ).await?;
    
    let feature = entity_manager.create_feature_full(
        project.id.clone(),
        "Test Feature".to_string(),
        "Feature for deletion testing".to_string(),
        None,
    ).await?;
    
    let task = entity_manager.create_task_full(
        project.id.clone(),
        feature.id.clone(),
        "Test task".to_string(),
        "testing".to_string(),
    ).await?;
    
    let session = entity_manager.create_session(
        project.id.clone(),
        "Test Session".to_string(),
        "Testing session".to_string(),
    ).await?;
    
    // Test delete task
    entity_manager.delete_task(&task.id).await?;
    let deleted_task = entity_manager.get_task(&task.id).await?;
    assert!(deleted_task.is_none());
    
    // Test delete feature (should handle foreign key relationships properly)
    entity_manager.delete_feature(&feature.id).await?;
    let deleted_feature = entity_manager.get_feature(&feature.id).await?;
    assert!(deleted_feature.is_none());
    
    // Test delete session
    entity_manager.delete_session(&session.id).await?;
    let deleted_session = entity_manager.get_session(&session.id).await?;
    assert!(deleted_session.is_none());
    
    // Test delete project (should cascade properly)
    entity_manager.delete_project(&project.id).await?;
    let deleted_project = entity_manager.get_project(&project.id).await?;
    assert!(deleted_project.is_none());
    
    Ok(())
}
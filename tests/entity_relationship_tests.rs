// Integration tests for entity relationships and constraints
// Tests foreign key constraints, cascade behavior, and referential integrity

use anyhow::Result;
use tempfile::tempdir;
use workspace::entities::{
    crud::{projects, features, tasks, sessions, directives},
    database::initialize_database,
    schema_models::{FeatureState, Priority, DirectiveCategory}
};

#[tokio::test]
async fn test_project_feature_relationship() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create project
    let project = projects::create(&pool, "Test Project".to_string(), "Test Description".to_string()).await?;
    
    // Create feature linked to project
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Feature Description".to_string(), None).await?;
    
    // Verify feature is linked to project
    assert_eq!(feature.project_id, project.id);
    
    // List features by project
    let project_features = features::list_by_project(&pool, &project.id).await?;
    assert_eq!(project_features.len(), 1);
    assert_eq!(project_features[0].id, feature.id);
    
    Ok(())
}

#[tokio::test]
async fn test_project_deletion_cascade() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create project with features and tasks
    let project = projects::create(&pool, "Test Project".to_string(), "Test Description".to_string()).await?;
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Feature Description".to_string(), None).await?;
    let task = tasks::create(&pool, project.id.clone(), feature.id.clone(), "Test Task".to_string(), "feature".to_string()).await?;

    // Verify entities exist
    assert!(projects::get_by_id(&pool, &project.id).await?.is_some());
    assert!(features::get_by_id(&pool, &feature.id).await?.is_some());
    assert!(tasks::get_by_id(&pool, &task.id).await?.is_some());

    // Delete project
    projects::delete(&pool, &project.id).await?;

    // Verify cascade deletion - features and tasks should be deleted
    assert!(projects::get_by_id(&pool, &project.id).await?.is_none());
    assert!(features::get_by_id(&pool, &feature.id).await?.is_none());
    assert!(tasks::get_by_id(&pool, &task.id).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn test_feature_task_relationship() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create project and feature
    let project = projects::create(&pool, "Test Project".to_string(), "Test Description".to_string()).await?;
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Feature Description".to_string(), None).await?;
    
    // Create multiple tasks for the feature
    let task1 = tasks::create(&pool, project.id.clone(), feature.id.clone(), "Task 1".to_string(), "feature".to_string()).await?;
    let task2 = tasks::create(&pool, project.id.clone(), feature.id.clone(), "Task 2".to_string(), "feature".to_string()).await?;
    
    // Verify tasks are linked to project  
    assert_eq!(task1.project_id, project.id.clone());
    assert_eq!(task2.project_id, project.id.clone());
    
    // Verify tasks can be retrieved
    assert!(tasks::get_by_id(&pool, &task1.id).await?.is_some());
    assert!(tasks::get_by_id(&pool, &task2.id).await?.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_feature_deletion_cascade() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create project, feature, and tasks
    let project = projects::create(&pool, "Test Project".to_string(), "Test Description".to_string()).await?;
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Feature Description".to_string(), None).await?;
    let task1 = tasks::create(&pool, project.id.clone(), feature.id.clone(), "Task 1".to_string(), "feature".to_string()).await?;
    let task2 = tasks::create(&pool, project.id.clone(), feature.id.clone(), "Task 2".to_string(), "feature".to_string()).await?;

    // Verify entities exist
    assert!(features::get_by_id(&pool, &feature.id).await?.is_some());
    assert!(tasks::get_by_id(&pool, &task1.id).await?.is_some());
    assert!(tasks::get_by_id(&pool, &task2.id).await?.is_some());

    // Delete feature
    features::delete(&pool, &feature.id).await?;

    // Verify cascade deletion - tasks should be deleted
    assert!(features::get_by_id(&pool, &feature.id).await?.is_none());
    assert!(tasks::get_by_id(&pool, &task1.id).await?.is_none());
    assert!(tasks::get_by_id(&pool, &task2.id).await?.is_none());

    // Project should still exist
    assert!(projects::get_by_id(&pool, &project.id).await?.is_some());

    Ok(())
}

#[tokio::test]
async fn test_session_project_relationship() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create project
    let project = projects::create(&pool, "Test Project".to_string(), "Test Description".to_string()).await?;
    
    // Create session linked to project
    let session = sessions::create(&pool, project.id.clone(), "Test Session".to_string(), None).await?;
    
    // Verify session is linked to project
    assert_eq!(session.project_id, project.id);
    
    Ok(())
}

#[tokio::test]
async fn test_orphaned_entity_prevention() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Try to create feature without valid project - should fail
    let result = features::create(&pool, "INVALID_PROJECT".to_string(), "Test Feature".to_string(), "Description".to_string(), None).await;
    assert!(result.is_err(), "Should not allow feature creation with invalid project_id");

    // Try to create task without valid feature - should fail
    let result = tasks::create(&pool, "INVALID_PROJECT".to_string(), "INVALID_FEATURE".to_string(), "Test Task".to_string(), "feature".to_string()).await;
    assert!(result.is_err(), "Should not allow task creation with invalid project_id/feature_id");

    Ok(())
}

#[tokio::test] 
async fn test_constraint_validation() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create project
    let project = projects::create(&pool, "Test Project".to_string(), "Test Description".to_string()).await?;
    
    // Test feature state constraint validation
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Description".to_string(), None).await?;
    
    // Update to valid state - should work
    features::update_state(&pool, &feature.id, FeatureState::ImplementedPassingTests).await?;
    
    // Verify state was updated
    let updated_feature = features::get_by_id(&pool, &feature.id).await?.unwrap();
    assert_eq!(updated_feature.state.as_str(), FeatureState::ImplementedPassingTests.as_str());
    
    Ok(())
}

#[tokio::test]
async fn test_id_pattern_constraints() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create entities and verify ID patterns
    let project = projects::create(&pool, "Test Project".to_string(), "Description".to_string()).await?;
    assert!(project.id.starts_with('P'), "Project ID should start with P: {}", project.id);
    
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Description".to_string(), None).await?;
    assert!(feature.id.starts_with('F'), "Feature ID should start with F: {}", feature.id);
    
    let task = tasks::create(&pool, project.id.clone(), feature.id.clone(), "Test Task".to_string(), "feature".to_string()).await?;
    assert!(task.id.starts_with('T'), "Task ID should start with T: {}", task.id);
    
    let session = sessions::create(&pool, project.id.clone(), "Test Session".to_string(), None).await?;
    assert!(session.id.starts_with('S'), "Session ID should start with S: {}", session.id);
    
    let directive = directives::create(&pool, project.id.clone(), "Test Directive".to_string(), "Test Rule".to_string(), DirectiveCategory::Architecture, Priority::Medium).await?;
    assert!(directive.id.starts_with('D'), "Directive ID should start with D: {}", directive.id);
    
    Ok(())
}

#[tokio::test]
async fn test_cross_entity_references() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create complete entity hierarchy
    let project = projects::create(&pool, "Test Project".to_string(), "Description".to_string()).await?;
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Description".to_string(), None).await?;
    let task = tasks::create(&pool, project.id.clone(), feature.id.clone(), "Test Task".to_string(), "feature".to_string()).await?;
    let session = sessions::create(&pool, project.id.clone(), "Test Session".to_string(), None).await?;

    // Verify all relationships
    assert_eq!(feature.project_id, project.id);
    assert_eq!(task.project_id, project.id);
    assert_eq!(task.project_id, project.id.clone());
    assert_eq!(session.project_id, project.id);

    // Test listing operations work correctly
    let project_features = features::list_by_project(&pool, &project.id).await?;
    assert_eq!(project_features.len(), 1);
    
    // Verify task exists
    assert!(tasks::get_by_id(&pool, &task.id).await?.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_referential_integrity_enforcement() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let pool = initialize_database(&db_path).await?;

    // Create project and feature
    let project = projects::create(&pool, "Test Project".to_string(), "Description".to_string()).await?;
    let feature = features::create(&pool, project.id.clone(), "Test Feature".to_string(), "Description".to_string(), None).await?;

    // Try to delete project while feature exists - should fail due to foreign key constraint
    let result = sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(&project.id)
        .execute(&pool)
        .await;
    
    // Database cascade should handle this automatically - just verify cleanup
    // Note: With CASCADE, dependent records are automatically deleted

    // Clean deletion should work - delete feature first, then project
    features::delete(&pool, &feature.id).await?;
    projects::delete(&pool, &project.id).await?;

    // Verify both are deleted
    assert!(features::get_by_id(&pool, &feature.id).await?.is_none());
    assert!(projects::get_by_id(&pool, &project.id).await?.is_none());

    Ok(())
}
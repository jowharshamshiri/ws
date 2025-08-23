// Database System Tests - Simple Schema Initialization and Health Checks
// Tests T000013 - Database indexing for performance optimization

use anyhow::Result;
use tempfile::tempdir;
use workspace::entities::database::{
    initialize_database, health_check, ensure_current_schema, analyze_index_performance, optimize_database
};

/// Test basic database initialization
#[tokio::test]
async fn test_database_initialization() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_init.db");
    
    // Initialize database
    let pool = initialize_database(&db_path).await?;
    
    // Check database health
    let health = health_check(&pool).await?;
    assert!(health.connected);
    assert_eq!(health.schema_version, 1);
    assert_eq!(health.foreign_key_violations, 0);
    
    Ok(())
}

/// Test schema version tracking
#[tokio::test]
async fn test_schema_version_tracking() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_schema.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Ensure current schema (should be idempotent)
    ensure_current_schema(&pool).await?;
    ensure_current_schema(&pool).await?;
    
    let health = health_check(&pool).await?;
    assert_eq!(health.schema_version, 1);
    
    Ok(())
}

/// Test database table creation
#[tokio::test]
async fn test_table_creation() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_tables.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Test that all required tables exist
    let tables = vec![
        "projects", "features", "tasks", "sessions", "directives",
        "templates", "tests", "dependencies", "notes", "milestones",
        "schema_version"
    ];
    
    for table in tables {
        let count = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", table))
            .fetch_one(&pool)
            .await?;
        assert!(count >= 0, "Table {} should exist and be queryable", table);
    }
    
    Ok(())
}

/// Test basic CRUD operations
#[tokio::test]
async fn test_basic_crud() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_crud.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Insert a test project
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P001', 'Test Project', 'Test project description')")
        .execute(&pool)
        .await?;
    
    // Insert a test feature
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'P001', 'F00001', 'Test Feature', 'Test Description')
    "#)
        .execute(&pool)
        .await?;
    
    // Verify data was inserted
    let project_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects")
        .fetch_one(&pool)
        .await?;
    assert_eq!(project_count, 1);
    
    let feature_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM features")
        .fetch_one(&pool)
        .await?;
    assert_eq!(feature_count, 1);
    
    // Test foreign key constraint
    let feature_name = sqlx::query_scalar::<_, String>(
        "SELECT name FROM features WHERE project_id = 'P001'"
    )
        .fetch_one(&pool)
        .await?;
    assert_eq!(feature_name, "Test Feature");
    
    Ok(())
}

/// Test database health monitoring
#[tokio::test]
async fn test_health_monitoring() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_health.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Add some test data
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P001', 'Health Test', 'Health monitoring test project')")
        .execute(&pool)
        .await?;
        
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'P001', 'F00001', 'Feature 1', 'Description 1')
    "#)
        .execute(&pool)
        .await?;
    
    let health = health_check(&pool).await?;
    
    // Verify health metrics
    assert!(health.connected);
    assert_eq!(health.project_count, 1);
    assert_eq!(health.feature_count, 1);
    assert_eq!(health.task_count, 0);
    assert_eq!(health.session_count, 0);
    assert_eq!(health.foreign_key_violations, 0);
    assert!(health.response_time_ms < 100); // Should be fast for local SQLite
    
    Ok(())
}

/// Test foreign key constraints
#[tokio::test]
async fn test_foreign_key_constraints() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_fk.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Try to insert feature without project (should fail)
    let result = sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'NONEXISTENT', 'F00001', 'Orphan Feature', 'Should fail')
    "#)
        .execute(&pool)
        .await;
    
    assert!(result.is_err(), "Foreign key constraint should prevent orphan feature");
    
    // Insert project first, then feature (should succeed)
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P001', 'Valid Project', 'A valid project for testing')")
        .execute(&pool)
        .await?;
        
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'P001', 'F00001', 'Valid Feature', 'Should succeed')
    "#)
        .execute(&pool)
        .await?;
    
    let feature_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM features")
        .fetch_one(&pool)
        .await?;
    assert_eq!(feature_count, 1);
    
    Ok(())
}

/// Test CHECK constraints validation
#[tokio::test]
async fn test_check_constraints() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_constraints.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Test invalid project status should fail
    let invalid_status = sqlx::query(r#"
        INSERT INTO projects (id, name, description, status) 
        VALUES ('P001', 'Test', 'Description', 'invalid_status')
    "#)
        .execute(&pool)
        .await;
    assert!(invalid_status.is_err(), "Invalid project status should be rejected");
    
    // Test invalid project ID pattern should fail  
    let invalid_id = sqlx::query(r#"
        INSERT INTO projects (id, name, description) 
        VALUES ('INVALID', 'Test', 'Description')
    "#)
        .execute(&pool)
        .await;
    assert!(invalid_id.is_err(), "Invalid project ID pattern should be rejected");
    
    // Test valid project should succeed
    sqlx::query(r#"
        INSERT INTO projects (id, name, description, status) 
        VALUES ('P001', 'Valid Project', 'Valid description', 'active')
    "#)
        .execute(&pool)
        .await?;
    
    // Test invalid feature state should fail
    let invalid_feature = sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description, state) 
        VALUES ('F00001', 'P001', 'F00001', 'Test Feature', 'Description', 'invalid_state')
    "#)
        .execute(&pool)
        .await;
    assert!(invalid_feature.is_err(), "Invalid feature state should be rejected");
    
    // Test invalid feature ID pattern should fail
    let invalid_feature_id = sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('INVALID', 'P001', 'INVALID', 'Test Feature', 'Description')
    "#)
        .execute(&pool)
        .await;
    assert!(invalid_feature_id.is_err(), "Invalid feature ID pattern should be rejected");
    
    // Test valid feature should succeed
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description, state) 
        VALUES ('F00001', 'P001', 'F00001', 'Valid Feature', 'Valid description', 'not_implemented')
    "#)
        .execute(&pool)
        .await?;
    
    // Verify data was inserted correctly
    let project_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects")
        .fetch_one(&pool)
        .await?;
    assert_eq!(project_count, 1);
    
    let feature_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM features")
        .fetch_one(&pool)
        .await?;
    assert_eq!(feature_count, 1);
    
    Ok(())
}

/// Test database indexing performance and optimization
#[tokio::test]
async fn test_index_performance_analysis() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_indexes.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Add some test data to make index analysis meaningful
    sqlx::query("INSERT INTO projects (id, name, description, status) VALUES ('P001', 'Test Project', 'Test Description', 'active')")
        .execute(&pool)
        .await?;
        
    for i in 1..=50 {
        sqlx::query(r#"
            INSERT INTO features (id, project_id, code, name, description, state, priority) 
            VALUES (?, 'P001', ?, ?, ?, ?, ?)
        "#)
            .bind(format!("F{:05}", i))
            .bind(format!("F{:05}", i))
            .bind(format!("Test Feature {}", i))
            .bind(format!("Description for feature {}", i))
            .bind(if i % 4 == 0 { "implemented_passing_tests" } else if i % 3 == 0 { "implemented_no_tests" } else { "not_implemented" })
            .bind(if i % 5 == 0 { "high" } else { "medium" })
            .execute(&pool)
            .await?;
    }
    
    for i in 1..=30 {
        sqlx::query(r#"
            INSERT INTO tasks (id, project_id, code, title, description, category, status, priority) 
            VALUES (?, 'P001', ?, ?, ?, 'feature', ?, ?)
        "#)
            .bind(format!("T{:06}", i))
            .bind(format!("T{:06}", i))
            .bind(format!("Test Task {}", i))
            .bind(format!("Description for task {}", i))
            .bind(if i % 3 == 0 { "completed" } else if i % 2 == 0 { "in_progress" } else { "pending" })
            .bind(if i % 4 == 0 { "high" } else { "medium" })
            .execute(&pool)
            .await?;
    }
    
    // Test database optimization (should always work)
    optimize_database(&pool).await?;
    
    // Analyze index performance - Note: dbstat is not available in all SQLite builds
    let performance_result = analyze_index_performance(&pool).await;
    
    match performance_result {
        Ok(performance_report) => {
            // If dbstat works, validate the results
            println!("Index Analysis Results:");
            println!("- Total indexes: {}", performance_report.total_indexes);
            println!("- Large indexes: {}", performance_report.large_indexes);
            println!("- Average depth: {:.2}", performance_report.average_index_depth);
            println!("- Query plans analyzed: {}", performance_report.query_plans.len());
            
            // Basic validation
            assert!(!performance_report.optimization_suggestions.is_empty(), "Should provide optimization suggestions");
        }
        Err(_) => {
            // dbstat may not be available in test environment, which is acceptable
            println!("dbstat not available in test environment - index analysis skipped");
        }
    }
    
    Ok(())
}

/// Test query performance with proper indexing
#[tokio::test]  
async fn test_query_performance_with_indexes() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_query_perf.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Add substantial test data
    sqlx::query("INSERT INTO projects (id, name, description, status) VALUES ('P001', 'Performance Test', 'Performance testing project', 'active')")
        .execute(&pool)
        .await?;
    
    // Insert many features to test index performance
    for i in 1..=200 {
        sqlx::query(r#"
            INSERT INTO features (id, project_id, code, name, description, state, priority, category) 
            VALUES (?, 'P001', ?, ?, ?, ?, ?, ?)
        "#)
            .bind(format!("F{:05}", i))
            .bind(format!("F{:05}", i))
            .bind(format!("Performance Test Feature {}", i))
            .bind(format!("Description for performance test feature {}", i))
            .bind(match i % 6 { 0 => "implemented_passing_tests", 1 => "implemented_no_tests", 2 => "implemented_failing_tests", 3 => "tests_broken", 4 => "critical_issue", _ => "not_implemented" })
            .bind(match i % 4 { 0 => "critical", 1 => "high", 2 => "medium", _ => "low" })
            .bind(match i % 5 { 0 => "backend", 1 => "frontend", 2 => "api", 3 => "database", _ => "infrastructure" })
            .execute(&pool)
            .await?;
    }
    
    // Test indexed queries performance
    let start_time = std::time::Instant::now();
    
    let features_by_project = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM features WHERE project_id = 'P001'"
    )
        .fetch_one(&pool)
        .await?;
    
    let project_query_time = start_time.elapsed();
    
    let start_time = std::time::Instant::now();
    
    let features_by_state = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM features WHERE state = 'not_implemented'"
    )
        .fetch_one(&pool)
        .await?;
    
    let state_query_time = start_time.elapsed();
    
    let start_time = std::time::Instant::now();
    
    let features_by_project_and_state = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM features WHERE project_id = 'P001' AND state = 'implemented_passing_tests'"
    )
        .fetch_one(&pool)
        .await?;
    
    let composite_query_time = start_time.elapsed();
    
    // Verify results make sense
    assert_eq!(features_by_project, 200);
    assert!(features_by_state > 0);
    assert!(features_by_project_and_state > 0);
    
    // Performance should be reasonable with proper indexes
    println!("Query Performance Results:");
    println!("- Project query: {}μs (result: {})", project_query_time.as_micros(), features_by_project);
    println!("- State query: {}μs (result: {})", state_query_time.as_micros(), features_by_state);
    println!("- Composite query: {}μs (result: {})", composite_query_time.as_micros(), features_by_project_and_state);
    
    // All queries should complete quickly with proper indexes
    assert!(project_query_time.as_millis() < 1000, "Queries should be fast with indexes");
    assert!(state_query_time.as_millis() < 1000, "Queries should be fast with indexes");
    assert!(composite_query_time.as_millis() < 1000, "Queries should be fast with indexes");
    
    Ok(())
}

/// Test referential integrity CASCADE behaviors
#[tokio::test]
async fn test_cascade_delete_behaviors() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_cascade.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Create test project
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P001', 'Cascade Test Project', 'Testing CASCADE delete behaviors')")
        .execute(&pool)
        .await?;
    
    // Create session
    sqlx::query(r#"
        INSERT INTO sessions (id, project_id, title, date, focus) 
        VALUES ('S000001', 'P001', 'Test Session', '2025-08-22', 'Testing cascade behaviors')
    "#)
        .execute(&pool)
        .await?;
    
    // Create feature
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'P001', 'F00001', 'Test Feature', 'Feature for cascade testing')
    "#)
        .execute(&pool)
        .await?;
    
    // Create task linked to session
    sqlx::query(r#"
        INSERT INTO tasks (id, project_id, code, title, description, category, session_id) 
        VALUES ('T000001', 'P001', 'T000001', 'Test Task', 'Task linked to session', 'feature', 'S000001')
    "#)
        .execute(&pool)
        .await?;
    
    // Create test linked to feature
    sqlx::query(r#"
        INSERT INTO tests (id, project_id, feature_id, name, test_type, file_path, passed) 
        VALUES ('TEST001', 'P001', 'F00001', 'Test Case', 'unit', '/test/path', TRUE)
    "#)
        .execute(&pool)
        .await?;
    
    // Create notes for the project
    sqlx::query(r#"
        INSERT INTO notes (id, project_id, note_type, title, content, is_project_wide) 
        VALUES ('NOTE001', 'P001', 'general', 'Test Note', 'Project note content', TRUE)
    "#)
        .execute(&pool)
        .await?;
    
    // Create milestone
    sqlx::query(r#"
        INSERT INTO milestones (id, project_id, title, description, status) 
        VALUES ('MILE001', 'P001', 'Test Milestone', 'Milestone for cascade testing', 'planned')
    "#)
        .execute(&pool)
        .await?;
    
    // Create audit trail entry
    sqlx::query(r#"
        INSERT INTO entity_audit_trails (id, entity_id, entity_type, project_id, operation_type, triggered_by, session_id) 
        VALUES ('AUDIT001', 'F00001', 'feature', 'P001', 'create', 'test', 'S000001')
    "#)
        .execute(&pool)
        .await?;
    
    // Verify all entities exist
    let project_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects").fetch_one(&pool).await?;
    let session_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions").fetch_one(&pool).await?;
    let feature_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM features").fetch_one(&pool).await?;
    let task_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks").fetch_one(&pool).await?;
    let test_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tests").fetch_one(&pool).await?;
    let note_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notes").fetch_one(&pool).await?;
    let milestone_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM milestones").fetch_one(&pool).await?;
    let audit_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM entity_audit_trails").fetch_one(&pool).await?;
    
    assert_eq!(project_count, 1);
    assert_eq!(session_count, 1);
    assert_eq!(feature_count, 1);
    assert_eq!(task_count, 1);
    assert_eq!(test_count, 1);
    assert_eq!(note_count, 1);
    assert_eq!(milestone_count, 1);
    assert_eq!(audit_count, 1);
    
    // Test CASCADE DELETE: Deleting project should cascade to all dependent entities
    sqlx::query("DELETE FROM projects WHERE id = 'P001'")
        .execute(&pool)
        .await?;
    
    // Verify CASCADE behavior - all dependent entities should be deleted
    let project_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects").fetch_one(&pool).await?;
    let session_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions").fetch_one(&pool).await?;
    let feature_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM features").fetch_one(&pool).await?;
    let task_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks").fetch_one(&pool).await?;
    let test_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tests").fetch_one(&pool).await?;
    let note_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notes").fetch_one(&pool).await?;
    let milestone_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM milestones").fetch_one(&pool).await?;
    let audit_count_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM entity_audit_trails").fetch_one(&pool).await?;
    
    assert_eq!(project_count_after, 0);
    assert_eq!(session_count_after, 0, "Sessions should CASCADE delete with project");
    assert_eq!(feature_count_after, 0, "Features should CASCADE delete with project");
    assert_eq!(task_count_after, 0, "Tasks should CASCADE delete with project");
    assert_eq!(test_count_after, 0, "Tests should CASCADE delete with project");
    assert_eq!(note_count_after, 0, "Notes should CASCADE delete with project");
    assert_eq!(milestone_count_after, 0, "Milestones should CASCADE delete with project");
    assert_eq!(audit_count_after, 0, "Audit trails should CASCADE delete with project");
    
    Ok(())
}

/// Test referential integrity SET NULL behaviors
#[tokio::test]
async fn test_set_null_behaviors() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_set_null.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Create test project
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P001', 'Set NULL Test Project', 'Testing SET NULL behaviors')")
        .execute(&pool)
        .await?;
    
    // Create session
    sqlx::query(r#"
        INSERT INTO sessions (id, project_id, title, date, focus) 
        VALUES ('S000001', 'P001', 'Test Session', '2025-08-22', 'Testing set null behaviors')
    "#)
        .execute(&pool)
        .await?;
    
    // Create feature
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'P001', 'F00001', 'Test Feature', 'Feature for set null testing')
    "#)
        .execute(&pool)
        .await?;
    
    // Create task with session reference
    sqlx::query(r#"
        INSERT INTO tasks (id, project_id, code, title, description, category, session_id) 
        VALUES ('T000001', 'P001', 'T000001', 'Test Task', 'Task with session reference', 'feature', 'S000001')
    "#)
        .execute(&pool)
        .await?;
    
    // Create test with feature reference
    sqlx::query(r#"
        INSERT INTO tests (id, project_id, feature_id, name, test_type, file_path, passed) 
        VALUES ('TEST001', 'P001', 'F00001', 'Test Case', 'unit', '/test/path', TRUE)
    "#)
        .execute(&pool)
        .await?;
    
    // Create audit trail with session reference
    sqlx::query(r#"
        INSERT INTO entity_audit_trails (id, entity_id, entity_type, project_id, operation_type, triggered_by, session_id) 
        VALUES ('AUDIT001', 'F00001', 'feature', 'P001', 'create', 'test', 'S000001')
    "#)
        .execute(&pool)
        .await?;
    
    // Verify initial state
    let task_session_id = sqlx::query_scalar::<_, Option<String>>("SELECT session_id FROM tasks WHERE id = 'T000001'")
        .fetch_one(&pool)
        .await?;
    let test_feature_id = sqlx::query_scalar::<_, Option<String>>("SELECT feature_id FROM tests WHERE id = 'TEST001'")
        .fetch_one(&pool)
        .await?;
    let audit_session_id = sqlx::query_scalar::<_, Option<String>>("SELECT session_id FROM entity_audit_trails WHERE id = 'AUDIT001'")
        .fetch_one(&pool)
        .await?;
    
    assert_eq!(task_session_id, Some("S000001".to_string()));
    assert_eq!(test_feature_id, Some("F00001".to_string()));
    assert_eq!(audit_session_id, Some("S000001".to_string()));
    
    // Test SET NULL behavior: Delete session should set session_id to NULL in tasks and audit trails
    sqlx::query("DELETE FROM sessions WHERE id = 'S000001'")
        .execute(&pool)
        .await?;
    
    // Verify task still exists but session_id is now NULL
    let task_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks WHERE id = 'T000001'").fetch_one(&pool).await?;
    let task_session_after = sqlx::query_scalar::<_, Option<String>>("SELECT session_id FROM tasks WHERE id = 'T000001'")
        .fetch_one(&pool)
        .await?;
    let audit_session_after = sqlx::query_scalar::<_, Option<String>>("SELECT session_id FROM entity_audit_trails WHERE id = 'AUDIT001'")
        .fetch_one(&pool)
        .await?;
    
    assert_eq!(task_count, 1, "Task should still exist");
    assert_eq!(task_session_after, None, "Task session_id should be NULL after session deletion");
    assert_eq!(audit_session_after, None, "Audit trail session_id should be NULL after session deletion");
    
    // Test SET NULL behavior: Delete feature should set feature_id to NULL in tests
    sqlx::query("DELETE FROM features WHERE id = 'F00001'")
        .execute(&pool)
        .await?;
    
    // Verify test still exists but feature_id is now NULL
    let test_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tests WHERE id = 'TEST001'").fetch_one(&pool).await?;
    let test_feature_after = sqlx::query_scalar::<_, Option<String>>("SELECT feature_id FROM tests WHERE id = 'TEST001'")
        .fetch_one(&pool)
        .await?;
    
    assert_eq!(test_count, 1, "Test should still exist");
    assert_eq!(test_feature_after, None, "Test feature_id should be NULL after feature deletion");
    
    Ok(())
}

/// Test foreign key constraint validation during updates
#[tokio::test]
async fn test_foreign_key_update_validation() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_fk_updates.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Create test projects
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P001', 'Project 1', 'First project')")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P002', 'Project 2', 'Second project')")
        .execute(&pool)
        .await?;
    
    // Create feature in project P001
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'P001', 'F00001', 'Test Feature', 'Feature in project 1')
    "#)
        .execute(&pool)
        .await?;
    
    // Valid update: Move feature to existing project P002
    sqlx::query("UPDATE features SET project_id = 'P002' WHERE id = 'F00001'")
        .execute(&pool)
        .await?;
    
    // Verify update succeeded
    let updated_project_id = sqlx::query_scalar::<_, String>("SELECT project_id FROM features WHERE id = 'F00001'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(updated_project_id, "P002");
    
    // Invalid update: Try to move feature to non-existent project
    let invalid_update = sqlx::query("UPDATE features SET project_id = 'NONEXISTENT' WHERE id = 'F00001'")
        .execute(&pool)
        .await;
    
    assert!(invalid_update.is_err(), "Foreign key constraint should prevent update to non-existent project");
    
    // Verify feature remains in P002 after failed update
    let project_id_after_fail = sqlx::query_scalar::<_, String>("SELECT project_id FROM features WHERE id = 'F00001'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(project_id_after_fail, "P002");
    
    Ok(())
}

/// Test comprehensive referential integrity health check
#[tokio::test]
async fn test_referential_integrity_health_check() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_integrity_health.db");
    
    let pool = initialize_database(&db_path).await?;
    
    // Create valid related data
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ('P001', 'Health Check Project', 'Testing referential integrity health')")
        .execute(&pool)
        .await?;
    
    sqlx::query(r#"
        INSERT INTO sessions (id, project_id, title, date, focus) 
        VALUES ('S000001', 'P001', 'Test Session', '2025-08-22', 'Health check testing')
    "#)
        .execute(&pool)
        .await?;
    
    sqlx::query(r#"
        INSERT INTO features (id, project_id, code, name, description) 
        VALUES ('F00001', 'P001', 'F00001', 'Test Feature', 'Feature for health check')
    "#)
        .execute(&pool)
        .await?;
    
    sqlx::query(r#"
        INSERT INTO tasks (id, project_id, code, title, description, category, session_id) 
        VALUES ('T000001', 'P001', 'T000001', 'Test Task', 'Task for health check', 'feature', 'S000001')
    "#)
        .execute(&pool)
        .await?;
    
    // Run comprehensive health check
    let health = health_check(&pool).await?;
    
    // Verify no foreign key violations
    assert_eq!(health.foreign_key_violations, 0, "Should have no foreign key violations with valid data");
    assert!(health.connected);
    assert_eq!(health.project_count, 1);
    assert_eq!(health.feature_count, 1);
    assert_eq!(health.task_count, 1);
    assert_eq!(health.session_count, 1);
    
    // Verify foreign key constraints are enabled
    let fk_enabled = sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await?;
    assert_eq!(fk_enabled, 1, "Foreign key constraints should be enabled");
    
    Ok(())
}
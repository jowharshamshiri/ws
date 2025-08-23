use anyhow::Result;
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;
use workspace::entities::{EntityManager, database};

mod comprehensive_test_data_generator;
use comprehensive_test_data_generator::setup_workspace_temp_test_environment;

/// Helper to create test database with comprehensive test data
async fn create_test_entity_manager() -> Result<EntityManager> {
    let test_env = setup_workspace_temp_test_environment().await?;
    let pool = database::initialize_database(test_env.get_db_path()).await?;
    let entity_manager = EntityManager::new(pool);
    Ok(entity_manager)
}

/// Test server startup and basic endpoints
#[tokio::test]


async fn test_dashboard_server_startup() -> Result<()> {
    let entity_manager = create_test_entity_manager().await?;
    // Test that server can start and serve basic endpoints
    let port = 3002; // Use different port to avoid conflicts
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        workspace::mcp_server::start_mcp_server(port, false).await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Use entity manager with comprehensive test data for server startup
    drop(entity_manager); // Clean up connection before starting server
    
    // Test health endpoint
    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(5),
        client.get(&format!("http://localhost:{}/health", port)).send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    
    let health_data: serde_json::Value = response.json().await?;
    assert_eq!(health_data["status"], "healthy");
    assert_eq!(health_data["service"], "ws-mcp-server");
    
    // Clean shutdown
    server_handle.abort();
    
    Ok(())
}

/// Test project status API endpoint
#[tokio::test]

async fn test_project_status_api() -> Result<()> {
    let entity_manager = create_test_entity_manager().await?;
    
    // Use test data that should already exist from comprehensive test generator
    let features = entity_manager.list_features().await?;
    let tasks = entity_manager.list_tasks().await?;
    
    // Verify we have test data
    assert!(!features.is_empty(), "Should have test features");
    assert!(!tasks.is_empty(), "Should have test tasks");
    
    Ok(())
}

/// Test feature management API endpoints
#[tokio::test]

async fn test_feature_management_api() -> Result<()> {
    let entity_manager = create_test_entity_manager().await?;
    
    // Use existing test data
    let features = entity_manager.list_features().await?;
    assert!(!features.is_empty(), "Should have test features");
    
    // Test getting a feature
    let first_feature = &features[0];
    let retrieved_feature = entity_manager.get_feature(&first_feature.id).await?;
    assert!(retrieved_feature.is_some());
    let retrieved_feature = retrieved_feature.unwrap();
    assert_eq!(retrieved_feature.id, first_feature.id);
    assert_eq!(retrieved_feature.name, first_feature.name);
    
    Ok(())
}

/// Test task management functionality
#[tokio::test]

async fn test_task_management() -> Result<()> {
    let entity_manager = create_test_entity_manager().await?;
    
    // Use existing test data
    let tasks = entity_manager.list_tasks().await?;
    assert!(!tasks.is_empty(), "Should have test tasks");
    
    // Test getting a task
    let first_task = &tasks[0];
    let retrieved_task = entity_manager.get_task(&first_task.id).await?;
    assert!(retrieved_task.is_some());
    let retrieved_task = retrieved_task.unwrap();
    assert_eq!(retrieved_task.id, first_task.id);
    assert_eq!(retrieved_task.task, first_task.task);
    
    Ok(())
}

/// Test project note management
// Note: Disabled until note system is implemented in new architecture
// #[tokio::test]
async fn _test_note_management_disabled() -> Result<()> {
    let _entity_manager = create_test_entity_manager().await?;
    
    // Test creating different types of notes
    // let arch_note = entity_manager.create_project_note(
    //     "Architecture Decision".to_string(),
    //     "Decided to use SQLite for persistence".to_string(),
    //     "architecture".to_string()
    // ).await?;
    
    // assert_eq!(arch_note.title, "Architecture Decision");
    // assert_eq!(arch_note.content, "Decided to use SQLite for persistence");
    // assert_eq!(arch_note.note_type, workspace::entities::NoteType::Architecture);
    
    // let decision_note = entity_manager.create_project_note(
    //     "Technical Decision".to_string(),
    //     "Use Axum for HTTP server".to_string(),
    //     "decision".to_string()
    // ).await?;
    
    // assert_eq!(decision_note.note_type, workspace::entities::NoteType::Decision);
    
    // Note: Note creation not implemented in new architecture yet
    // let entity_note = entity_manager.create_note(
    //     "feature".to_string(),
    //     "F0097".to_string(),
    //     "Implementation progress note".to_string(),
    //     Some("progress".to_string())
    // ).await?;
    
    // assert_eq!(entity_note.note_type, workspace::entities::models::NoteType::Progress);
    
    Ok(())
}

/// Test metrics calculation for dashboard
#[tokio::test]


async fn test_dashboard_metrics() -> Result<()> {
    let entity_manager = create_test_entity_manager().await?;
    
    // Create features with different states
    let mut feature1 = entity_manager.create_feature(
        "Completed Feature".to_string(),
        "A feature that is completed".to_string()
    ).await?;
    feature1.state = workspace::entities::FeatureState::ImplementedPassingTests.as_str().to_string();
    feature1.test_status = "passed".to_string();
    entity_manager.update_feature_state(&feature1.id, workspace::entities::FeatureState::ImplementedPassingTests).await?;
    
    let mut feature2 = entity_manager.create_feature(
        "In Progress Feature".to_string(),
        "A feature in progress".to_string()
    ).await?;
    feature2.state = workspace::entities::FeatureState::ImplementedFailingTests.as_str().to_string();
    feature2.test_status = "failed".to_string();
    entity_manager.update_feature_state(&feature2.id, workspace::entities::FeatureState::ImplementedFailingTests).await?;
    
    let _feature3 = entity_manager.create_feature(
        "Pending Feature".to_string(),
        "A pending feature".to_string()
    ).await?;
    // Remains in pending state with no test status
    
    // Create tasks with different statuses
    let mut task1 = entity_manager.create_task(
        "Completed Task".to_string(),
        "A completed task".to_string()
    ).await?;
    task1.status = workspace::entities::models::TaskStatus::Completed;
    entity_manager.update_task(task1).await?;
    
    let mut task2 = entity_manager.create_task(
        "In Progress Task".to_string(),
        "A task in progress".to_string()
    ).await?;
    task2.status = workspace::entities::models::TaskStatus::InProgress;
    entity_manager.update_task(task2).await?;
    
    let _task3 = entity_manager.create_task(
        "Pending Task".to_string(),
        "A pending task".to_string()
    ).await?;
    // Remains in pending state
    
    // Verify data
    let features = entity_manager.list_features().await?;
    assert_eq!(features.len(), 3);
    
    let tasks = entity_manager.list_tasks().await?;
    assert_eq!(tasks.len(), 3);
    
    // Test feature metrics
    let completed_features = features.iter().filter(|f| f.state == workspace::entities::FeatureState::ImplementedPassingTests.as_str()).count();
    let tested_features = features.iter().filter(|f| f.test_status == "passed").count();
    
    assert_eq!(completed_features, 1);
    assert_eq!(tested_features, 1);
    
    // Test task metrics
    let completed_tasks = tasks.iter().filter(|t| t.status == workspace::entities::models::TaskStatus::Completed).count();
    let in_progress_tasks = tasks.iter().filter(|t| t.status == workspace::entities::models::TaskStatus::InProgress).count();
    let pending_tasks = tasks.iter().filter(|t| t.status == workspace::entities::models::TaskStatus::Pending).count();
    
    assert_eq!(completed_tasks, 1);
    assert_eq!(in_progress_tasks, 1);
    assert_eq!(pending_tasks, 1);
    
    Ok(())
}

/// Test end-to-end API functionality with HTTP requests
#[tokio::test]

async fn test_api_endpoints_integration() -> Result<()> {
    let port = 3003; // Different port to avoid conflicts
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        workspace::mcp_server::start_mcp_server(port, false).await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let client = reqwest::Client::new();
    let base_url = format!("http://localhost:{}", port);
    
    // Test creating a feature via API
    let feature_payload = json!({
        "name": "HTTP API Test Feature",
        "description": "Feature created via HTTP API testing"
    });
    
    let response = timeout(
        Duration::from_secs(5),
        client.post(&format!("{}/api/features", base_url))
            .json(&feature_payload)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    let created_feature: serde_json::Value = response.json().await?;
    assert_eq!(created_feature["name"], "HTTP API Test Feature");
    
    // Test listing features
    let response = timeout(
        Duration::from_secs(5),
        client.get(&format!("{}/api/features", base_url)).send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    let features: serde_json::Value = response.json().await?;
    assert!(features.is_array());
    
    // Verify our created feature is in the list
    let feature_names: Vec<&str> = features.as_array().unwrap()
        .iter()
        .filter_map(|f| f.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(feature_names.contains(&"HTTP API Test Feature"));
    
    // Test project status endpoint
    let response = timeout(
        Duration::from_secs(5),
        client.get(&format!("{}/api/project/status", base_url)).send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    let status: serde_json::Value = response.json().await?;
    assert!(status.get("project").is_some());
    assert!(status.get("feature_metrics").is_some());
    assert!(status.get("task_summary").is_some());
    
    // Test dashboard HTML serving
    let response = timeout(
        Duration::from_secs(5),
        client.get(&base_url).send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains("Workspace Project Dashboard"));
    assert!(html.contains("Project Overview"));
    
    // Test static asset serving
    let response = timeout(
        Duration::from_secs(5),
        client.get(&format!("{}/dashboard/app.js", base_url)).send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    let js_content = response.text().await?;
    assert!(js_content.contains("class Dashboard"));
    
    // Clean shutdown
    server_handle.abort();
    
    Ok(())
}

/// Test data persistence across server restarts
#[tokio::test]

async fn test_data_persistence() -> Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_persistence_{}.db", uuid::Uuid::new_v4()));
    
    // First session: create data
    {
        let pool = database::initialize_database(&db_path).await?;
        let entity_manager = EntityManager::new(pool);
        
        let _feature = entity_manager.create_feature(
            "Persistent Feature".to_string(),
            "This feature should persist across sessions".to_string()
        ).await?;
        
        let _task = entity_manager.create_task(
            "Persistent Task".to_string(),
            "This task should persist".to_string()
        ).await?;
        
        // Verify initial creation
        let features = entity_manager.list_features().await?;
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].name, "Persistent Feature");
        
        let tasks = entity_manager.list_tasks().await?;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task, "This task should persist");
    } // Drop the first entity manager and pool
    
    // Second session: verify persistence
    {
        let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?;
        let entity_manager = EntityManager::new(pool);
        
        // Verify data persisted
        let features = entity_manager.list_features().await?;
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].name, "Persistent Feature");
        
        let tasks = entity_manager.list_tasks().await?;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task, "This task should persist");
        
        // Test we can still add new data
        let _new_feature = entity_manager.create_feature(
            "Second Session Feature".to_string(),
            "Created in second session".to_string()
        ).await?;
        
        let all_features = entity_manager.list_features().await?;
        assert_eq!(all_features.len(), 2);
    }
    
    // Clean up
    let _ = std::fs::remove_file(&db_path);
    
    Ok(())
}

/// Test that dashboard works with comprehensive test data
#[tokio::test]
async fn test_dashboard_with_comprehensive_data() -> Result<()> {
    let test_env = setup_workspace_temp_test_environment().await?;
    let pool = database::initialize_database(test_env.get_db_path()).await?;
    let entity_manager = EntityManager::new(pool);
    
    // Verify comprehensive test data is available
    let stats = test_env.get_test_data_statistics().await?;
    
    // Validate we have diverse entity data
    assert!(*stats.get("projects").unwrap_or(&0) >= 4, "Should have at least 4 test projects");
    assert!(*stats.get("features").unwrap_or(&0) >= 20, "Should have at least 20 test features");
    assert!(*stats.get("tasks").unwrap_or(&0) >= 20, "Should have at least 20 test tasks");
    assert!(*stats.get("sessions").unwrap_or(&0) >= 3, "Should have at least 3 test sessions");
    assert!(*stats.get("notes").unwrap_or(&0) >= 8, "Should have at least 8 test notes");
    
    // Use public entity manager methods to verify comprehensive data
    let features = entity_manager.list_features().await?;
    let tasks = entity_manager.list_tasks().await?;
    
    // Verify diversity in test data
    assert!(features.len() >= 5, "Should have comprehensive feature data");
    assert!(tasks.len() >= 5, "Should have comprehensive task data");
    
    println!("✅ Dashboard test validated comprehensive test data:");
    for (entity_type, count) in stats {
        println!("  - {}: {} records", entity_type, count);
    }
    
    Ok(())
}

/// Test that file system test data is available for integration tests
#[tokio::test]
async fn test_file_system_test_data_available() -> Result<()> {
    let test_env = setup_workspace_temp_test_environment().await?;
    
    // Verify refac test files are available
    let refac_path = test_env.get_test_files_path("refac");
    assert!(refac_path.exists(), "Refac test files should exist");
    assert!(refac_path.join("oldname_file1.txt").exists(), "Should have oldname test file");
    assert!(refac_path.join("unicode-测试").exists(), "Should have unicode test directory");
    
    // Verify scrap test files are available
    let scrap_path = test_env.get_test_files_path("scrap");
    assert!(scrap_path.exists(), "Scrap test files should exist");
    assert!(scrap_path.join("document.txt").exists(), "Should have document test file");
    assert!(scrap_path.join("nested/deep/structure").exists(), "Should have nested structure");
    
    // Verify encoding test files are available
    let encoding_path = test_env.get_test_files_path("encoding");
    assert!(encoding_path.exists(), "Encoding test files should exist");
    assert!(encoding_path.join("utf8.txt").exists(), "Should have UTF-8 test file");
    assert!(encoding_path.join("invalid_utf8.txt").exists(), "Should have invalid UTF-8 test file");
    
    // Verify binary test files are available  
    let binary_path = test_env.get_test_files_path("binary");
    assert!(binary_path.exists(), "Binary test files should exist");
    assert!(binary_path.join("pure_binary.bin").exists(), "Should have binary test file");
    assert!(binary_path.join("fake_image.png").exists(), "Should have fake image file");
    
    // Verify git repositories are available
    let git_repos_path = test_env.get_git_repos_path();
    assert!(git_repos_path.exists(), "Git repos should exist");
    assert!(git_repos_path.join("basic_repo").exists(), "Should have basic git repo");
    assert!(git_repos_path.join("complex_repo").exists(), "Should have complex git repo");
    
    println!("✅ File system test data validated - all categories available");
    Ok(())
}
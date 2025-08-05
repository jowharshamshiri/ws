// Database Schema and Migration System for Workspace Entity Management

use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::path::Path;

/// Initialize SQLite database with all required tables and indexes
pub async fn initialize_database(db_path: &Path) -> Result<SqlitePool> {
    let database_url = format!("sqlite:{}", db_path.display());
    
    // Create database if it doesn't exist
    if !Sqlite::database_exists(&database_url).await.unwrap_or(false) {
        Sqlite::create_database(&database_url).await?;
        log::info!("Created workspace database at {}", db_path.display());
    }
    
    let pool = SqlitePool::connect(&database_url).await?;
    
    // Initialize all tables
    initialize_tables(&pool).await?;
    
    Ok(pool)
}

/// Create all required tables with proper constraints and indexes
pub async fn initialize_tables(pool: &SqlitePool) -> Result<()> {
    // Enable foreign key constraints
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await?;

    // Projects table - root container
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            repository_url TEXT,
            version TEXT NOT NULL DEFAULT '0.1.0',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            archived BOOLEAN NOT NULL DEFAULT FALSE,
            metadata TEXT
        )
    "#)
    .execute(pool)
    .await?;

    // Features table - central capability tracking
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS features (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            code TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            category TEXT,
            state TEXT NOT NULL DEFAULT 'not_implemented',
            test_status TEXT NOT NULL DEFAULT 'not_tested',
            priority TEXT NOT NULL DEFAULT 'medium',
            implementation_notes TEXT,
            test_evidence TEXT,
            dependencies TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT,
            estimated_effort INTEGER,
            actual_effort INTEGER,
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            UNIQUE (project_id, code)
        )
    "#)
    .execute(pool)
    .await?;

    // Tasks table - work items with feature integration
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            code TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            category TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            priority TEXT NOT NULL DEFAULT 'medium',
            feature_ids TEXT,
            depends_on TEXT,
            acceptance_criteria TEXT,
            validation_steps TEXT,
            evidence TEXT,
            session_id TEXT,
            assigned_to TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            started_at TEXT,
            completed_at TEXT,
            estimated_effort INTEGER,
            actual_effort INTEGER,
            tags TEXT,
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE SET NULL,
            UNIQUE (project_id, code)
        )
    "#)
    .execute(pool)
    .await?;

    // Sessions table - development session tracking
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            state TEXT NOT NULL DEFAULT 'active',
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            ended_at TEXT,
            summary TEXT,
            achievements TEXT,
            files_modified TEXT,
            features_worked TEXT,
            tasks_completed TEXT,
            next_priority TEXT,
            reminder TEXT,
            validation_evidence TEXT,
            context_remaining REAL,
            commit_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Directives table - persistent rules and constraints
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS directives (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            code TEXT NOT NULL,
            title TEXT NOT NULL,
            rule TEXT NOT NULL,
            category TEXT NOT NULL,
            priority TEXT NOT NULL DEFAULT 'medium',
            context TEXT,
            rationale TEXT,
            examples TEXT,
            violations TEXT,
            override_behavior TEXT,
            active BOOLEAN NOT NULL DEFAULT TRUE,
            compliance_checked TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            archived_at TEXT,
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            UNIQUE (project_id, code)
        )
    "#)
    .execute(pool)
    .await?;

    // Templates table - Tera template management
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS templates (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            content TEXT NOT NULL,
            output_path TEXT,
            enabled BOOLEAN NOT NULL DEFAULT FALSE,
            variables TEXT,
            last_rendered TEXT,
            render_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            UNIQUE (project_id, name)
        )
    "#)
    .execute(pool)
    .await?;

    // Tests table - test results linked to features
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS tests (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            feature_id TEXT,
            name TEXT NOT NULL,
            description TEXT,
            test_type TEXT NOT NULL,
            file_path TEXT NOT NULL,
            function_name TEXT,
            passed BOOLEAN NOT NULL,
            output TEXT,
            error_message TEXT,
            duration_ms INTEGER,
            is_tautological BOOLEAN NOT NULL DEFAULT FALSE,
            coverage_percentage REAL,
            run_at TEXT NOT NULL DEFAULT (datetime('now')),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            FOREIGN KEY (feature_id) REFERENCES features (id) ON DELETE SET NULL
        )
    "#)
    .execute(pool)
    .await?;

    // Dependencies table - entity relationships
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS dependencies (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            from_entity_id TEXT NOT NULL,
            from_entity_type TEXT NOT NULL,
            to_entity_id TEXT NOT NULL,
            to_entity_type TEXT NOT NULL,
            dependency_type TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            resolved_at TEXT,
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Notes table - attachable to any entity or project-wide
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            entity_id TEXT,
            entity_type TEXT,
            note_type TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT,
            author TEXT,
            is_project_wide BOOLEAN NOT NULL DEFAULT FALSE,
            is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Milestones table - project milestones with feature linkage
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS milestones (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            target_date TEXT,
            achieved_date TEXT,
            status TEXT NOT NULL DEFAULT 'planned',
            feature_ids TEXT,
            success_criteria TEXT,
            completion_percentage REAL NOT NULL DEFAULT 0.0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Feature state transitions table - audit trail
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS feature_state_transitions (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            from_state TEXT NOT NULL,
            to_state TEXT NOT NULL,
            evidence TEXT,
            notes TEXT,
            triggered_by TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (feature_id) REFERENCES features (id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Session metrics table - comprehensive timeseries tracking
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS session_metrics (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            session_duration_seconds INTEGER NOT NULL DEFAULT 0,
            total_messages INTEGER NOT NULL DEFAULT 0,
            tool_calls INTEGER NOT NULL DEFAULT 0,
            context_usage_tokens INTEGER NOT NULL DEFAULT 0,
            average_response_time_ms INTEGER NOT NULL DEFAULT 0,
            peak_response_time_ms INTEGER NOT NULL DEFAULT 0,
            total_tool_calls INTEGER NOT NULL DEFAULT 0,
            total_response_time_ms INTEGER NOT NULL DEFAULT 0,
            context_used INTEGER NOT NULL DEFAULT 0,
            session_duration_ms INTEGER NOT NULL DEFAULT 0,
            features_created INTEGER NOT NULL DEFAULT 0,
            features_updated INTEGER NOT NULL DEFAULT 0,
            tasks_created INTEGER NOT NULL DEFAULT 0,
            tasks_completed INTEGER NOT NULL DEFAULT 0,
            files_modified INTEGER NOT NULL DEFAULT 0,
            issues_resolved INTEGER NOT NULL DEFAULT 0,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Entity audit trails table for F0131 Entity State Tracking
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS entity_audit_trails (
            id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            project_id TEXT NOT NULL,
            operation_type TEXT NOT NULL,
            field_changed TEXT,
            old_value TEXT,
            new_value TEXT,
            change_reason TEXT,
            triggered_by TEXT NOT NULL,
            session_id TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE SET NULL
        )
    "#)
    .execute(pool)
    .await?;

    // Create indexes for performance
    create_indexes(pool).await?;

    log::info!("Database tables initialized successfully");
    Ok(())
}

/// Create indexes for optimized queries
async fn create_indexes(pool: &SqlitePool) -> Result<()> {
    let indexes = vec![
        // Project indexes
        "CREATE INDEX IF NOT EXISTS idx_projects_name ON projects (name)",
        "CREATE INDEX IF NOT EXISTS idx_projects_archived ON projects (archived)",
        
        // Feature indexes
        "CREATE INDEX IF NOT EXISTS idx_features_project_id ON features (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_features_code ON features (code)",
        "CREATE INDEX IF NOT EXISTS idx_features_state ON features (state)",
        "CREATE INDEX IF NOT EXISTS idx_features_priority ON features (priority)",
        "CREATE INDEX IF NOT EXISTS idx_features_category ON features (category)",
        
        // Task indexes
        "CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_code ON tasks (code)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks (priority)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_category ON tasks (category)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_session_id ON tasks (session_id)",
        
        // Session indexes
        "CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_state ON sessions (state)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions (started_at)",
        
        // Directive indexes
        "CREATE INDEX IF NOT EXISTS idx_directives_project_id ON directives (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_directives_category ON directives (category)",
        "CREATE INDEX IF NOT EXISTS idx_directives_priority ON directives (priority)",
        "CREATE INDEX IF NOT EXISTS idx_directives_active ON directives (active)",
        
        // Template indexes
        "CREATE INDEX IF NOT EXISTS idx_templates_project_id ON templates (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_templates_enabled ON templates (enabled)",
        
        // Test indexes
        "CREATE INDEX IF NOT EXISTS idx_tests_project_id ON tests (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_tests_feature_id ON tests (feature_id)",
        "CREATE INDEX IF NOT EXISTS idx_tests_passed ON tests (passed)",
        "CREATE INDEX IF NOT EXISTS idx_tests_is_tautological ON tests (is_tautological)",
        "CREATE INDEX IF NOT EXISTS idx_tests_run_at ON tests (run_at)",
        
        // Dependency indexes
        "CREATE INDEX IF NOT EXISTS idx_dependencies_project_id ON dependencies (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_dependencies_from_entity ON dependencies (from_entity_id, from_entity_type)",
        "CREATE INDEX IF NOT EXISTS idx_dependencies_to_entity ON dependencies (to_entity_id, to_entity_type)",
        
        // Note indexes
        "CREATE INDEX IF NOT EXISTS idx_notes_project_id ON notes (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_notes_entity ON notes (entity_id, entity_type)",
        
        // Milestone indexes
        "CREATE INDEX IF NOT EXISTS idx_milestones_project_id ON milestones (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_milestones_status ON milestones (status)",
        "CREATE INDEX IF NOT EXISTS idx_milestones_target_date ON milestones (target_date)",
        "CREATE INDEX IF NOT EXISTS idx_milestones_completion ON milestones (completion_percentage)",
        "CREATE INDEX IF NOT EXISTS idx_notes_note_type ON notes (note_type)",
        "CREATE INDEX IF NOT EXISTS idx_notes_is_project_wide ON notes (is_project_wide)",
        "CREATE INDEX IF NOT EXISTS idx_notes_is_pinned ON notes (is_pinned)",
        
        // Session metrics indexes
        "CREATE INDEX IF NOT EXISTS idx_session_metrics_session_id ON session_metrics (session_id)",
        "CREATE INDEX IF NOT EXISTS idx_session_metrics_timestamp ON session_metrics (timestamp)",
        
        // State transition indexes
        "CREATE INDEX IF NOT EXISTS idx_state_transitions_feature_id ON feature_state_transitions (feature_id)",
        "CREATE INDEX IF NOT EXISTS idx_state_transitions_timestamp ON feature_state_transitions (timestamp)",
        
        // Full-text search indexes
        "CREATE INDEX IF NOT EXISTS idx_features_search ON features (name, description)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_search ON tasks (title, description)",
        "CREATE INDEX IF NOT EXISTS idx_notes_search ON notes (title, content)",
        
        // Audit trail indexes for F0131
        "CREATE INDEX IF NOT EXISTS idx_audit_entity ON entity_audit_trails (entity_id, entity_type)",
        "CREATE INDEX IF NOT EXISTS idx_audit_project ON entity_audit_trails (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON entity_audit_trails (timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_audit_operation ON entity_audit_trails (operation_type)",
        "CREATE INDEX IF NOT EXISTS idx_audit_triggered_by ON entity_audit_trails (triggered_by)",
        "CREATE INDEX IF NOT EXISTS idx_audit_session ON entity_audit_trails (session_id)",
    ];

    for index_sql in indexes {
        sqlx::query(index_sql).execute(pool).await?;
    }

    log::info!("Database indexes created successfully");
    Ok(())
}

/// Database migration system for schema changes
pub async fn migrate_database(pool: &SqlitePool, current_version: i32) -> Result<()> {
    // Check current schema version
    let version = get_schema_version(pool).await?;
    
    if version < current_version {
        log::info!("Migrating database from version {} to {}", version, current_version);
        
        // Apply migrations in sequence
        for migration_version in (version + 1)..=current_version {
            apply_migration(pool, migration_version).await?;
        }
        
        set_schema_version(pool, current_version).await?;
        log::info!("Database migration completed");
    }
    
    Ok(())
}

/// Get current schema version
async fn get_schema_version(pool: &SqlitePool) -> Result<i32> {
    // Create schema_version table if it doesn't exist
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
    "#)
    .execute(pool)
    .await?;

    let result = sqlx::query_scalar::<_, i32>("SELECT MAX(version) FROM schema_version")
        .fetch_optional(pool)
        .await?;
    
    Ok(result.unwrap_or(0))
}

/// Set schema version
async fn set_schema_version(pool: &SqlitePool, version: i32) -> Result<()> {
    sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
        .bind(version)
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Apply specific migration
async fn apply_migration(pool: &SqlitePool, version: i32) -> Result<()> {
    log::info!("Applying migration version {}", version);
    
    match version {
        1 => {
            // Initial schema - already handled by initialize_tables
            Ok(())
        }
        2 => {
            // Example future migration
            sqlx::query("ALTER TABLE features ADD COLUMN tags TEXT")
                .execute(pool)
                .await?;
            Ok(())
        }
        _ => {
            log::warn!("Unknown migration version: {}", version);
            Ok(())
        }
    }
}

/// Vacuum and optimize database
pub async fn optimize_database(pool: &SqlitePool) -> Result<()> {
    sqlx::query("VACUUM").execute(pool).await?;
    sqlx::query("ANALYZE").execute(pool).await?;
    log::info!("Database optimized");
    Ok(())
}

/// Database health check
pub async fn health_check(pool: &SqlitePool) -> Result<DatabaseHealth> {
    let start_time = std::time::Instant::now();
    
    // Test basic connectivity
    let connection_test = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await;
    
    let response_time = start_time.elapsed();
    
    // Get table counts
    let project_count = count_table(pool, "projects").await?;
    let feature_count = count_table(pool, "features").await?;
    let task_count = count_table(pool, "tasks").await?;
    let session_count = count_table(pool, "sessions").await?;
    let note_count = count_table(pool, "notes").await?;
    
    // Check for foreign key constraint violations
    let fk_violations = sqlx::query_scalar::<_, i32>("PRAGMA foreign_key_check")
        .fetch_all(pool)
        .await?
        .len();
    
    Ok(DatabaseHealth {
        connected: connection_test.is_ok(),
        response_time_ms: response_time.as_millis() as u64,
        project_count,
        feature_count,
        task_count,
        session_count,
        note_count,
        foreign_key_violations: fk_violations,
        schema_version: get_schema_version(pool).await?,
    })
}

async fn count_table(pool: &SqlitePool, table: &str) -> Result<i64> {
    let sql = format!("SELECT COUNT(*) FROM {}", table);
    let count = sqlx::query_scalar::<_, i64>(&sql)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub response_time_ms: u64,
    pub project_count: i64,
    pub feature_count: i64,
    pub task_count: i64,
    pub session_count: i64,
    pub note_count: i64,
    pub foreign_key_violations: usize,
    pub schema_version: i32,
}
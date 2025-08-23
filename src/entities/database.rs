// Database Schema and Migration System for Workspace Entity Management

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

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
    
    // Initialize session continuity tables
    initialize_continuity_tables(&pool).await?;
    
    // Ensure current schema version
    ensure_current_schema(&pool).await?;
    
    Ok(pool)
}

/// Create all required tables with proper constraints and indexes
pub async fn initialize_tables(pool: &SqlitePool) -> Result<()> {
    // Enable foreign key constraints
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await?;

    // Projects table - root container with proper constraints
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            current_phase TEXT,
            repository_url TEXT,
            version TEXT NOT NULL DEFAULT '0.1.0',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            archived BOOLEAN NOT NULL DEFAULT FALSE,
            metadata TEXT,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_projects_status CHECK (status IN ('active', 'paused', 'completed', 'archived')),
            CONSTRAINT chk_projects_id_pattern CHECK (id GLOB 'P[0-9][0-9][0-9]')
        )
    "#)
    .execute(pool)
    .await?;

    // Features table - central capability tracking with proper constraints
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
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT,
            estimated_effort INTEGER,
            actual_effort INTEGER,
            metadata TEXT,
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            
            -- Unique constraints
            UNIQUE (project_id, code),
            
            -- Check constraints for data integrity
            CONSTRAINT chk_features_state CHECK (state IN (
                'not_implemented', 'implemented_no_tests', 'implemented_failing_tests',
                'implemented_passing_tests', 'tests_broken', 'critical_issue'
            )),
            CONSTRAINT chk_features_test_status CHECK (test_status IN (
                'not_tested', 'failing', 'passing', 'broken', 'tautological'
            )),
            CONSTRAINT chk_features_priority CHECK (priority IN ('critical', 'high', 'medium', 'low')),
            CONSTRAINT chk_features_id_pattern CHECK (id GLOB 'F[0-9][0-9][0-9][0-9][0-9]'),
            CONSTRAINT chk_features_code_pattern CHECK (code GLOB 'F[0-9][0-9][0-9][0-9][0-9]'),
            CONSTRAINT chk_features_notes_length CHECK (LENGTH(notes) <= 100 OR notes IS NULL)
        )
    "#)
    .execute(pool)
    .await?;

    // Tasks table - work items with feature integration and proper constraints
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
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            started_at TEXT,
            completed_at TEXT,
            estimated_effort INTEGER,
            actual_effort INTEGER,
            tags TEXT,
            metadata TEXT,
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE SET NULL,
            
            -- Unique constraints
            UNIQUE (project_id, code),
            
            -- Check constraints for data integrity
            CONSTRAINT chk_tasks_status CHECK (status IN (
                'pending', 'in_progress', 'blocked', 'completed', 'cancelled'
            )),
            CONSTRAINT chk_tasks_priority CHECK (priority IN ('critical', 'high', 'medium', 'low')),
            CONSTRAINT chk_tasks_category CHECK (category IN (
                'feature', 'bug', 'refactor', 'testing', 'documentation', 'infrastructure', 'api', 'migration'
            )),
            CONSTRAINT chk_tasks_id_pattern CHECK (id GLOB 'T[0-9][0-9][0-9][0-9][0-9][0-9]'),
            CONSTRAINT chk_tasks_code_pattern CHECK (code GLOB 'T[0-9][0-9][0-9][0-9][0-9][0-9]'),
            CONSTRAINT chk_tasks_notes_length CHECK (LENGTH(notes) <= 100 OR notes IS NULL)
        )
    "#)
    .execute(pool)
    .await?;

    // Sessions table - development session tracking with proper constraints
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            state TEXT NOT NULL DEFAULT 'active',
            date TEXT NOT NULL,
            start_time TEXT,
            end_time TEXT,
            focus TEXT NOT NULL,
            major_achievement TEXT,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            ended_at TEXT,
            summary TEXT,
            completed_tasks TEXT,
            key_achievements TEXT,
            files_modified TEXT,
            issues_resolved TEXT,
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_sessions_state CHECK (state IN ('active', 'completed', 'cancelled')),
            CONSTRAINT chk_sessions_id_pattern CHECK (id GLOB 'S[0-9][0-9][0-9][0-9][0-9][0-9]')
        )
    "#)
    .execute(pool)
    .await?;

    // Directives table - persistent rules and constraints with proper validation
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS directives (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            code TEXT NOT NULL,
            title TEXT NOT NULL,
            rule TEXT NOT NULL,
            category TEXT NOT NULL,
            priority TEXT NOT NULL DEFAULT 'medium',
            status TEXT NOT NULL DEFAULT 'active',
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            
            -- Unique constraints
            UNIQUE (project_id, code),
            
            -- Check constraints for data integrity
            CONSTRAINT chk_directives_priority CHECK (priority IN ('critical', 'high', 'medium', 'low')),
            CONSTRAINT chk_directives_status CHECK (status IN ('active', 'inactive', 'archived')),
            CONSTRAINT chk_directives_id_pattern CHECK (id GLOB 'D[0-9][0-9][0-9]'),
            CONSTRAINT chk_directives_code_pattern CHECK (code GLOB 'D[0-9][0-9][0-9]')
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

    // Tests table - test results linked to features with proper constraints
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            FOREIGN KEY (feature_id) REFERENCES features (id) ON DELETE SET NULL,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_tests_test_type CHECK (test_type IN (
                'unit', 'integration', 'end_to_end', 'performance', 'security', 'manual'
            )),
            CONSTRAINT chk_tests_coverage CHECK (coverage_percentage >= 0 AND coverage_percentage <= 100 OR coverage_percentage IS NULL),
            CONSTRAINT chk_tests_duration CHECK (duration_ms >= 0 OR duration_ms IS NULL)
        )
    "#)
    .execute(pool)
    .await?;

    // Dependencies table - entity relationships with proper constraints
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_dependencies_entity_types CHECK (
                from_entity_type IN ('project', 'feature', 'task', 'session', 'directive') AND
                to_entity_type IN ('project', 'feature', 'task', 'session', 'directive')
            ),
            CONSTRAINT chk_dependencies_type CHECK (dependency_type IN (
                'blocks', 'requires', 'implements', 'tests', 'documents', 'references'
            )),
            CONSTRAINT chk_dependencies_not_self CHECK (from_entity_id != to_entity_id OR from_entity_type != to_entity_type)
        )
    "#)
    .execute(pool)
    .await?;

    // Notes table - attachable to any entity or project-wide with proper constraints
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_notes_note_type CHECK (note_type IN (
                'general', 'implementation', 'testing', 'bug', 'feature_request', 'technical_debt', 'decision'
            )),
            CONSTRAINT chk_notes_entity_type CHECK (
                entity_type IN ('project', 'feature', 'task', 'session', 'directive') OR entity_type IS NULL
            ),
            CONSTRAINT chk_notes_entity_consistency CHECK (
                (entity_id IS NULL AND entity_type IS NULL AND is_project_wide = TRUE) OR
                (entity_id IS NOT NULL AND entity_type IS NOT NULL AND is_project_wide = FALSE)
            )
        )
    "#)
    .execute(pool)
    .await?;

    // Milestones table - project milestones with feature linkage and proper constraints
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_milestones_status CHECK (status IN (
                'planned', 'in_progress', 'achieved', 'missed', 'cancelled'
            )),
            CONSTRAINT chk_milestones_completion CHECK (
                completion_percentage >= 0.0 AND completion_percentage <= 100.0
            ),
            CONSTRAINT chk_milestones_dates CHECK (
                achieved_date IS NULL OR target_date IS NULL OR achieved_date >= target_date OR status != 'achieved'
            )
        )
    "#)
    .execute(pool)
    .await?;

    // Feature state transitions table - audit trail with proper constraints
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (feature_id) REFERENCES features (id) ON DELETE CASCADE,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_transitions_states CHECK (
                from_state IN ('not_implemented', 'implemented_no_tests', 'implemented_failing_tests',
                              'implemented_passing_tests', 'tests_broken', 'critical_issue') AND
                to_state IN ('not_implemented', 'implemented_no_tests', 'implemented_failing_tests',
                            'implemented_passing_tests', 'tests_broken', 'critical_issue')
            ),
            CONSTRAINT chk_transitions_different CHECK (from_state != to_state)
        )
    "#)
    .execute(pool)
    .await?;

    // Session metrics table - comprehensive timeseries tracking with constraints
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_metrics_non_negative CHECK (
                session_duration_seconds >= 0 AND total_messages >= 0 AND tool_calls >= 0 AND
                context_usage_tokens >= 0 AND average_response_time_ms >= 0 AND
                peak_response_time_ms >= 0 AND features_created >= 0 AND features_updated >= 0 AND
                tasks_created >= 0 AND tasks_completed >= 0 AND files_modified >= 0 AND issues_resolved >= 0
            ),
            CONSTRAINT chk_metrics_peak_avg CHECK (peak_response_time_ms >= average_response_time_ms)
        )
    "#)
    .execute(pool)
    .await?;

    // Entity audit trails table for F0131 Entity State Tracking with proper constraints
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
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE SET NULL,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_audit_entity_type CHECK (entity_type IN (
                'project', 'feature', 'task', 'session', 'directive', 'template', 'test', 'note', 'milestone'
            )),
            CONSTRAINT chk_audit_operation CHECK (operation_type IN (
                'create', 'update', 'delete', 'state_change', 'relationship_change'
            ))
        )
    "#)
    .execute(pool)
    .await?;

    // Note links table for F0137 Note Linking and References with proper constraints
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS note_links (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            source_note_id TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id TEXT NOT NULL,
            target_entity_type TEXT,
            link_type TEXT NOT NULL,
            auto_detected BOOLEAN NOT NULL DEFAULT FALSE,
            detection_reason TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT,
            
            -- Foreign Key Constraints
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            FOREIGN KEY (source_note_id) REFERENCES notes (id) ON DELETE CASCADE,
            
            -- Check constraints for data integrity
            CONSTRAINT chk_note_links_target_type CHECK (target_type IN (
                'entity', 'url', 'file', 'code_reference'
            )),
            CONSTRAINT chk_note_links_entity_type CHECK (
                target_entity_type IN ('project', 'feature', 'task', 'session', 'directive', 'template', 'test', 'milestone') OR
                target_entity_type IS NULL
            ),
            CONSTRAINT chk_note_links_link_type CHECK (link_type IN (
                'references', 'implements', 'blocks', 'relates_to', 'documents', 'tests'
            )),
            CONSTRAINT chk_note_links_consistency CHECK (
                (target_type = 'entity' AND target_entity_type IS NOT NULL) OR
                (target_type != 'entity' AND target_entity_type IS NULL)
            )
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
        // Project indexes - Enhanced for common query patterns
        "CREATE INDEX IF NOT EXISTS idx_projects_name ON projects (name)",
        "CREATE INDEX IF NOT EXISTS idx_projects_archived ON projects (archived)",
        "CREATE INDEX IF NOT EXISTS idx_projects_status ON projects (status)",
        "CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects (created_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_projects_updated_at ON projects (updated_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_projects_active_name ON projects (status, name) WHERE status = 'active'",
        "CREATE INDEX IF NOT EXISTS idx_projects_version ON projects (version)",
        
        // Feature indexes - Optimized for state machine operations and queries
        "CREATE INDEX IF NOT EXISTS idx_features_project_id ON features (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_features_code ON features (code)",
        "CREATE INDEX IF NOT EXISTS idx_features_state ON features (state)",
        "CREATE INDEX IF NOT EXISTS idx_features_priority ON features (priority)",
        "CREATE INDEX IF NOT EXISTS idx_features_category ON features (category)",
        "CREATE INDEX IF NOT EXISTS idx_features_test_status ON features (test_status)",
        "CREATE INDEX IF NOT EXISTS idx_features_completed_at ON features (completed_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_features_created_at ON features (created_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_features_updated_at ON features (updated_at DESC)",
        // Composite indexes for common filtering patterns
        "CREATE INDEX IF NOT EXISTS idx_features_project_state ON features (project_id, state)",
        "CREATE INDEX IF NOT EXISTS idx_features_project_priority ON features (project_id, priority)",
        "CREATE INDEX IF NOT EXISTS idx_features_state_priority ON features (state, priority)",
        "CREATE INDEX IF NOT EXISTS idx_features_category_state ON features (category, state)",
        // Performance indexes for dashboard queries
        "CREATE INDEX IF NOT EXISTS idx_features_not_implemented ON features (project_id, created_at) WHERE state = 'not_implemented'",
        "CREATE INDEX IF NOT EXISTS idx_features_in_progress ON features (project_id, updated_at) WHERE state IN ('implemented_no_tests', 'implemented_failing_tests')",
        "CREATE INDEX IF NOT EXISTS idx_features_completed ON features (project_id, completed_at) WHERE state = 'implemented_passing_tests'",
        
        // Task indexes - Optimized for work management and kanban board queries
        "CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_code ON tasks (code)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks (priority)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_category ON tasks (category)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_session_id ON tasks (session_id)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_assigned_to ON tasks (assigned_to)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks (created_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_updated_at ON tasks (updated_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_started_at ON tasks (started_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_completed_at ON tasks (completed_at DESC)",
        // Composite indexes for kanban and dashboard queries
        "CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON tasks (project_id, status)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_project_priority ON tasks (project_id, priority)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_status_priority ON tasks (status, priority)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_category_status ON tasks (category, status)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_assignee_status ON tasks (assigned_to, status)",
        // Performance indexes for active work tracking
        "CREATE INDEX IF NOT EXISTS idx_tasks_active ON tasks (project_id, updated_at) WHERE status IN ('pending', 'in_progress')",
        "CREATE INDEX IF NOT EXISTS idx_tasks_overdue ON tasks (project_id, created_at) WHERE status = 'pending'",
        "CREATE INDEX IF NOT EXISTS idx_tasks_completed_recent ON tasks (project_id, completed_at) WHERE status = 'completed'",
        
        // Session indexes - Optimized for activity tracking and analytics
        "CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_state ON sessions (state)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions (started_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_ended_at ON sessions (ended_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_date ON sessions (date DESC)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_focus ON sessions (focus)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions (created_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions (updated_at DESC)",
        // Composite indexes for session analytics
        "CREATE INDEX IF NOT EXISTS idx_sessions_project_date ON sessions (project_id, date DESC)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_project_state ON sessions (project_id, state)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_date_focus ON sessions (date DESC, focus)",
        // Performance indexes for recent activity queries
        "CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions (project_id, started_at) WHERE state = 'active'",
        "CREATE INDEX IF NOT EXISTS idx_sessions_recent ON sessions (project_id, date) WHERE state = 'completed'",
        
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
        
        // Full-text search indexes - Enhanced for search functionality
        "CREATE INDEX IF NOT EXISTS idx_features_search ON features (name, description)",
        "CREATE INDEX IF NOT EXISTS idx_features_name_lower ON features (LOWER(name))",
        "CREATE INDEX IF NOT EXISTS idx_tasks_search ON tasks (title, description)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_title_lower ON tasks (LOWER(title))",
        "CREATE INDEX IF NOT EXISTS idx_notes_search ON notes (title, content)",
        "CREATE INDEX IF NOT EXISTS idx_notes_title_lower ON notes (LOWER(title))",
        "CREATE INDEX IF NOT EXISTS idx_projects_name_lower ON projects (LOWER(name))",
        // Tag-based search indexes
        "CREATE INDEX IF NOT EXISTS idx_tasks_tags ON tasks (tags)",
        "CREATE INDEX IF NOT EXISTS idx_notes_tags ON notes (tags)",
        
        // Audit trail indexes for F0131
        "CREATE INDEX IF NOT EXISTS idx_audit_entity ON entity_audit_trails (entity_id, entity_type)",
        "CREATE INDEX IF NOT EXISTS idx_audit_project ON entity_audit_trails (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON entity_audit_trails (timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_audit_operation ON entity_audit_trails (operation_type)",
        "CREATE INDEX IF NOT EXISTS idx_audit_triggered_by ON entity_audit_trails (triggered_by)",
        "CREATE INDEX IF NOT EXISTS idx_audit_session ON entity_audit_trails (session_id)",
        
        // Note link indexes for F0137 - Enhanced for relationship queries
        "CREATE INDEX IF NOT EXISTS idx_note_links_source ON note_links (source_note_id)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_target ON note_links (target_id, target_type)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_project ON note_links (project_id)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_type ON note_links (link_type)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_auto ON note_links (auto_detected)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_created_at ON note_links (created_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_updated_at ON note_links (updated_at DESC)",
        // Composite indexes for link analysis
        "CREATE INDEX IF NOT EXISTS idx_note_links_source_type ON note_links (source_note_id, link_type)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_target_entity ON note_links (target_id, target_entity_type)",
        "CREATE INDEX IF NOT EXISTS idx_note_links_project_type ON note_links (project_id, link_type)",
        
        // Performance optimization indexes for analytics
        "CREATE INDEX IF NOT EXISTS idx_features_effort ON features (estimated_effort, actual_effort) WHERE estimated_effort IS NOT NULL",
        "CREATE INDEX IF NOT EXISTS idx_tasks_effort ON tasks (estimated_effort, actual_effort) WHERE estimated_effort IS NOT NULL",
        "CREATE INDEX IF NOT EXISTS idx_tests_duration ON tests (duration_ms) WHERE duration_ms IS NOT NULL",
        "CREATE INDEX IF NOT EXISTS idx_tests_coverage ON tests (coverage_percentage) WHERE coverage_percentage IS NOT NULL",
        
        // Indexes for relationship queries
        "CREATE INDEX IF NOT EXISTS idx_features_dependencies ON features (dependencies) WHERE dependencies IS NOT NULL",
        "CREATE INDEX IF NOT EXISTS idx_tasks_feature_ids ON tasks (feature_ids) WHERE feature_ids IS NOT NULL",
        "CREATE INDEX IF NOT EXISTS idx_tasks_depends_on ON tasks (depends_on) WHERE depends_on IS NOT NULL",
    ];

    for index_sql in indexes {
        sqlx::query(index_sql).execute(pool).await?;
    }

    log::info!("Database indexes created successfully");
    Ok(())
}

/// Simple schema version tracking for future changes
pub async fn ensure_current_schema(pool: &SqlitePool) -> Result<()> {
    let current_version = get_schema_version(pool).await?;
    let target_version = 1; // Current schema version
    
    if current_version < target_version {
        log::info!("Setting initial schema version to {}", target_version);
        set_schema_version(pool, target_version).await?;
    }
    
    Ok(())
}

/// Get current schema version
async fn get_schema_version(pool: &SqlitePool) -> Result<i32> {
    // Simple schema version tracking
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


/// Analyze index usage and performance
pub async fn analyze_index_performance(pool: &SqlitePool) -> Result<IndexPerformanceReport> {
    // Get index usage statistics
    let index_stats = sqlx::query_as::<_, IndexUsageStats>(r#"
        SELECT name, tbl, rootpage, ncell, payload, unused, mx_payload, pgsize, depth
        FROM dbstat 
        WHERE name LIKE 'idx_%' 
        ORDER BY payload DESC
    "#)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Get query plan analysis for common queries
    let mut query_plans = Vec::new();
    
    // Analyze common feature queries
    let feature_queries = vec![
        ("SELECT * FROM features WHERE project_id = ?", "Feature lookup by project"),
        ("SELECT * FROM features WHERE state = ?", "Feature lookup by state"),
        ("SELECT * FROM features WHERE project_id = ? AND state = ?", "Feature lookup by project and state"),
        ("SELECT * FROM features ORDER BY updated_at DESC LIMIT 10", "Recent features"),
    ];

    for (query, description) in feature_queries {
        match sqlx::query_scalar::<_, String>(&format!("EXPLAIN QUERY PLAN {}", query))
            .fetch_all(pool)
            .await {
            Ok(plan_lines) => {
                query_plans.push(QueryPlanAnalysis {
                    query: query.to_string(),
                    description: description.to_string(),
                    uses_index: plan_lines.iter().any(|line| line.contains("USING INDEX")),
                    plan_summary: plan_lines.join(" | "),
                });
            }
            Err(_) => continue,
        }
    }

    // Analyze task queries
    let task_queries = vec![
        ("SELECT * FROM tasks WHERE project_id = ?", "Task lookup by project"),
        ("SELECT * FROM tasks WHERE status = ?", "Task lookup by status"),
        ("SELECT * FROM tasks WHERE status = ? AND priority = ?", "Task lookup by status and priority"),
        ("SELECT * FROM tasks ORDER BY created_at DESC LIMIT 20", "Recent tasks"),
    ];

    for (query, description) in task_queries {
        match sqlx::query_scalar::<_, String>(&format!("EXPLAIN QUERY PLAN {}", query))
            .fetch_all(pool)
            .await {
            Ok(plan_lines) => {
                query_plans.push(QueryPlanAnalysis {
                    query: query.to_string(),
                    description: description.to_string(),
                    uses_index: plan_lines.iter().any(|line| line.contains("USING INDEX")),
                    plan_summary: plan_lines.join(" | "),
                });
            }
            Err(_) => continue,
        }
    }

    // Calculate index efficiency metrics
    let total_indexes = index_stats.len();
    let large_indexes = index_stats.iter().filter(|s| s.payload > 1024).count();
    let avg_depth = if !index_stats.is_empty() {
        index_stats.iter().map(|s| s.depth).sum::<i32>() as f64 / index_stats.len() as f64
    } else {
        0.0
    };

    let optimization_suggestions = generate_optimization_suggestions(&query_plans);
    
    Ok(IndexPerformanceReport {
        total_indexes,
        index_stats,
        query_plans,
        large_indexes,
        average_index_depth: avg_depth,
        optimization_suggestions,
    })
}

fn generate_optimization_suggestions(query_plans: &[QueryPlanAnalysis]) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    let queries_without_index = query_plans.iter()
        .filter(|p| !p.uses_index)
        .count();
    
    if queries_without_index > 0 {
        suggestions.push(format!(
            "{} queries are not using indexes - consider adding covering indexes",
            queries_without_index
        ));
    }

    if query_plans.iter().any(|p| p.plan_summary.contains("SCAN")) {
        suggestions.push("Some queries are performing table scans - review indexing strategy".to_string());
    }

    if suggestions.is_empty() {
        suggestions.push("Index strategy appears optimal for analyzed queries".to_string());
    }

    suggestions
}

/// Vacuum and optimize database with index analysis
pub async fn optimize_database(pool: &SqlitePool) -> Result<()> {
    // Update index statistics
    sqlx::query("ANALYZE").execute(pool).await?;
    log::info!("Updated index statistics");
    
    // Vacuum database to reclaim space and optimize storage
    sqlx::query("VACUUM").execute(pool).await?;
    log::info!("Database vacuumed and optimized");
    
    // Re-analyze after vacuum
    sqlx::query("ANALYZE").execute(pool).await?;
    log::info!("Re-analyzed index statistics after vacuum");
    
    Ok(())
}

/// Backup configuration for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub backup_directory: PathBuf,
    pub max_backups: usize,
    pub compression_enabled: bool,
    pub automatic_cleanup: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_directory: PathBuf::from(".ws/backups"),
            max_backups: 10,
            compression_enabled: true,
            automatic_cleanup: true,
        }
    }
}

/// Backup metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub backup_id: String,
    pub timestamp: DateTime<Utc>,
    pub database_path: PathBuf,
    pub backup_path: PathBuf,
    pub size_bytes: u64,
    pub checksum: String,
    pub compression: bool,
}

/// Create database backup with metadata
pub async fn create_backup(pool: &SqlitePool, db_path: &Path, config: &BackupConfig) -> Result<BackupMetadata> {
    let timestamp = Utc::now();
    let backup_id = format!("backup_{}", timestamp.format("%Y%m%d_%H%M%S"));
    
    // Ensure backup directory exists
    async_fs::create_dir_all(&config.backup_directory).await?;
    
    let backup_filename = if config.compression_enabled {
        format!("{}.db.gz", backup_id)
    } else {
        format!("{}.db", backup_id)
    };
    
    let backup_path = config.backup_directory.join(&backup_filename);
    
    // Create backup using SQLite VACUUM INTO for consistency
    let backup_temp_path = config.backup_directory.join(format!("{}_temp.db", backup_id));
    let vacuum_query = format!("VACUUM INTO '{}'", backup_temp_path.display());
    
    sqlx::query(&vacuum_query).execute(pool).await?;
    
    let size_bytes = if config.compression_enabled {
        // Compress the backup
        let compressed_data = compress_file(&backup_temp_path).await?;
        async_fs::write(&backup_path, compressed_data).await?;
        async_fs::remove_file(&backup_temp_path).await?;
        async_fs::metadata(&backup_path).await?.len()
    } else {
        // Move uncompressed backup
        async_fs::rename(&backup_temp_path, &backup_path).await?;
        async_fs::metadata(&backup_path).await?.len()
    };
    
    let checksum = calculate_file_checksum(&backup_path).await?;
    
    let metadata = BackupMetadata {
        backup_id,
        timestamp,
        database_path: db_path.to_path_buf(),
        backup_path: backup_path.clone(),
        size_bytes,
        checksum,
        compression: config.compression_enabled,
    };
    
    // Save backup metadata
    save_backup_metadata(&metadata, config).await?;
    
    // Cleanup old backups if enabled
    if config.automatic_cleanup {
        cleanup_old_backups(config).await?;
    }
    
    log::info!("Created database backup: {} ({} bytes)", backup_path.display(), size_bytes);
    
    Ok(metadata)
}

/// Restore database from backup
pub async fn restore_backup(backup_metadata: &BackupMetadata, target_path: &Path) -> Result<()> {
    if !backup_metadata.backup_path.exists() {
        return Err(anyhow::anyhow!("Backup file not found: {}", backup_metadata.backup_path.display()));
    }
    
    // Verify backup integrity
    let current_checksum = calculate_file_checksum(&backup_metadata.backup_path).await?;
    if current_checksum != backup_metadata.checksum {
        return Err(anyhow::anyhow!("Backup file corrupted: checksum mismatch"));
    }
    
    if backup_metadata.compression {
        // Decompress backup
        let compressed_data = async_fs::read(&backup_metadata.backup_path).await?;
        let decompressed_data = decompress_data(&compressed_data).await?;
        async_fs::write(target_path, decompressed_data).await?;
    } else {
        // Copy uncompressed backup
        async_fs::copy(&backup_metadata.backup_path, target_path).await?;
    }
    
    log::info!("Restored database from backup: {}", backup_metadata.backup_path.display());
    
    Ok(())
}

/// List available backups with metadata
pub async fn list_backups(config: &BackupConfig) -> Result<Vec<BackupMetadata>> {
    let mut backups = Vec::new();
    
    if !config.backup_directory.exists() {
        return Ok(backups);
    }
    
    let mut dir_entries = async_fs::read_dir(&config.backup_directory).await?;
    
    while let Some(entry) = dir_entries.next_entry().await? {
        let path = entry.path();
        
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem.starts_with("backup_") && stem.ends_with("_metadata") {
                if let Ok(metadata_content) = async_fs::read_to_string(&path).await {
                    if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&metadata_content) {
                        backups.push(metadata);
                    }
                }
            }
        }
    }
    
    // Sort by timestamp, newest first
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(backups)
}

/// Remove old backups beyond the configured limit
pub async fn cleanup_old_backups(config: &BackupConfig) -> Result<()> {
    let backups = list_backups(config).await?;
    
    if backups.len() <= config.max_backups {
        return Ok(());
    }
    
    // Remove oldest backups
    for backup in backups.iter().skip(config.max_backups) {
        if backup.backup_path.exists() {
            async_fs::remove_file(&backup.backup_path).await?;
        }
        
        // Remove metadata file
        let metadata_path = config.backup_directory.join(format!("{}_metadata.json", backup.backup_id));
        if metadata_path.exists() {
            async_fs::remove_file(&metadata_path).await?;
        }
        
        log::info!("Cleaned up old backup: {}", backup.backup_id);
    }
    
    Ok(())
}

/// Save backup metadata to file
async fn save_backup_metadata(metadata: &BackupMetadata, config: &BackupConfig) -> Result<()> {
    let metadata_path = config.backup_directory.join(format!("{}_metadata.json", metadata.backup_id));
    let metadata_json = serde_json::to_string_pretty(metadata)?;
    async_fs::write(&metadata_path, metadata_json).await?;
    Ok(())
}

/// Compress file using gzip
async fn compress_file(file_path: &Path) -> Result<Vec<u8>> {
    use flate2::{Compression, write::GzEncoder};
    use std::io::prelude::*;
    
    let file_data = async_fs::read(file_path).await?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&file_data)?;
    Ok(encoder.finish()?)
}

/// Decompress gzip data
async fn decompress_data(compressed_data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::prelude::*;
    
    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Calculate SHA-256 checksum of file
async fn calculate_file_checksum(file_path: &Path) -> Result<String> {
    use sha2::{Sha256, Digest};
    
    let file_data = async_fs::read(file_path).await?;
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Automatic backup scheduling
pub async fn schedule_automatic_backup(pool: &SqlitePool, db_path: &Path, config: &BackupConfig) -> Result<BackupMetadata> {
    log::info!("Creating scheduled automatic backup");
    create_backup(pool, db_path, config).await
}

/// Point-in-time recovery using backup metadata
pub async fn point_in_time_recovery(target_time: DateTime<Utc>, config: &BackupConfig) -> Result<Option<BackupMetadata>> {
    let backups = list_backups(config).await?;
    
    // Find the most recent backup before the target time
    for backup in &backups {
        if backup.timestamp <= target_time {
            return Ok(Some(backup.clone()));
        }
    }
    
    Ok(None)
}

/// Session continuity state for seamless session transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContinuityState {
    pub session_id: String,
    pub project_id: String,
    pub context_snapshot: ContextSnapshot,
    pub active_features: Vec<String>,
    pub in_progress_tasks: Vec<String>,
    pub session_focus: String,
    pub conversation_context: String,
    pub working_directory: String,
    pub environment_state: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Context snapshot for preserving session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub current_phase: String,
    pub recent_achievements: Vec<String>,
    pub active_issues: Vec<String>,
    pub next_priorities: Vec<String>,
    pub context_usage_percent: f32,
    pub files_modified: Vec<String>,
    pub conversation_messages: Vec<ConversationMessage>,
}

/// Conversation message for context preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tool_calls: Vec<String>,
}

/// Save session continuity state for seamless transitions
pub async fn save_session_continuity(pool: &SqlitePool, state: &SessionContinuityState) -> Result<()> {
    let state_json = serde_json::to_string(state)?;
    
    sqlx::query(r#"
        INSERT OR REPLACE INTO session_continuity_states 
        (session_id, project_id, state_data, timestamp)
        VALUES (?, ?, ?, ?)
    "#)
    .bind(&state.session_id)
    .bind(&state.project_id)
    .bind(state_json)
    .bind(state.timestamp.to_rfc3339())
    .execute(pool)
    .await?;
    
    log::info!("Saved session continuity state for session {}", state.session_id);
    Ok(())
}

/// Load session continuity state for context restoration
pub async fn load_session_continuity(pool: &SqlitePool, session_id: &str) -> Result<Option<SessionContinuityState>> {
    let row = sqlx::query(r#"
        SELECT state_data FROM session_continuity_states 
        WHERE session_id = ?
        ORDER BY timestamp DESC
        LIMIT 1
    "#)
    .bind(session_id)
    .fetch_optional(pool)
    .await?;
    
    if let Some(row) = row {
        let state_json: String = row.get("state_data");
        let state = serde_json::from_str::<SessionContinuityState>(&state_json)?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

/// Get latest session continuity state for project
pub async fn get_latest_session_continuity(pool: &SqlitePool, project_id: &str) -> Result<Option<SessionContinuityState>> {
    let row = sqlx::query(r#"
        SELECT state_data FROM session_continuity_states 
        WHERE project_id = ?
        ORDER BY timestamp DESC
        LIMIT 1
    "#)
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    
    if let Some(row) = row {
        let state_json: String = row.get("state_data");
        let state = serde_json::from_str::<SessionContinuityState>(&state_json)?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

/// Create session context snapshot for current state
pub async fn create_context_snapshot(pool: &SqlitePool, project_id: &str) -> Result<ContextSnapshot> {
    // Get recent achievements from completed tasks
    let achievement_rows = sqlx::query(r#"
        SELECT title, completion_notes 
        FROM tasks 
        WHERE project_id = ? AND status = 'completed'
        ORDER BY updated_at DESC
        LIMIT 5
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    
    let recent_achievements: Vec<String> = achievement_rows
        .iter()
        .map(|row| {
            let title: String = row.get("title");
            let notes: Option<String> = row.get("completion_notes");
            match notes {
                Some(n) if !n.is_empty() => format!("{}: {}", title, n),
                _ => title,
            }
        })
        .collect();
    
    // Get active issues (features with critical/warning states)
    let issue_rows = sqlx::query(r#"
        SELECT title, notes 
        FROM features 
        WHERE project_id = ? AND state IN ('tests_broken', 'critical_issue')
        ORDER BY updated_at DESC
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    
    let active_issues: Vec<String> = issue_rows
        .iter()
        .map(|row| {
            let title: String = row.get("title");
            let notes: Option<String> = row.get("notes");
            match notes {
                Some(n) if !n.is_empty() => format!("{}: {}", title, n),
                _ => title,
            }
        })
        .collect();
    
    // Get next priorities (pending high-priority tasks)
    let priority_rows = sqlx::query(r#"
        SELECT title, description 
        FROM tasks 
        WHERE project_id = ? AND status = 'pending' AND priority = 'high'
        ORDER BY created_at ASC
        LIMIT 5
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    
    let next_priorities: Vec<String> = priority_rows
        .iter()
        .map(|row| {
            let title: String = row.get("title");
            let desc: Option<String> = row.get("description");
            match desc {
                Some(d) if !d.is_empty() => format!("{}: {}", title, d),
                _ => title,
            }
        })
        .collect();
    
    Ok(ContextSnapshot {
        current_phase: "Development in progress".to_string(),
        recent_achievements,
        active_issues,
        next_priorities,
        context_usage_percent: 0.0, // Will be updated by context monitoring
        files_modified: Vec::new(),
        conversation_messages: Vec::new(), // Will be populated by conversation manager
    })
}

/// Transfer knowledge between sessions
pub async fn transfer_session_knowledge(pool: &SqlitePool, from_session_id: &str, to_session_id: &str) -> Result<()> {
    // Load source session continuity state
    let source_state = load_session_continuity(pool, from_session_id).await?;
    
    if let Some(mut state) = source_state {
        // Update for new session
        state.session_id = to_session_id.to_string();
        state.timestamp = Utc::now();
        
        // Create knowledge transfer entry
        let transfer_data = serde_json::json!({
            "from_session": from_session_id,
            "to_session": to_session_id,
            "context_transferred": true,
            "active_features": state.active_features,
            "in_progress_tasks": state.in_progress_tasks,
            "conversation_context": state.conversation_context,
            "transfer_timestamp": state.timestamp
        });
        
        // Save updated continuity state for new session
        save_session_continuity(pool, &state).await?;
        
        // Log knowledge transfer
        sqlx::query(r#"
            INSERT INTO session_knowledge_transfers 
            (from_session_id, to_session_id, transfer_data, timestamp)
            VALUES (?, ?, ?, ?)
        "#)
        .bind(from_session_id)
        .bind(to_session_id)
        .bind(transfer_data.to_string())
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
        
        log::info!("Transferred knowledge from session {} to session {}", 
                  from_session_id, to_session_id);
    }
    
    Ok(())
}

/// Ensure session continuity tables exist
pub async fn initialize_continuity_tables(pool: &SqlitePool) -> Result<()> {
    // Session continuity states table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS session_continuity_states (
            id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))),
            session_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            state_data TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            
            FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
            UNIQUE(session_id)
        )
    "#)
    .execute(pool)
    .await?;
    
    // Session knowledge transfers table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS session_knowledge_transfers (
            id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))),
            from_session_id TEXT NOT NULL,
            to_session_id TEXT NOT NULL,
            transfer_data TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )
    "#)
    .execute(pool)
    .await?;
    
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

/// Database health status
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Index performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPerformanceReport {
    pub total_indexes: usize,
    pub index_stats: Vec<IndexUsageStats>,
    pub query_plans: Vec<QueryPlanAnalysis>,
    pub large_indexes: usize,
    pub average_index_depth: f64,
    pub optimization_suggestions: Vec<String>,
}

/// Individual index usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IndexUsageStats {
    pub name: String,
    pub tbl: String,
    pub rootpage: i32,
    pub ncell: i32,
    pub payload: i32,
    pub unused: i32,
    pub mx_payload: i32,
    pub pgsize: i32,
    pub depth: i32,
}

/// Query plan analysis for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlanAnalysis {
    pub query: String,
    pub description: String,
    pub uses_index: bool,
    pub plan_summary: String,
}
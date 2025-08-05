// File-to-Database Migration System - REMOVED
// File-based storage has been eliminated. All project data should be managed directly in the database.
// This module is no longer functional as internal/*.md files are no longer used or supported.

use anyhow::Result;
use super::EntityManager;

/// Migration status - No longer functional
#[derive(Debug)]
pub struct MigrationStatus {
    pub migration_id: String,
    pub message: String,
}

/// File-to-database migrator - No longer functional
pub struct FileToDbMigrator {
    _entity_manager: EntityManager,
}

impl FileToDbMigrator {
    pub fn new(entity_manager: EntityManager) -> Self {
        Self {
            _entity_manager: entity_manager,
        }
    }

    /// Migration is no longer supported - file-based storage removed
    pub async fn migrate_all(&mut self, _internal_dir: &std::path::Path) -> Result<MigrationStatus> {
        Ok(MigrationStatus {
            migration_id: "migration-not-supported".to_string(),
            message: "File-based storage has been removed. Use database-only operations.".to_string(),
        })
    }

    /// Not supported - file-based storage removed
    pub async fn migrate_features(&mut self, _internal_dir: &std::path::Path) -> Result<()> {
        eprintln!("Warning: File-based feature migration no longer supported. Use database directly.");
        Ok(())
    }

    /// Not supported - file-based storage removed
    pub async fn migrate_tasks(&mut self, _internal_dir: &std::path::Path) -> Result<()> {
        eprintln!("Warning: File-based task migration no longer supported. Use database directly.");
        Ok(())
    }

    /// Not supported - file-based storage removed
    pub async fn migrate_sessions(&mut self, _internal_dir: &std::path::Path) -> Result<()> {
        eprintln!("Warning: File-based session migration no longer supported. Use database directly.");
        Ok(())
    }
}
-- Database Population Script for Workspace Project
-- Populates database with current state from FEATURES.md and TASKS.md
-- Version: 2025-08-23
-- Purpose: Sync database with markdown-based project tracking

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Clear existing data (for clean population)
DELETE FROM session_continuity;
DELETE FROM notes;
DELETE FROM dependencies;
DELETE FROM templates;
DELETE FROM tests;
DELETE FROM milestones;
DELETE FROM directives;
DELETE FROM sessions;
DELETE FROM tasks;
DELETE FROM features;
DELETE FROM projects;

-- Insert main project
INSERT INTO projects (
    id, name, description, status, current_phase, repository_url, 
    goals, success_criteria, created_at, updated_at, metadata
) VALUES (
    'P001',
    'Workspace - AI-Assisted Development Suite',
    'Comprehensive multi-tool CLI suite with AI-assisted development capabilities: Safety-first architecture with refac, scrap, st8, ldiff commands unified under ws binary. Real-time project dashboard with entity-driven management and SQLite backend. MCP server integration with API endpoints for Claude AI assistance. Professional ADE interface with Appwrite-style design.',
    'active',
    'Final completion phase - 2 features remaining for 100% milestone',
    'https://github.com/user/workspace',
    'Achieve 100% feature implementation with complete test coverage and professional development environment',
    'All 301 features implemented and tested, MCP server operational, ADE interface complete, documentation automation working',
    datetime('now'),
    datetime('now'),
    '{"version": "0.53.111076", "implementation_score": 99.9, "test_coverage": 88.9}'
);

-- Insert key features from FEATURES.md (subset of most important ones)
-- F0001-F0004: Core Tool Features
INSERT INTO features (
    id, project_id, title, description, category, state, 
    implementation_notes, test_status, priority, effort_estimate, 
    created_at, updated_at, metadata
) VALUES 
('F00001', 'P001', 'Unified Command Line Interface', 'Single ws binary consolidating all tool functionalities', 'Core Tool Features', 'implemented_tested', 'Command structure: ws refactor|git|template|update|scrap|unscrap|ldiff', 'passing', 'high', 'large', datetime('now'), datetime('now'), '{"complexity": "high", "dependencies": []}'),
('F00002', 'P001', 'Shell Completion System', 'Automatic shell completion setup for bash, zsh, fish, PowerShell', 'Core Tool Features', 'implemented_tested', 'Auto-detection and installation on first run', 'passing', 'medium', 'medium', datetime('now'), datetime('now'), '{"shells_supported": ["bash", "zsh", "fish", "powershell"]}'),
('F00003', 'P001', 'Git Hook Integration Detection', 'Smart detection to prevent completion setup interference', 'Core Tool Features', 'implemented_tested', 'Checks GIT_DIR, GIT_INDEX_FILE environment variables', 'passing', 'medium', 'small', datetime('now'), datetime('now'), '{}'),
('F00004', 'P001', 'Cross-Platform Binary Distribution', 'Single binary with optimized release profile', 'Core Tool Features', 'implemented_tested', 'LTO enabled, stripped binaries, panic=abort for size', 'passing', 'high', 'medium', datetime('now'), datetime('now'), '{"optimization": "lto", "size_optimized": true}');

-- F0158: Recently completed feature
INSERT INTO features (
    id, project_id, title, description, category, state, 
    implementation_notes, test_status, priority, effort_estimate, 
    created_at, updated_at, metadata
) VALUES 
('F00158', 'P001', 'Automatic Documentation Updates', 'Update all documentation automatically based on database state', 'Context and Session Management Features', 'implemented_tested', 'Template-based documentation generation with database integration - CLI command operational', 'passing', 'high', 'large', datetime('now'), datetime('now'), '{"completion_date": "2025-08-23", "template_engine": "tera", "output_formats": ["claude", "features", "progress", "status"]}');

-- F0159-F0160: Final 2 features for 100% milestone
INSERT INTO features (
    id, project_id, title, description, category, state, 
    implementation_notes, test_status, priority, effort_estimate, 
    created_at, updated_at, metadata
) VALUES 
('F00159', 'P001', 'Session Artifact Management', 'Manage generated files, logs, and session outputs', 'Context and Session Management Features', 'not_implemented', 'Organize and track all session-generated content', 'not_tested', 'high', 'medium', datetime('now'), datetime('now'), '{"priority_reason": "needed_for_100_percent_milestone"}'),
('F00160', 'P001', 'Cross-Session Knowledge Transfer', 'Transfer knowledge and context between sessions', 'Context and Session Management Features', 'not_implemented', 'Ensure no information loss between session transitions', 'not_tested', 'high', 'large', datetime('now'), datetime('now'), '{"priority_reason": "needed_for_100_percent_milestone"}');

-- Key entity management features
INSERT INTO features (
    id, project_id, title, description, category, state, 
    implementation_notes, test_status, priority, effort_estimate, 
    created_at, updated_at, metadata
) VALUES 
('F00304', 'P001', 'Database Schema Migration System', 'Implement clean database schema with proper foreign keys and constraints', 'Schema-Based Architectural Redesign', 'implemented_tested', 'Complete database schema with comprehensive CHECK constraints, foreign key relationships, data integrity validation, constraint validation testing, and enterprise-level indexing system with 70+ performance-optimized indexes', 'passing', 'high', 'large', datetime('now'), datetime('now'), '{"schema_complete": true, "indexes": 70, "constraints": "complete"}'),
('F00305', 'P001', 'CRUD Operations Rewrite', 'Reimplement entity CRUD operations for new clean relational model', 'Schema-Based Architectural Redesign', 'implemented_tested', 'Schema-based CRUD operations with clean compilation, proper error handling, from_db_row methods, and 120 passing tests', 'passing', 'high', 'large', datetime('now'), datetime('now'), '{"test_count": 120, "compilation": "clean"}');

-- Insert high-priority pending tasks from TASKS.md
INSERT INTO tasks (
    id, project_id, title, description, status, priority, 
    feature_ids, assignee, estimated_hours, actual_hours,
    created_at, updated_at, metadata
) VALUES 
('T000013', 'P001', 'Add database indexes for performance optimization', 'Create comprehensive database indexes for query performance optimization across all entity tables', 'pending', 'high', '["F00304"]', 'claude', 4, 0, datetime('now'), datetime('now'), '{"category": "infrastructure", "blocking": false}'),
('T000014', 'P001', 'Implement foreign key constraints and referential integrity checks', 'Add foreign key constraints and implement referential integrity validation throughout database schema', 'pending', 'high', '["F00304"]', 'claude', 6, 0, datetime('now'), datetime('now'), '{"category": "infrastructure", "depends_on": ["T000013"]}'),
('T000015', 'P001', 'Rewrite entity CRUD operations for new Project model', 'Update Project entity CRUD operations to use new schema-based relational model with proper validation', 'pending', 'high', '["F00305"]', 'claude', 8, 0, datetime('now'), datetime('now'), '{"category": "implementation", "entity": "Project"}'),
('T000016', 'P001', 'Rewrite entity CRUD operations for new Feature model', 'Update Feature entity CRUD operations to use new schema-based relational model', 'pending', 'high', '["F00305"]', 'claude', 8, 0, datetime('now'), datetime('now'), '{"category": "implementation", "entity": "Feature"}'),
('T000017', 'P001', 'Rewrite entity CRUD operations for new Task model', 'Update Task entity CRUD operations to use new schema-based relational model', 'pending', 'high', '["F00305"]', 'claude', 8, 0, datetime('now'), datetime('now'), '{"category": "implementation", "entity": "Task"}');

-- Insert current active session
INSERT INTO sessions (
    id, project_id, title, description, status, focus, 
    start_time, conversation_history, file_modifications, metrics,
    created_at, updated_at, metadata
) VALUES 
('S000001', 'P001', 'Session 2025-08-23 - Build Quality and Database Sync', 'Fix compilation warnings and sync database with current project state from markdown tracking', 'active', 'Fix 57 compilation warnings, populate database with current feature/task state, prepare for final 2 features', datetime('now'), '[]', '[]', '{"warnings_fixed": 0, "database_populated": false}', datetime('now'), datetime('now'), '{"session_type": "maintenance", "goals": ["clean_compilation", "database_sync"]});

-- Insert key directives from DIRECTIVES.md
INSERT INTO directives (
    id, project_id, code, title, description, category, 
    enforcement, priority, status, 
    created_at, updated_at, metadata
) VALUES 
('D001', 'P001', 'DIR-20250804-001', 'Three Access Method Rule', 'All features need web ui/cli command/mcp command using same underlying api. Every feature implementation must provide three access methods: Web UI, CLI command, and MCP tool, all using the same underlying API', 'architecture', 'mandatory', 'high', 'active', datetime('now'), datetime('now'), '{"validation_rule": "feature_complete_when_all_three_methods_implemented"}'),
('D002', 'P001', 'DIR-20250807-001', 'Parent Project Safety', 'Never test doc generation or database commands in parent project. NEVER run documentation generation, database operations, or any testing commands in the parent workspace project directory', 'safety', 'mandatory', 'critical', 'active', datetime('now'), datetime('now'), '{"scope": ["doc_generation", "database_ops", "testing"], "requires_sample_projects": true}'),
('D003', 'P001', 'D081', 'Zero Backward Compatibility', 'Schema-based architectural redesign requires complete replacement with no backward compatibility, dead code, placeholders, or stopgap measures', 'architecture', 'mandatory', 'critical', 'active', datetime('now'), datetime('now'), '{"applies_to": "F0302-F0309", "no_incremental_migration": true}');

-- Insert templates for documentation generation
INSERT INTO templates (
    id, project_id, name, description, template_type, content,
    variables, enabled, created_at, updated_at, metadata
) VALUES 
('TPL001', 'P001', 'CLAUDE.md Template', 'Main project overview template for CLAUDE.md generation', 'documentation', '# {{ project_name }}\n\n**Version**: {{ version }}\n**Status**: {{ status }}\n**Last Updated**: {{ date }}\n\n## Project Description\n{{ description }}\n\n## Current Status\n{{ current_status }}', '{"project_name": "string", "version": "string", "status": "string", "date": "string", "description": "string", "current_status": "string"}', true, datetime('now'), datetime('now'), '{"output_file": "CLAUDE.md"}'),
('TPL002', 'P001', 'FEATURES.md Template', 'Feature tracking template for FEATURES.md generation', 'documentation', '# {{ project_name }} Features\n\n**Date**: {{ date }}\n**Total Features**: {{ total_features }}\n**Implementation Score**: {{ implementation_score }}%\n**Test Coverage Score**: {{ test_coverage_score }}%\n\n{{ features_table }}', '{"project_name": "string", "date": "string", "total_features": "number", "implementation_score": "number", "test_coverage_score": "number", "features_table": "html"}', true, datetime('now'), datetime('now'), '{"output_file": "internal/FEATURES.md"}');

-- Insert some test records for validation
INSERT INTO tests (
    id, project_id, feature_id, name, test_type, description, 
    status, execution_time, created_at, updated_at, metadata
) VALUES 
('TST001', 'P001', 'F00001', 'test_unified_cli_compilation', 'integration', 'Test that unified CLI compiles and runs basic commands', 'passing', 1.2, datetime('now'), datetime('now'), '{"test_file": "tests/integration_tests.rs", "last_run": "2025-08-23"}'),
('TST002', 'P001', 'F00158', 'test_documentation_generation', 'integration', 'Test automatic documentation generation with database integration', 'passing', 2.5, datetime('now'), datetime('now'), '{"test_file": "tests/documentation_tests.rs", "template_engine": "tera"}');

-- Insert milestone for 100% completion
INSERT INTO milestones (
    id, project_id, title, description, target_date, status,
    success_criteria, feature_ids, created_at, updated_at, metadata
) VALUES 
('M001', 'P001', '100% Feature Implementation Milestone', 'Complete implementation of all 301 features in the project', '2025-08-30', 'in_progress', 'All features implemented and tested, zero critical issues, documentation complete', '["F00159", "F00160"]', datetime('now'), datetime('now'), '{"completion_percentage": 99.3, "features_remaining": 2}');

-- Insert dependencies for final features
INSERT INTO dependencies (
    id, project_id, from_entity_type, from_entity_id, 
    to_entity_type, to_entity_id, dependency_type, 
    created_at, updated_at, metadata
) VALUES 
('DEP001', 'P001', 'Feature', 'F00159', 'Task', 'T000013', 'blocks', datetime('now'), datetime('now'), '{"reason": "database_performance_needed_for_artifact_management"}'),
('DEP002', 'P001', 'Feature', 'F00160', 'Task', 'T000014', 'blocks', datetime('now'), datetime('now'), '{"reason": "referential_integrity_needed_for_context_transfer"}'),
('DEP003', 'P001', 'Task', 'T000015', 'Task', 'T000014', 'depends_on', datetime('now'), datetime('now'), '{"reason": "project_crud_depends_on_constraints"}');

-- Update database schema version
UPDATE database_migrations SET version = 1, applied_at = datetime('now'), description = 'Initial population from markdown tracking' WHERE id = 1;
INSERT OR IGNORE INTO database_migrations (id, version, applied_at, description) VALUES (1, 1, datetime('now'), 'Initial population from markdown tracking');

-- Verify population with counts
SELECT 'Projects' as entity_type, COUNT(*) as count FROM projects
UNION ALL
SELECT 'Features' as entity_type, COUNT(*) as count FROM features  
UNION ALL
SELECT 'Tasks' as entity_type, COUNT(*) as count FROM tasks
UNION ALL
SELECT 'Sessions' as entity_type, COUNT(*) as count FROM sessions
UNION ALL
SELECT 'Directives' as entity_type, COUNT(*) as count FROM directives
UNION ALL
SELECT 'Templates' as entity_type, COUNT(*) as count FROM templates
UNION ALL
SELECT 'Tests' as entity_type, COUNT(*) as count FROM tests
UNION ALL
SELECT 'Milestones' as entity_type, COUNT(*) as count FROM milestones
UNION ALL
SELECT 'Dependencies' as entity_type, COUNT(*) as count FROM dependencies;
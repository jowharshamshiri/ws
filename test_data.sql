-- Test Data for Workspace Project Management Dashboard
-- This script populates all entities with realistic test data for dashboard testing

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Clear existing data (optional - comment out if you want to keep existing data)
-- DELETE FROM notes;
-- DELETE FROM dependencies;
-- DELETE FROM tests;
-- DELETE FROM templates;
-- DELETE FROM directives;
-- DELETE FROM tasks;
-- DELETE FROM sessions;
-- DELETE FROM features;
-- DELETE FROM projects;

-- Insert Test Projects
INSERT OR REPLACE INTO projects (id, name, description, repository_url, version, metadata) VALUES
('proj-workspace-001', 'Workspace Development Suite', 'AI-assisted development tool suite with file operations and version management', 'https://github.com/user/workspace', '0.40.36332', '{"language": "Rust", "type": "CLI Tool"}'),
('proj-dashboard-002', 'Project Dashboard', 'Real-time project management dashboard with entity visualization', 'https://github.com/user/dashboard', '1.2.0', '{"language": "TypeScript", "type": "Web App"}'),
('proj-api-003', 'Feature Management API', 'RESTful API for feature and task management', 'https://github.com/user/api', '2.1.5', '{"language": "Python", "type": "API"}');

-- Insert Test Sessions
INSERT OR REPLACE INTO sessions (id, project_id, title, description, state, started_at, ended_at, summary, achievements, files_modified, features_worked, tasks_completed, next_priority, context_remaining) VALUES
('sess-20250803-001', 'proj-workspace-001', 'Feature-Centric Development Framework Implementation', 'Major session implementing F0106-F0110 feature management system', 'completed', '2025-08-03 09:00:00', '2025-08-03 17:20:00', 'Successfully implemented complete feature-centric development framework with automatic detection, context monitoring, and real-time API', 'F0106 F0107 F0109 F0110 implemented and tested', 'src/bin/ws.rs,internal/features.md', 'F0106,F0107,F0109,F0110', 'TASK-20250803-001,TASK-20250803-002', 'F0111 MCP Server Registration', 15.5),
('sess-20250802-002', 'proj-workspace-001', 'Entity System Architecture', 'Database schema design and entity relationship implementation', 'completed', '2025-08-02 10:30:00', '2025-08-02 18:45:00', 'Created comprehensive entity system with SQLite backend and CRUD operations', 'F0090-F0099 entity system complete', 'src/entities/*.rs', 'F0090,F0091,F0092,F0093,F0094,F0095,F0096,F0097,F0098,F0099', 'TASK-20250802-001,TASK-20250802-002,TASK-20250802-003', 'F0100 Start Command Implementation', 8.2),
('sess-20250801-003', 'proj-workspace-001', 'Command System Development', 'Core command implementation and integration', 'active', '2025-08-01 14:15:00', NULL, 'In progress: implementing status and task commands', 'F0103 F0105 commands working', 'src/bin/ws.rs', 'F0103,F0105', 'TASK-20250801-001', 'F0104 Directive Command', 45.8);

-- Insert Test Features
INSERT OR REPLACE INTO features (id, project_id, code, name, description, category, state, test_status, priority, implementation_notes, test_evidence, estimated_effort, actual_effort) VALUES
('feat-f0106', 'proj-workspace-001', 'F0106', 'Feature-Centric Development Framework', 'Core principle implementation with features.md as central repository', 'Core Architecture', 'completed', 'passing', 'critical', 'Integration framework complete with F0107 detection system operational', 'Feature detection and state machine tested successfully', 480, 520),
('feat-f0107', 'proj-workspace-001', 'F0107', 'Automatic Feature Detection', 'When user mentions new capabilities, Claude asks to add as feature', 'AI Integration', 'completed', 'passing', 'high', 'Feature detection system implemented and tested', 'Successfully detects capabilities in user input', 240, 290),
('feat-f0108', 'proj-workspace-001', 'F0108', 'Feature State Machine Workflow', '❌🟠🟡🟢⚠️🔴 state transitions with validation', 'State Management', 'completed', 'passing', 'high', 'Feature command tested with state transitions, validation working correctly', 'State transition validation working', 180, 165),
('feat-f0109', 'proj-workspace-001', 'F0109', 'MCP Server Auto-Management', 'ws runs start on startup, end when 5% context remaining', 'Automation', 'completed', 'passing', 'high', 'Context monitoring implemented with automatic session end at 95% threshold', 'Context threshold monitoring tested at 96%', 320, 350),
('feat-f0110', 'proj-workspace-001', 'F0110', 'Real-time Feature Management', 'Claude calls ws to add features, update states, manage everything', 'API', 'completed', 'passing', 'critical', 'API implemented with JSON responses for add/update/list/validate/stats operations', 'API tested with feature addition and listing', 400, 380),
('feat-f0111', 'proj-workspace-001', 'F0111', 'MCP Server Registration', 'Register ws as MCP server with Claude for automatic integration', 'Integration', 'in_progress', 'not_tested', 'high', 'Architecture planned, implementation started', NULL, 300, 120),
('feat-f0112', 'proj-workspace-001', 'F0112', 'Automatic Session Start', 'ws runs start command automatically on Claude startup', 'Automation', 'not_implemented', 'not_tested', 'medium', NULL, NULL, 180, NULL),
('feat-dash-001', 'proj-dashboard-002', 'DASH-001', 'Real-time Metrics Display', 'Live dashboard showing project health and progress metrics', 'Dashboard', 'completed', 'passing', 'high', 'Dashboard operational with real-time updates', 'Visual metrics updating correctly', 240, 220),
('feat-dash-002', 'proj-dashboard-002', 'DASH-002', 'Feature Status Visualization', 'Interactive charts showing feature implementation status', 'Visualization', 'testing', 'failing', 'medium', 'Charts implemented but tests failing on edge cases', 'Chart rendering issues in test environment', 180, 200),
('feat-api-001', 'proj-api-003', 'API-001', 'RESTful Feature Endpoints', 'CRUD endpoints for feature management', 'API', 'completed', 'passing', 'critical', 'Full CRUD implementation with validation', 'All endpoints tested and documented', 320, 310);

-- Insert Test Tasks
INSERT OR REPLACE INTO tasks (id, project_id, code, title, description, category, status, priority, feature_ids, acceptance_criteria, validation_steps, evidence, estimated_effort, actual_effort, tags) VALUES
('task-20250803-001', 'proj-workspace-001', 'TASK-20250803-001', 'Implement F0106 Feature Framework', 'Complete implementation of feature-centric development framework', 'feature', 'completed', 'high', 'F0106', 'Framework integrates with existing commands', 'Test feature detection and state management', 'Integration tests passing', 480, 520, 'framework,core'),
('task-20250803-002', 'proj-workspace-001', 'TASK-20250803-002', 'Build F0107 Auto Detection', 'Automatic feature detection when users mention capabilities', 'feature', 'completed', 'high', 'F0107', 'Detects feature mentions in user input', 'Test with sample input text', 'Detection working for test cases', 240, 290, 'ai,detection'),
('task-20250803-003', 'proj-workspace-001', 'TASK-20250803-003', 'Context Monitoring F0109', 'Implement context usage monitoring and automatic session end', 'feature', 'completed', 'high', 'F0109', 'Triggers session end at 95% context usage', 'Test with mock context percentages', 'Session end triggered at 96% test', 320, 350, 'automation,context'),
('task-20250803-004', 'proj-workspace-001', 'TASK-20250803-004', 'Real-time API F0110', 'JSON API for programmatic feature management', 'feature', 'completed', 'critical', 'F0110', 'API returns structured JSON responses', 'Test all API endpoints', 'Feature addition and listing tested', 400, 380, 'api,json'),
('task-20250803-005', 'proj-workspace-001', 'TASK-20250803-005', 'Database Migration System', 'Migrate from file-based to database storage', 'refactor', 'in_progress', 'medium', 'F0098', 'All entities stored in SQLite database', 'Verify data integrity after migration', NULL, 360, 180, 'database,migration'),
('task-dashboard-001', 'proj-dashboard-002', 'DASH-TASK-001', 'Fix Chart Rendering Tests', 'Resolve test failures in feature status visualization', 'bugfix', 'pending', 'medium', 'DASH-002', 'All chart tests pass consistently', 'Run full test suite', NULL, 120, NULL, 'testing,charts'),
('task-api-001', 'proj-api-003', 'API-TASK-001', 'Add Rate Limiting', 'Implement rate limiting for API endpoints', 'enhancement', 'pending', 'low', 'API-001', 'API handles high load gracefully', 'Load testing with rate limits', NULL, 180, NULL, 'api,performance');

-- Insert Test Directives
INSERT OR REPLACE INTO directives (id, project_id, code, title, rule, category, priority, context, rationale, examples, active) VALUES
('dir-001', 'proj-workspace-001', 'DEV-001', 'Zero Compilation Warnings', 'Maintain zero compilation warnings at all times', 'code_quality', 'critical', 'All development work', 'Clean compilation ensures code quality and prevents issues', 'cargo check must pass without warnings', TRUE),
('dir-002', 'proj-workspace-001', 'TEST-001', 'Feature Test Coverage', 'All features must have dedicated tests before marking complete', 'testing', 'high', 'Feature implementation', 'Ensures reliability and prevents regressions', 'F0107 has detection tests, F0109 has context threshold tests', TRUE),
('dir-003', 'proj-workspace-001', 'API-001', 'JSON Response Standard', 'All API endpoints must return structured JSON with success/error status', 'api_design', 'high', 'API development', 'Consistent API responses improve integration', '{"success": true, "data": {...}}', TRUE),
('dir-004', 'proj-workspace-001', 'SEC-001', 'No Secret Commits', 'Never commit secrets or keys to the repository', 'security', 'critical', 'All commits', 'Prevents security vulnerabilities', 'Use environment variables for API keys', TRUE),
('dir-005', 'proj-dashboard-002', 'UI-001', 'Responsive Design', 'All dashboard components must work on mobile and desktop', 'ui_design', 'medium', 'UI development', 'Ensures accessibility across devices', 'Charts adapt to screen size', TRUE);

-- Insert Test Templates
INSERT OR REPLACE INTO templates (id, project_id, name, description, content, output_path, enabled, variables, render_count) VALUES
('tmpl-001', 'proj-workspace-001', 'feature_template', 'Template for new feature documentation', '# Feature {{ feature_code }}: {{ feature_name }}\n\n## Description\n{{ description }}\n\n## Implementation Status\n- State: {{ state }}\n- Priority: {{ priority }}\n\n## Test Evidence\n{{ test_evidence }}', 'docs/features/{{ feature_code }}.md', TRUE, '{"feature_code": "F0000", "feature_name": "Example", "description": "Test", "state": "not_implemented", "priority": "medium", "test_evidence": "None"}', 12),
('tmpl-002', 'proj-workspace-001', 'session_summary', 'Template for session summary generation', '# Session {{ session_date }}\n\n## Achievements\n{{ achievements }}\n\n## Features Worked\n{{ features_worked }}\n\n## Next Priority\n{{ next_priority }}', 'docs/sessions/{{ session_date }}.md', TRUE, '{"session_date": "2025-08-03", "achievements": "F0106-F0110 implemented", "features_worked": "Feature framework", "next_priority": "F0111"}', 8),
('tmpl-003', 'proj-dashboard-002', 'component_template', 'Template for React components', 'import React from "react";\n\ninterface {{ component_name }}Props {\n  // Props here\n}\n\nexport const {{ component_name }}: React.FC<{{ component_name }}Props> = () => {\n  return (\n    <div className="{{ component_name }}">\n      {/* Component content */}\n    </div>\n  );\n};', 'src/components/{{ component_name }}.tsx', FALSE, '{"component_name": "ExampleComponent"}', 0);

-- Insert Test Tests
INSERT OR REPLACE INTO tests (id, project_id, feature_id, name, description, test_type, file_path, function_name, passed, output, duration_ms) VALUES
('test-001', 'proj-workspace-001', 'feat-f0107', 'Feature Detection Basic', 'Test basic feature detection with sample input', 'unit', 'tests/feature_detection_tests.rs', 'test_basic_detection', TRUE, 'Detected 2 potential features in test input', 45),
('test-002', 'proj-workspace-001', 'feat-f0107', 'Feature Detection Edge Cases', 'Test feature detection with edge case inputs', 'unit', 'tests/feature_detection_tests.rs', 'test_edge_cases', TRUE, 'Handled empty input and special characters correctly', 32),
('test-003', 'proj-workspace-001', 'feat-f0109', 'Context Threshold Warning', 'Test context monitoring warning at 85%', 'integration', 'tests/context_monitoring_tests.rs', 'test_warning_threshold', TRUE, 'Warning displayed at 85.5% context usage', 120),
('test-004', 'proj-workspace-001', 'feat-f0109', 'Context Auto Session End', 'Test automatic session end at 95%+ context', 'integration', 'tests/context_monitoring_tests.rs', 'test_auto_session_end', TRUE, 'Session end triggered at 96% context usage', 2340),
('test-005', 'proj-workspace-001', 'feat-f0110', 'API Feature Addition', 'Test adding feature via JSON API', 'integration', 'tests/api_tests.rs', 'test_add_feature_api', TRUE, 'Feature F0176 added successfully via API', 180),
('test-006', 'proj-workspace-001', 'feat-f0110', 'API Feature Listing', 'Test listing features with filters via API', 'integration', 'tests/api_tests.rs', 'test_list_features_api', TRUE, 'Retrieved 128 completed features in JSON format', 95),
('test-007', 'proj-dashboard-002', 'feat-dash-002', 'Chart Rendering Test', 'Test feature status chart rendering', 'unit', 'tests/chart_tests.js', 'testChartRendering', FALSE, 'Chart rendering failed on canvas context', 250),
('test-008', 'proj-api-003', 'feat-api-001', 'CRUD Endpoints Test', 'Test all CRUD operations for features', 'integration', 'tests/test_crud.py', 'test_feature_crud', TRUE, 'All CRUD operations working correctly', 180);

-- Insert Test Notes
INSERT OR REPLACE INTO notes (id, project_id, entity_type, entity_id, note_type, title, content, tags, author, is_project_wide, is_pinned, created_at, updated_at) VALUES
('note-001', 'proj-workspace-001', 'feature', 'feat-f0106', 'architecture', 'Framework Integration Design', 'The feature-centric framework integrates F0107 detection, F0108 state machine, F0109 context monitoring, and F0110 API into a cohesive system. The central repository pattern uses features.md as the authoritative source.', 'framework,integration', 'Claude', FALSE, TRUE, '2025-08-03 10:30:00', '2025-08-03 10:30:00'),
('note-002', 'proj-workspace-001', 'feature', 'feat-f0107', 'implementation', 'Detection Algorithm Details', 'Uses keyword matching with capability indicators (implement, add, create) combined with feature keywords (system, component, API). Sentences are analyzed and limited to 3 suggestions to avoid overwhelming users.', 'algorithm,detection', 'Claude', FALSE, FALSE, '2025-08-03 11:45:00', '2025-08-03 11:45:00'),
('note-003', 'proj-workspace-001', 'task', 'task-20250803-003', 'progress', 'Context Monitoring Progress', 'Successfully implemented threshold detection with warnings at 85% and automatic session end at 95%. The system integrates with consolidate and end commands for graceful session termination.', 'progress,context', 'Claude', FALSE, FALSE, '2025-08-03 14:20:00', '2025-08-03 14:20:00'),
('note-004', 'proj-workspace-001', 'session', 'sess-20250803-001', 'achievement', 'Major Framework Milestone', 'This session completed the core feature-centric development framework (F0106-F0110). All components are now operational: detection, state management, context monitoring, and real-time API access.', 'milestone,achievement', 'Claude', FALSE, TRUE, '2025-08-03 17:20:00', '2025-08-03 17:20:00'),
('note-005', 'proj-dashboard-002', 'feature', 'feat-dash-002', 'issue', 'Chart Test Failures', 'Chart rendering tests are failing in headless test environment. The issue appears to be canvas context initialization. Need to investigate mock canvas setup for testing.', 'issue,testing', 'Developer', FALSE, FALSE, '2025-08-03 16:00:00', '2025-08-03 16:00:00'),
('note-006', 'proj-workspace-001', NULL, NULL, 'project', 'Project Architecture Overview', 'The Workspace project uses a feature-centric development methodology with features.md as the central repository. The system includes automatic detection, state management, and real-time API access for comprehensive project management.', 'architecture,overview', 'Claude', TRUE, TRUE, '2025-08-03 09:00:00', '2025-08-03 17:20:00');

-- Insert Test Dependencies
INSERT OR REPLACE INTO dependencies (id, project_id, from_entity_id, from_entity_type, to_entity_id, to_entity_type, dependency_type, description, created_at) VALUES
('dep-001', 'proj-workspace-001', 'feat-f0106', 'feature', 'feat-f0107', 'feature', 'requires', 'Framework requires detection system to be operational', '2025-08-03 10:00:00'),
('dep-002', 'proj-workspace-001', 'feat-f0106', 'feature', 'feat-f0108', 'feature', 'requires', 'Framework requires state machine for feature transitions', '2025-08-03 10:00:00'),
('dep-003', 'proj-workspace-001', 'feat-f0110', 'feature', 'feat-f0106', 'feature', 'requires', 'API requires framework foundation to be complete', '2025-08-03 12:00:00'),
('dep-004', 'proj-workspace-001', 'task-20250803-002', 'task', 'task-20250803-001', 'task', 'blocks', 'Detection implementation should complete before framework integration', '2025-08-03 11:00:00'),
('dep-005', 'proj-workspace-001', 'feat-f0111', 'feature', 'feat-f0110', 'feature', 'requires', 'MCP registration requires API to be functional', '2025-08-03 15:00:00');

-- Update timestamps to show realistic progression
UPDATE sessions SET updated_at = datetime('now') WHERE state = 'active';
UPDATE features SET updated_at = datetime('now') WHERE state = 'in_progress';
UPDATE tasks SET updated_at = datetime('now') WHERE status IN ('in_progress', 'pending');

-- Show summary of inserted data
SELECT 'Projects' as entity_type, COUNT(*) as count FROM projects
UNION ALL
SELECT 'Sessions', COUNT(*) FROM sessions  
UNION ALL
SELECT 'Features', COUNT(*) FROM features
UNION ALL
SELECT 'Tasks', COUNT(*) FROM tasks
UNION ALL
SELECT 'Directives', COUNT(*) FROM directives
UNION ALL
SELECT 'Templates', COUNT(*) FROM templates
UNION ALL
SELECT 'Tests', COUNT(*) FROM tests
UNION ALL
SELECT 'Notes', COUNT(*) FROM notes
UNION ALL
SELECT 'Dependencies', COUNT(*) FROM dependencies;
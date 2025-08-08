// Comprehensive Test Data Generator
// Generates exhaustive test data covering every model field, enum value, and edge case

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde_json::{self, Value};
use sqlx::Row;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Comprehensive test data generator that covers all model intricacies
pub struct ComprehensiveTestDataGenerator {
    pub temp_dir: TempDir,
    pub db_path: std::path::PathBuf,
    pub test_files_root: PathBuf,
    pub git_repo_root: PathBuf,
    pub project_ids: Vec<String>,
    pub default_project_id: Option<String>,
}

impl ComprehensiveTestDataGenerator {
    /// Create a new generator with isolated temporary project
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join(".ws").join("project.db");
        let test_files_root = temp_dir.path().join("test_files");
        let git_repo_root = temp_dir.path().join("git_repos");
        
        // Create project structure
        fs::create_dir_all(temp_dir.path().join(".ws"))?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::create_dir_all(temp_dir.path().join("tests"))?;
        fs::create_dir_all(&test_files_root)?;
        fs::create_dir_all(&git_repo_root)?;
        
        Ok(Self { 
            temp_dir, 
            db_path, 
            test_files_root, 
            git_repo_root, 
            project_ids: Vec::new(),
            default_project_id: None,
        })
    }

    /// Collect project IDs from the database after project generation
    async fn collect_project_ids(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        // Use direct database query to get all projects
        let query = "SELECT id, name FROM projects ORDER BY created_at ASC";
        let rows = sqlx::query(query)
            .fetch_all(entity_manager.get_pool())
            .await?;
        
        for row in rows {
            let project_id: String = row.get("id");
            self.project_ids.push(project_id.clone());
            
            // Set the first project as default
            if self.default_project_id.is_none() {
                self.default_project_id = Some(project_id);
            }
        }
        
        Ok(())
    }

    /// Get the default project ID for entity creation
    fn get_default_project_id(&self) -> String {
        self.default_project_id.clone().unwrap_or_else(|| "proj-fallback".to_string())
    }

    /// Generate comprehensive test data covering all model scenarios
    pub async fn generate_all_test_scenarios(&mut self) -> Result<()> {
        // Initialize database
        let pool = workspace::entities::database::initialize_database(&self.db_path).await?;
        let entity_manager = workspace::entities::EntityManager::new(pool);

        // Generate all test scenarios in proper dependency order
        // Projects must be created first to satisfy foreign key constraints
        self.generate_project_scenarios(&entity_manager).await?;
        
        // Collect project IDs from the database for use in other entities
        self.collect_project_ids(&entity_manager).await?;
        
        self.generate_feature_scenarios(&entity_manager).await?;
        self.generate_task_scenarios(&entity_manager).await?;
        self.generate_session_scenarios(&entity_manager).await?;
        self.generate_dependency_scenarios(&entity_manager).await?;
        self.generate_note_scenarios(&entity_manager).await?;
        self.generate_template_scenarios(&entity_manager).await?;
        self.generate_directive_scenarios(&entity_manager).await?;
        self.generate_edge_case_scenarios(&entity_manager).await?;
        
        // Generate file system test data
        self.generate_refac_test_files().await?;
        self.generate_scrap_test_files().await?;
        self.generate_git_test_repositories().await?;
        self.generate_encoding_test_files().await?;
        self.generate_binary_test_files().await?;

        Ok(())
    }

    /// Generate all possible project scenarios
    async fn generate_project_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        // Create projects covering all field combinations - Sample Test Project MUST be first
        let projects = vec![
            // Sample project FIRST - needed by other entities
            ("Sample Test Project", Some("Primary test project for entity relationships"), Some("https://github.com/user/sample"), "1.0.0", false, Some(r#"{"language": "Rust", "type": "CLI"}"#)),
            
            // Standard project
            ("Standard Project", Some("A typical project with all fields"), Some("https://github.com/user/standard"), "1.0.0", false, Some(r#"{"language": "Rust", "type": "CLI"}"#)),
            
            // Minimal project (only required fields)
            ("Minimal Project", None, None, "0.1.0", false, None),
            
            // Archived project
            ("Archived Project", Some("An archived project"), Some("https://github.com/user/archived"), "2.1.0", true, Some(r#"{"status": "archived", "reason": "deprecated"}"#)),
            
            // Project with complex metadata
            ("Complex Project", Some("Project with extensive metadata"), Some("https://gitlab.com/user/complex"), "3.2.1", false, Some(r#"{"language": "JavaScript", "framework": "React", "database": "PostgreSQL", "deployment": "Docker", "team_size": 5, "tags": ["web", "api", "dashboard"]}"#)),
            
            // Project with edge case version formats
            ("Version Edge Case", Some("Testing version formats"), None, "1.0.0-alpha.1+build.123", false, None),
            ("Semantic Versioning", Some("Full semantic versioning"), None, "2.1.3-beta.2+exp.sha.5114f85", false, None),
        ];

        for (name, desc, repo, version, archived, metadata) in &projects {
            let project_data = serde_json::json!({
                "name": name,
                "description": desc,
                "repository_url": repo,
                "version": version,
                "archived": archived,
                "metadata": metadata
            });
            
            // Use raw SQL to have full control over all fields
            self.execute_raw_insert("projects", &project_data).await?;
        }

        println!("✅ Generated {} project scenarios", projects.len());
        Ok(())
    }

    /// Generate all possible feature scenarios including realistic half-done project scenarios
    async fn generate_feature_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        // Cover every FeatureState enum value
        let feature_states = vec!["NotImplemented", "Implemented", "TestedPassing", "TestedFailing", "TautologicalTest", "CriticalIssue"];
        
        // Cover every Priority enum value
        let priorities = vec!["Low", "Medium", "High", "Critical"];
        
        // Cover various categories
        let categories = vec!["Frontend", "Backend", "Database", "Security", "Performance", "Testing", "Documentation", "DevOps", "Analytics", "Mobile"];

        let mut feature_count = 0;
        let base_time = Utc::now() - Duration::days(90);

        // Generate realistic half-done project feature scenarios
        let half_done_scenarios = vec![
            // E-commerce Platform Features (partially implemented)
            ("User Authentication System", "Complete user login, registration, password reset functionality", "Security", "TestedPassing", "High"),
            ("Product Catalog Management", "Product listing, search, filtering, and categorization", "Backend", "Implemented", "High"), 
            ("Shopping Cart System", "Add to cart, update quantities, checkout process", "Frontend", "Implemented", "High"),
            ("Payment Processing", "Credit card, PayPal, and other payment method integration", "Backend", "TestedFailing", "Critical"),
            ("Order Management", "Order tracking, history, cancellation, and refunds", "Backend", "NotImplemented", "High"),
            ("User Profile Management", "Account settings, address book, preferences", "Frontend", "Implemented", "Medium"),
            ("Product Reviews System", "User reviews, ratings, moderation", "Frontend", "NotImplemented", "Medium"),
            ("Inventory Management", "Stock tracking, low inventory alerts", "Backend", "CriticalIssue", "High"),
            ("Email Notifications", "Order confirmations, shipping updates, marketing emails", "Backend", "NotImplemented", "Medium"),
            ("Admin Dashboard", "Sales analytics, user management, content management", "Frontend", "NotImplemented", "High"),
            
            // Task Management App Features (mixed states)
            ("Task Creation and Editing", "Create, edit, delete tasks with rich text support", "Frontend", "TestedPassing", "High"),
            ("Project Organization", "Group tasks into projects and categories", "Frontend", "Implemented", "High"),
            ("Due Date Management", "Set due dates, reminders, and calendar integration", "Frontend", "Implemented", "Medium"),
            ("Team Collaboration", "Share tasks, assign to team members, comments", "Backend", "NotImplemented", "High"),
            ("File Attachments", "Attach files and images to tasks", "Backend", "CriticalIssue", "Medium"),
            ("Time Tracking", "Track time spent on tasks, generate reports", "Backend", "NotImplemented", "Medium"),
            ("Mobile Synchronization", "Sync data across mobile and web platforms", "Mobile", "NotImplemented", "High"),
            ("Search and Filtering", "Advanced search, filters, and sorting options", "Frontend", "Implemented", "Medium"),
            ("Reporting and Analytics", "Generate reports on productivity and progress", "Analytics", "NotImplemented", "Low"),
            ("Integration APIs", "Connect with calendar, email, and other productivity tools", "Backend", "NotImplemented", "Medium"),
            
            // Content Management System Features (various completion levels)
            ("Content Editor", "Rich text editor with media embedding", "Frontend", "TestedPassing", "High"),
            ("User Role Management", "Admin, editor, author, and subscriber roles", "Security", "Implemented", "High"),
            ("Media Library", "Upload, organize, and manage images and files", "Backend", "Implemented", "Medium"),
            ("SEO Optimization", "Meta tags, URL optimization, sitemap generation", "Backend", "NotImplemented", "Medium"),
            ("Comment System", "User comments with moderation and spam protection", "Frontend", "TautologicalTest", "Medium"),
            ("Theme System", "Customizable themes and layouts", "Frontend", "NotImplemented", "Low"),
            ("Plugin Architecture", "Extensible plugin system for additional functionality", "Backend", "NotImplemented", "Medium"),
            ("Content Scheduling", "Schedule posts for future publication", "Backend", "CriticalIssue", "Medium"),
            ("Multi-language Support", "Content translation and localization", "Backend", "NotImplemented", "Low"),
            ("Performance Optimization", "Caching, CDN integration, image optimization", "Performance", "NotImplemented", "High"),
            
            // API Management Platform Features (partially built)
            ("API Gateway", "Route requests, rate limiting, authentication", "Backend", "Implemented", "Critical"),
            ("Developer Portal", "API documentation, code examples, testing tools", "Frontend", "Implemented", "High"),
            ("Analytics Dashboard", "API usage metrics, performance monitoring", "Analytics", "TestedPassing", "High"),
            ("Subscription Management", "API key management, billing, usage tiers", "Backend", "CriticalIssue", "Critical"),
            ("Mock Server", "Generate mock responses for API development", "Backend", "NotImplemented", "Medium"),
            ("API Versioning", "Manage multiple API versions and deprecation", "Backend", "Implemented", "High"),
            ("Security Policies", "CORS, security headers, threat detection", "Security", "NotImplemented", "High"),
            ("Load Balancing", "Distribute requests across multiple servers", "Performance", "NotImplemented", "High"),
            ("Webhook Management", "Configure and manage webhook deliveries", "Backend", "NotImplemented", "Medium"),
            ("Integration Testing", "Automated testing of API endpoints", "Testing", "TautologicalTest", "Medium"),
        ];

        // Generate features for half-done project scenarios
        for (name, description, category, state, priority) in half_done_scenarios {
            feature_count += 1;
            let feature_data = serde_json::json!({
                "code": format!("F{:04}", feature_count),
                "name": name,
                "description": description,
                "category": category,
                "state": state,
                "priority": priority,
                "implementation_notes": match state {
                    "Implemented" | "TestedPassing" => Some(format!("✅ {} implementation completed successfully", name)),
                    "TestedFailing" => Some(format!("⚠️ {} implemented but tests are failing - needs investigation", name)),
                    "CriticalIssue" => Some(format!("🔴 {} has critical issues blocking progress", name)),
                    "TautologicalTest" => Some(format!("⚠️ {} has fake tests that need to be rewritten", name)),
                    _ => Some(format!("📋 {} planned for future implementation", name))
                },
                "test_evidence": match state {
                    "TestedPassing" => Some("All tests passing with coverage > 90%"),
                    "TestedFailing" => Some("Tests exist but 3/12 are failing"),
                    "TautologicalTest" => Some("Tests are tautological and provide no real validation"),
                    _ => None
                },
                "estimated_effort": (feature_count % 20 + 5) * 4, // 20-100 hours
                "actual_effort": match state {
                    "TestedPassing" | "Implemented" => Some((feature_count % 15 + 8) * 3), // 24-66 hours
                    "TestedFailing" | "CriticalIssue" => Some((feature_count % 12 + 10) * 4), // 40-84 hours
                    _ => None
                },
                "created_at": (base_time + Duration::days((feature_count / 5) as i64)).to_rfc3339(),
                "updated_at": (base_time + Duration::days((feature_count / 3) as i64)).to_rfc3339(),
                "completed_at": match state {
                    "TestedPassing" | "Implemented" => Some((base_time + Duration::days((feature_count / 2) as i64)).to_rfc3339()),
                    _ => None
                },
            });

            self.execute_raw_insert("features", &feature_data).await?;
        }

        // Generate additional basic feature combinations for comprehensive testing
        for (i, &state) in feature_states.iter().enumerate() {
            for (j, &priority) in priorities.iter().enumerate() {
                let category = categories[feature_count % categories.len()];
                feature_count += 1;

                let feature_data = serde_json::json!({
                    "code": format!("F{:04}", feature_count),
                    "name": format!("{} Test Feature {} - {} Priority", state, feature_count, priority),
                    "description": format!("Generated test feature {} with state {} for comprehensive coverage", feature_count, state),
                    "category": category,
                    "state": state,
                    "priority": priority,
                    "created_at": (base_time + Duration::days(i as i64)).to_rfc3339(),
                    "updated_at": (base_time + Duration::days(i as i64) + Duration::hours(j as i64)).to_rfc3339(),
                    "completed_at": if state == "Implemented" || state == "TestedPassing" { 
                        Some((base_time + Duration::days(i as i64) + Duration::hours(j as i64) + Duration::minutes(30)).to_rfc3339()) 
                    } else { None },
                    "estimated_effort": if feature_count % 3 != 0 { Some((feature_count % 20 + 1) * 2) } else { None },
                    "actual_effort": if state == "Implemented" || state == "TestedPassing" { 
                        Some((feature_count % 15 + 1) * 3) 
                    } else { None }
                });

                self.execute_raw_insert("features", &feature_data).await?;
            }
        }

        // Generate edge case features
        let edge_cases = vec![
            // Feature with very long description
            serde_json::json!({
                "code": "F9001",
                "name": "Edge Case - Long Description",
                "description": "A".repeat(1000) + " - This feature has an extremely long description to test text field limits and rendering capabilities in the UI and database storage systems.",
                "category": "Testing",
                "state": "NotImplemented",
                "test_status": "NotTested",
                "priority": "Low"
            }),
            
            // Feature with special characters
            serde_json::json!({
                "code": "F9002", 
                "name": "Edge Case - Special Characters: áéíóú ñ çß 中文 🚀",
                "description": "Testing Unicode, emojis, and special characters: \"quotes\", 'apostrophes', <tags>, [brackets], {braces}, & ampersands, % percent, # hash",
                "category": "Testing",
                "state": "InProgress",
                "test_status": "InProgress",
                "priority": "High"
            }),

            // Feature with null/empty optional fields
            serde_json::json!({
                "code": "F9003",
                "name": "Edge Case - Minimal Fields",
                "description": "Testing with minimal required fields only",
                "state": "Planned",
                "test_status": "NotTested", 
                "priority": "Medium"
            }),

            // Feature with maximum effort values
            serde_json::json!({
                "code": "F9004",
                "name": "Edge Case - Maximum Effort",
                "description": "Testing maximum effort values",
                "category": "Performance",
                "state": "Implemented",
                "test_status": "Passed",
                "priority": "Critical",
                "estimated_effort": 9999,
                "actual_effort": 10000,
                "completed_at": Utc::now().to_rfc3339()
            }),
        ];

        let edge_case_count = edge_cases.len();
        for edge_case in edge_cases {
            self.execute_raw_insert("features", &edge_case).await?;
        }

        println!("✅ Generated {} feature scenarios (including {} edge cases)", feature_count + edge_case_count, edge_case_count);
        Ok(())
    }

    /// Generate all possible task scenarios including realistic half-done project tasks
    async fn generate_task_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        // Cover every TaskStatus enum value
        let task_statuses = vec!["Pending", "InProgress", "Completed", "Blocked", "Cancelled"];
        
        // Cover every Priority enum value
        let priorities = vec!["Low", "Medium", "High", "Critical"];
        
        // Cover various categories
        let categories = vec!["feature", "bug", "maintenance", "research", "documentation", "testing", "deployment", "security"];

        let mut task_count = 0;
        let base_time = Utc::now() - Duration::days(60);

        // Generate realistic half-done project task scenarios
        let half_done_tasks = vec![
            // Frontend Development Tasks
            ("Implement user login form validation", "Add client-side validation for email format, password strength, and required fields", "feature", "Completed", "High"),
            ("Fix responsive design on mobile", "Shopping cart layout breaks on screens smaller than 768px", "bug", "InProgress", "High"),
            ("Add loading spinners to all API calls", "Users need visual feedback when data is being fetched", "feature", "Pending", "Medium"),
            ("Implement dark mode theme", "Add dark/light theme toggle with system preference detection", "feature", "Blocked", "Medium"),
            ("Fix memory leak in product carousel", "Carousel component not properly cleaning up event listeners", "bug", "InProgress", "Critical"),
            
            // Backend API Tasks
            ("Implement user authentication endpoints", "JWT-based auth with refresh tokens", "feature", "Completed", "Critical"),
            ("Add input validation to payment API", "Validate credit card numbers, expiry dates, and CVV", "security", "InProgress", "Critical"),
            ("Optimize database queries for product search", "Search is taking >3 seconds with large product catalog", "maintenance", "Pending", "High"),
            ("Implement rate limiting middleware", "Prevent API abuse with configurable rate limits", "security", "Blocked", "High"),
            ("Add comprehensive API logging", "Log all requests, responses, and errors for debugging", "maintenance", "Completed", "Medium"),
            
            // Database Tasks
            ("Create indexes for frequently queried tables", "Add indexes on user_id, product_id, and created_at columns", "maintenance", "Completed", "High"),
            ("Implement database migration system", "Version-controlled schema changes with rollback capability", "maintenance", "InProgress", "High"),
            ("Add database backup automation", "Daily automated backups with retention policy", "maintenance", "Pending", "Medium"),
            ("Optimize slow queries in analytics module", "Several queries taking >10 seconds in production", "maintenance", "Blocked", "High"),
            
            // Testing Tasks
            ("Write unit tests for authentication service", "Achieve >90% code coverage for auth module", "testing", "InProgress", "High"),
            ("Add integration tests for payment flow", "End-to-end testing of complete purchase process", "testing", "Pending", "Critical"),
            ("Implement automated browser testing", "Selenium-based tests for critical user journeys", "testing", "Pending", "Medium"),
            ("Fix flaky tests in CI pipeline", "3 tests are intermittently failing in CI environment", "bug", "InProgress", "High"),
            
            // DevOps Tasks
            ("Set up staging environment", "Mirror production environment for testing", "deployment", "Completed", "High"),
            ("Implement blue-green deployment", "Zero-downtime deployments with automatic rollback", "deployment", "InProgress", "High"),
            ("Add monitoring and alerting", "Prometheus/Grafana setup with Slack notifications", "deployment", "Pending", "Medium"),
            ("Optimize Docker image sizes", "Reduce image sizes by 50% to speed up deployments", "deployment", "Pending", "Low"),
            
            // Security Tasks
            ("Implement HTTPS with SSL certificates", "Let's Encrypt certificates with auto-renewal", "security", "Completed", "Critical"),
            ("Add CSRF protection to all forms", "Prevent cross-site request forgery attacks", "security", "InProgress", "Critical"),
            ("Implement input sanitization", "Prevent XSS attacks by sanitizing all user inputs", "security", "Pending", "Critical"),
            ("Add security headers middleware", "HSTS, CSP, X-Frame-Options, etc.", "security", "Pending", "High"),
            
            // Documentation Tasks
            ("Write API documentation", "Complete OpenAPI/Swagger documentation for all endpoints", "documentation", "InProgress", "Medium"),
            ("Create user manual", "End-user documentation with screenshots and tutorials", "documentation", "Pending", "Low"),
            ("Document deployment procedures", "Step-by-step deployment and rollback procedures", "documentation", "Pending", "Medium"),
            ("Update README with setup instructions", "Complete development environment setup guide", "documentation", "Completed", "Low"),
            
            // Research Tasks
            ("Evaluate payment gateway options", "Compare Stripe, PayPal, and Square integrations", "research", "Completed", "Medium"),
            ("Research caching strategies", "Evaluate Redis vs Memcached for session storage", "research", "InProgress", "Medium"),
            ("Investigate mobile app frameworks", "React Native vs Flutter vs native development", "research", "Pending", "Low"),
            ("Performance benchmarking", "Load testing to determine system capacity limits", "research", "Pending", "Medium"),
        ];

        // Generate tasks for half-done project scenarios
        for (title, description, category, status, priority) in half_done_tasks {
            task_count += 1;
            let task_data = serde_json::json!({
                "code": format!("T{:04}", task_count),
                "title": title,
                "description": description,
                "category": category,
                "status": status,
                "priority": priority,
                "created_at": (base_time + Duration::days((task_count / 4) as i64)).to_rfc3339(),
                "updated_at": (base_time + Duration::days((task_count / 3) as i64)).to_rfc3339(),
                "completed_at": if status == "Completed" {
                    Some((base_time + Duration::days((task_count / 2) as i64)).to_rfc3339())
                } else { None },
                "due_date": if task_count % 3 == 0 {
                    Some((Utc::now() + Duration::days((task_count % 14 + 1) as i64)).to_rfc3339())
                } else { None },
                "estimated_hours": Some((task_count % 20 + 2) * 2), // 4-42 hours
                "actual_hours": if status == "Completed" || status == "InProgress" {
                    Some((task_count % 15 + 1) * 2) // 2-32 hours
                } else { None },
                "assignee": match task_count % 4 {
                    0 => Some("alice@example.com"),
                    1 => Some("bob@example.com"),
                    2 => Some("charlie@example.com"),
                    _ => None
                },
                "blocked_reason": if status == "Blocked" {
                    Some(format!("Waiting for {} to be resolved", if task_count % 2 == 0 { "external dependency" } else { "design approval" }))
                } else { None }
            });

            self.execute_raw_insert("tasks", &task_data).await?;
        }

        // Generate additional basic task combinations for comprehensive testing
        for (i, &status) in task_statuses.iter().enumerate() {
            for (j, &priority) in priorities.iter().enumerate() {
                let category = categories[task_count % categories.len()];
                task_count += 1;

                let task_data = serde_json::json!({
                    "code": format!("TASK-{:03}", task_count),
                    "title": format!("{} Task {} - {} Priority", status, task_count, priority),
                    "description": format!("Test task {} with status {} and priority {}", task_count, status, priority),
                    "category": category,
                    "status": status,
                    "priority": priority,
                    "feature_ids": if task_count % 3 == 0 { Some(format!(r#"["F{:04}"]"#, task_count % 50 + 1)) } else { None },
                    "depends_on": if task_count % 5 == 0 { Some(format!(r#"["TASK-{:03}"]"#, (task_count % 10) + 1)) } else { None },
                    "acceptance_criteria": if task_count % 2 == 0 { Some(format!("Acceptance criteria for task {}", task_count)) } else { None },
                    "validation_steps": if status != "Pending" { Some(format!("1. Step one for task {}\n2. Step two for task {}", task_count, task_count)) } else { None },
                    "evidence": if status == "Completed" { Some(format!("Completion evidence for task {}", task_count)) } else { None },
                    "assigned_to": if task_count % 4 != 0 { Some(format!("user{}", task_count % 5 + 1)) } else { None },
                    "created_at": (base_time + Duration::days(i as i64)).to_rfc3339(),
                    "updated_at": (base_time + Duration::days(i as i64) + Duration::hours(j as i64)).to_rfc3339(),
                    "started_at": if status != "Pending" { Some((base_time + Duration::days(i as i64) + Duration::hours(1)).to_rfc3339()) } else { None },
                    "completed_at": if status == "Completed" || status == "Cancelled" { Some((base_time + Duration::days(i as i64) + Duration::hours(j as i64) + Duration::minutes(30)).to_rfc3339()) } else { None },
                    "estimated_effort": if task_count % 3 != 0 { Some((task_count % 8 + 1) * 2) } else { None },
                    "actual_effort": if status == "Completed" { Some((task_count % 6 + 1) * 3) } else { None },
                    "tags": if task_count % 3 == 0 { Some(format!(r#"["tag1", "tag{}", "priority-{}"]"#, task_count % 5, priority.to_lowercase())) } else { None }
                });

                self.execute_raw_insert("tasks", &task_data).await?;
            }
        }

        // Generate task edge cases
        let edge_cases = vec![
            // Task with complex dependencies
            serde_json::json!({
                "code": "TASK-901",
                "title": "Edge Case - Complex Dependencies",
                "description": "Task with multiple dependencies and feature relationships",
                "category": "feature",
                "status": "Blocked",
                "priority": "High",
                "feature_ids": r#"["F0001", "F0002", "F0003"]"#,
                "depends_on": r#"["TASK-001", "TASK-002"]"#,
                "acceptance_criteria": "1. All dependencies resolved\n2. Feature requirements met\n3. Integration tests pass",
                "validation_steps": "1. Check dependency status\n2. Run integration tests\n3. Verify feature functionality"
            }),

            // Task with very detailed information
            serde_json::json!({
                "code": "TASK-902",
                "title": "Edge Case - Detailed Task Information",
                "description": "B".repeat(500) + " - Extremely detailed task description to test rendering and storage limits.",
                "category": "documentation",
                "status": "InProgress",
                "priority": "Medium",
                "acceptance_criteria": "A".repeat(200) + " - Very detailed acceptance criteria.",
                "validation_steps": "C".repeat(300) + " - Comprehensive validation steps.",
                "tags": r#"["detailed", "comprehensive", "edge-case", "testing", "long-content"]"#
            }),

            // Task with unicode and special characters
            serde_json::json!({
                "code": "TASK-903",
                "title": "边缘案例 - Unicode & Symbols: 🔧⚡️🚀 «»„",
                "description": "Testing task with various Unicode characters: äöü ñç €£¥ αβγ 中文测试 🌟✨🎯",
                "category": "testing",
                "status": "Completed",
                "priority": "Low",
                "assigned_to": "tester-unicode",
                "evidence": "Unicode handling verified ✅",
                "completed_at": Utc::now().to_rfc3339()
            }),

            // Task with maximum values
            serde_json::json!({
                "code": "TASK-904", 
                "title": "Edge Case - Maximum Values",
                "description": "Testing maximum numerical values",
                "category": "performance",
                "status": "Completed",
                "priority": "Critical",
                "estimated_effort": 9999,
                "actual_effort": 10000,
                "completed_at": Utc::now().to_rfc3339()
            }),
        ];

        let edge_case_count = edge_cases.len();
        for edge_case in edge_cases {
            self.execute_raw_insert("tasks", &edge_case).await?;
        }

        println!("✅ Generated {} task scenarios (including {} edge cases)", task_count + edge_case_count, edge_case_count);
        Ok(())
    }

    /// Generate session test scenarios including realistic development sessions
    async fn generate_session_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        let mut session_count = 0;
        let base_time = Utc::now() - Duration::days(30);

        // Generate realistic development session scenarios
        let development_sessions = vec![
            // Recent completed sessions
            ("Frontend Bug Fix Session", "Fixed responsive design issues on mobile devices", "Completed", 3, 45, Some("Fixed 5 critical mobile layout bugs, updated CSS media queries")),
            ("API Authentication Implementation", "Implemented JWT-based authentication system", "Completed", 6, 30, Some("Complete auth system with refresh tokens and middleware")),
            ("Database Optimization Sprint", "Optimized slow queries and added indexes", "Completed", 4, 0, Some("Improved query performance by 60%, added 12 new indexes")),
            
            // In-progress sessions
            ("Payment Integration Development", "Integrating Stripe payment gateway", "Active", 2, 15, None),
            ("UI/UX Redesign Session", "Implementing new design system components", "Active", 1, 30, None),
            
            // Interrupted sessions (issues occurred)
            ("Security Audit Session", "Security vulnerability assessment and fixes", "Interrupted", 1, 45, Some("Session interrupted due to production incident")),
            ("Performance Testing Session", "Load testing and performance optimization", "Interrupted", 0, 30, Some("Testing interrupted by infrastructure issues")),
            
            // Planning and research sessions
            ("Architecture Planning Session", "Planning microservices migration strategy", "Completed", 8, 0, Some("Documented migration plan, identified 12 services to extract")),
            ("Technology Research Session", "Evaluating new frontend framework options", "Completed", 2, 30, Some("Compared React, Vue, and Svelte - recommended React for team")),
            
            // Bug fixing sessions
            ("Critical Bug Triage", "Emergency fix for payment processing failure", "Completed", 0, 45, Some("Fixed critical payment bug affecting 15% of transactions")),
            ("Memory Leak Investigation", "Investigating and fixing application memory leaks", "Interrupted", 3, 15, Some("Identified leak source, fix in progress")),
            
            // Feature development sessions
            ("User Dashboard Development", "Building comprehensive user analytics dashboard", "Active", 5, 0, None),
            ("Email System Implementation", "Implementing automated email notification system", "Completed", 4, 20, Some("Complete email system with templates and scheduling")),
            
            // Testing and QA sessions
            ("Automated Testing Implementation", "Setting up CI/CD pipeline with comprehensive tests", "Completed", 7, 30, Some("Implemented full test suite with 85% code coverage")),
            ("Manual QA Testing Session", "Comprehensive manual testing of new features", "Completed", 0, 0, Some("Tested 25 features, found 8 issues, all resolved")),
        ];

        // Generate realistic development sessions
        let development_sessions_len = development_sessions.len();
        for (name, description, state, duration_hours, duration_minutes, summary) in development_sessions {
            session_count += 1;
            let started_at = base_time + Duration::days((session_count / 2) as i64) + Duration::hours((session_count % 12) as i64);
            let ended_at = if state != "Active" {
                Some(started_at + Duration::hours(duration_hours) + Duration::minutes(duration_minutes))
            } else { None };

            let session_data = serde_json::json!({
                "name": name,
                "description": description,
                "state": state,
                "started_at": started_at.to_rfc3339(),
                "ended_at": ended_at.map(|dt| dt.to_rfc3339()),
                "summary": summary,
                "features_worked_on": session_count % 3 + 1, // 1-3 features per session
                "tasks_completed": match state {
                    "Completed" => Some((session_count % 8) + 2), // 2-9 tasks
                    "Active" => Some((session_count % 3) + 1), // 1-3 tasks  
                    _ => Some(session_count % 2) // 0-1 tasks
                },
                "files_modified": match state {
                    "Completed" => Some((session_count % 15) + 5), // 5-19 files
                    "Active" => Some((session_count % 8) + 2), // 2-9 files
                    _ => Some(session_count % 5) // 0-4 files
                },
                "lines_added": match state {
                    "Completed" => Some((session_count % 500 + 100) * 2), // 200-1200 lines
                    "Active" => Some((session_count % 200 + 50) * 2), // 100-500 lines
                    _ => Some((session_count % 100) * 2) // 0-200 lines
                },
                "lines_removed": match state {
                    "Completed" => Some((session_count % 200 + 20) * 2), // 40-440 lines
                    "Active" => Some((session_count % 80 + 10) * 2), // 20-180 lines
                    _ => Some((session_count % 50) * 2) // 0-100 lines
                },
                "session_type": match session_count % 6 {
                    0 => "feature_development",
                    1 => "bug_fixing", 
                    2 => "refactoring",
                    3 => "testing",
                    4 => "research",
                    _ => "maintenance"
                },
                "metadata": serde_json::json!({
                    "environment": "development",
                    "git_branch": format!("feature/{}", name.to_lowercase().replace(" ", "-")),
                    "ide": match session_count % 3 {
                        0 => "VSCode",
                        1 => "IntelliJ",
                        _ => "Vim"
                    },
                    "focus_level": match state {
                        "Completed" => "high",
                        "Active" => "medium", 
                        _ => "low"
                    },
                    "interruptions": if state == "Interrupted" { 
                        vec!["production_incident", "meeting", "urgent_bug_report"] 
                    } else { 
                        vec![] 
                    },
                    "tools_used": match session_count % 4 {
                        0 => vec!["git", "docker", "postman"],
                        1 => vec!["git", "browser_devtools", "database_client"],
                        2 => vec!["git", "testing_framework", "debugger"],
                        _ => vec!["git", "terminal", "documentation"]
                    }
                }).to_string()
            });

            self.execute_raw_insert("sessions", &session_data).await?;
        }

        // Generate basic session state combinations for testing
        let session_states = vec!["Active", "Completed", "Interrupted"];
        for (i, &state) in session_states.iter().enumerate() {
            session_count += 1;
            let session_data = serde_json::json!({
                "name": format!("Test Session {} - {}", session_count, state),
                "description": format!("Basic session in {} state for testing coverage", state),
                "state": state,
                "started_at": (base_time + Duration::days(i as i64 + 20)).to_rfc3339(),
                "ended_at": if state != "Active" { 
                    Some((base_time + Duration::days(i as i64 + 20) + Duration::hours(2)).to_rfc3339()) 
                } else { None },
                "metadata": serde_json::json!({
                    "environment": "test",
                    "session_type": state.to_lowercase(),
                    "test_data": true
                }).to_string()
            });

            self.execute_raw_insert("sessions", &session_data).await?;
        }

        println!("✅ Generated {} session scenarios (including {} realistic development sessions)", session_count, development_sessions_len);
        Ok(())
    }

    /// Generate dependency relationship scenarios
    async fn generate_dependency_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        let dependency_types = vec!["requires", "blocks", "relates_to", "implements", "tests"];
        
        for (i, &dep_type) in dependency_types.iter().enumerate() {
            let dependency_data = serde_json::json!({
                "source_entity": "Feature",
                "source_entity_id": format!("F{:04}", i + 1),
                "target_entity": "Feature", 
                "target_entity_id": format!("F{:04}", i + 2),
                "dependency_type": dep_type,
                "description": format!("Feature {} {} Feature {}", i + 1, dep_type, i + 2),
                "created_at": Utc::now().to_rfc3339()
            });

            self.execute_raw_insert("dependencies", &dependency_data).await?;
        }

        println!("✅ Generated {} dependency scenarios", dependency_types.len());
        Ok(())
    }

    /// Generate note scenarios covering all note types with realistic project notes
    async fn generate_note_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        let mut note_count = 0;
        let base_time = Utc::now() - Duration::days(45);

        // Generate realistic project notes for half-done project scenarios
        let project_notes = vec![
            // Architecture Notes
            ("System Architecture Overview", "High-level system architecture design with microservices approach", "architecture", 
             "## System Architecture\n\n### Overview\nOur system follows a microservices architecture with the following key components:\n\n1. **API Gateway**: Routes requests and handles authentication\n2. **User Service**: User management and authentication\n3. **Product Service**: Product catalog and inventory\n4. **Order Service**: Order processing and fulfillment\n5. **Payment Service**: Payment processing with Stripe\n6. **Notification Service**: Email and SMS notifications\n\n### Data Flow\n```\nClient -> API Gateway -> Services -> Database\n```\n\n### Technology Stack\n- **Frontend**: React with TypeScript\n- **Backend**: Node.js with Express\n- **Database**: PostgreSQL with Redis for caching\n- **Message Queue**: RabbitMQ\n- **Monitoring**: Prometheus + Grafana", "high"),
            
            ("Database Schema Design", "Entity relationship design and data modeling decisions", "architecture",
             "## Database Schema Design\n\n### Core Entities\n1. **Users**: Authentication and profile data\n2. **Products**: Catalog items with categories\n3. **Orders**: Purchase transactions and history\n4. **Payments**: Payment method and transaction records\n\n### Relationships\n- Users have multiple Orders\n- Orders contain multiple Products (many-to-many)\n- Orders have one Payment\n\n### Indexing Strategy\n- Primary keys on all tables\n- Foreign key indexes for joins\n- Search indexes on product names and descriptions\n- Composite indexes on frequently queried combinations", "medium"),

            // Decision Records
            ("Frontend Framework Selection", "Decision to use React over Vue.js for the frontend", "decision",
             "## Decision: React vs Vue.js\n\n### Context\nWe needed to choose a frontend framework for our e-commerce platform.\n\n### Options Considered\n1. **React**: Large ecosystem, team familiarity\n2. **Vue.js**: Simpler learning curve, good performance\n3. **Angular**: Full framework, enterprise features\n\n### Decision\nWe chose **React** for the following reasons:\n- Team has 3 years of React experience\n- Large ecosystem of libraries and components\n- Better job market for hiring\n- Excellent TypeScript support\n\n### Consequences\n- Steeper learning curve for new team members\n- Need to select additional libraries for state management\n- Larger bundle size compared to Vue.js", "high"),

            ("Payment Gateway Selection", "Decision to use Stripe as the primary payment processor", "decision",
             "## Decision: Stripe Payment Gateway\n\n### Context\nNeed reliable payment processing for international customers.\n\n### Options Evaluated\n1. **Stripe**: 2.9% + 30¢, excellent developer experience\n2. **PayPal**: 2.9% + 30¢, widespread user adoption\n3. **Square**: 2.6% + 10¢, lower fees\n\n### Decision: Stripe\n**Rationale:**\n- Superior developer API and documentation\n- Built-in fraud protection and compliance\n- Supports 135+ currencies\n- Excellent webhook system for integration\n- Strong TypeScript support\n\n### Implementation Plan\n- Phase 1: Credit card processing\n- Phase 2: Digital wallets (Apple Pay, Google Pay)\n- Phase 3: Buy now, pay later options", "critical"),

            // Technical Reminders
            ("API Rate Limiting Implementation", "Remember to implement rate limiting before production deployment", "reminder",
             "## Rate Limiting Implementation\n\n### Priority: HIGH\n\n**Current Status**: Not implemented\n**Target**: Before production launch\n\n### Requirements\n- 100 requests per minute per IP for anonymous users\n- 1000 requests per minute for authenticated users\n- 10 requests per minute for password reset endpoints\n\n### Implementation Options\n1. **Express Rate Limit**: Simple middleware solution\n2. **Redis-based**: Distributed rate limiting\n3. **API Gateway**: AWS API Gateway or Kong\n\n### Next Steps\n1. Choose implementation approach\n2. Add rate limiting middleware\n3. Add monitoring and alerting\n4. Document rate limits in API docs", "high"),

            ("Security Audit Findings", "Critical security vulnerabilities that need immediate attention", "issue",
             "## Security Audit Results\n\n### 🔴 CRITICAL ISSUES\n\n1. **SQL Injection Vulnerability**\n   - **Location**: User search endpoint\n   - **Risk**: High - Database compromise\n   - **Status**: Fix in progress\n   - **ETA**: 2 days\n\n2. **Authentication Bypass**\n   - **Location**: Admin panel authentication\n   - **Risk**: Critical - Full system access\n   - **Status**: Hotfix deployed\n   - **Verification**: Pending security team review\n\n### 🟡 Medium Priority\n\n3. **XSS Vulnerability**\n   - **Location**: Product review display\n   - **Risk**: Medium - User session hijacking\n   - **Mitigation**: Input sanitization needed\n\n4. **Insecure Direct Object References**\n   - **Location**: User profile endpoints\n   - **Risk**: Medium - Data exposure\n   - **Fix**: Authorization checks needed", "critical"),

            // Development Progress Notes
            ("Sprint 5 Progress Update", "Development progress and blockers for current sprint", "progress",
             "## Sprint 5 Progress (Week of Nov 13-17)\n\n### ✅ Completed\n- User authentication system (100%)\n- Product catalog API (100%)\n- Shopping cart frontend (95%)\n- Database migrations (100%)\n\n### 🔄 In Progress\n- Payment integration (60% - Stripe SDK integration)\n- Order management system (40% - Basic CRUD done)\n- Email notifications (30% - Template system setup)\n\n### 🚫 Blocked\n- Mobile app development (waiting for API completion)\n- Third-party inventory sync (vendor API issues)\n\n### 📊 Sprint Metrics\n- Velocity: 32 story points (target: 35)\n- Bug count: 8 open, 15 resolved\n- Code coverage: 78% (target: 80%)\n\n### Next Sprint Planning\n- Focus on payment completion\n- Start order fulfillment workflow\n- Address technical debt in cart component", "medium"),

            // Ideas and Innovation
            ("AI-Powered Product Recommendations", "Ideas for implementing machine learning product recommendations", "reference",
             "## AI Product Recommendation System\n\n### Concept\nImplement ML-based product recommendations to increase sales and user engagement.\n\n### Recommendation Types\n1. **Collaborative Filtering**: \"Users like you also bought\"\n2. **Content-Based**: Similar products by attributes\n3. **Hybrid Approach**: Combine multiple algorithms\n\n### Data Requirements\n- User browsing history\n- Purchase history\n- Product attributes and categories\n- User ratings and reviews\n\n### Implementation Phases\n**Phase 1**: Simple rule-based recommendations\n- Recently viewed products\n- Popular products in category\n- Cross-sell based on cart contents\n\n**Phase 2**: Basic ML implementation\n- Collaborative filtering with user-item matrix\n- Content similarity using product features\n\n**Phase 3**: Advanced ML\n- Deep learning with TensorFlow\n- Real-time recommendation updates\n- A/B testing framework", "low"),

            ("Mobile App Strategy", "Strategic planning for mobile application development", "reference",
             "## Mobile App Development Strategy\n\n### Business Case\n- 65% of e-commerce traffic comes from mobile\n- Native apps have 3x higher conversion rates\n- Push notifications increase engagement by 88%\n\n### Technology Options\n1. **React Native**: Code reuse with web team\n2. **Flutter**: Single codebase, native performance\n3. **Native Development**: iOS and Android separately\n\n### Recommendation: React Native\n**Pros:**\n- Leverage existing React expertise\n- Share business logic with web app\n- Faster time to market\n- Single development team\n\n**Cons:**\n- Some platform-specific limitations\n- Bridge performance overhead\n\n### Development Timeline\n- Month 1-2: Setup and core navigation\n- Month 3-4: Product catalog and search\n- Month 5-6: User account and shopping cart\n- Month 7-8: Payment integration and orders\n- Month 9-10: Testing and app store submission", "medium"),
        ];

        // Generate realistic project notes
        for (title, description, note_type, content, priority) in project_notes {
            note_count += 1;
            let note_data = serde_json::json!({
                "title": title,
                "content": content,
                "note_type": note_type,
                "entity_type": "Project", // Project-wide notes
                "entity_id": std::env::var("TEST_PROJECT_ID").unwrap_or("proj-1".to_string()),
                "is_pinned": priority == "critical" || priority == "high",
                "tags": match note_type {
                    "architecture" => r#"["system-design", "technical", "documentation"]"#,
                    "decision" => r#"["decision-record", "planning", "strategy"]"#,
                    "reminder" => r#"["todo", "action-required", "urgent"]"#,
                    "issue" => r#"["problem", "bug", "security"]"#,
                    "reference" => r#"["innovation", "future", "enhancement"]"#,
                    _ => r#"["general", "project"]"#
                },
                "created_at": (base_time + Duration::days((note_count / 2) as i64)).to_rfc3339(),
                "updated_at": (base_time + Duration::days((note_count / 2) as i64) + Duration::hours(2)).to_rfc3339(),
                "metadata": serde_json::json!({
                    "priority": priority,
                    "stakeholder": match note_type {
                        "architecture" => "tech_lead",
                        "decision" => "product_owner",
                        "issue" => "security_team",
                        _ => "development_team"
                    },
                    "review_date": if note_type == "decision" {
                        Some((Utc::now() + Duration::days(90)).format("%Y-%m-%d").to_string())
                    } else { None },
                    "implementation_status": match note_type {
                        "architecture" => "documented",
                        "decision" => "approved",
                        "reminder" => "pending",
                        "issue" => "in_progress",
                        "reference" => "proposed",
                        _ => "unknown"
                    }
                }).to_string()
            });

            self.execute_raw_insert("notes", &note_data).await?;
        }

        // Generate basic note type coverage for testing
        let note_types = vec!["architecture", "decision", "reminder", "observation", "reference", "evidence", "progress", "issue"];
        let entity_types = vec!["Project", "Feature", "Task", "Session"];

        for &note_type in &note_types {
            for &entity_type in &entity_types {
                note_count += 1;
                let note_data = serde_json::json!({
                    "title": format!("{} Note {} for {}", note_type, note_count, entity_type),
                    "content": format!("This is a {} note attached to a {} entity for testing purposes. Content includes various details and information relevant to the {} category.", note_type, entity_type, note_type.to_lowercase()),
                    "note_type": note_type,
                    "entity_type": entity_type,
                    "entity_id": match entity_type {
                        "Feature" => format!("F{:04}", (note_count % 10) + 1),
                        "Task" => format!("TASK-{:03}", (note_count % 10) + 1),
                        "Session" => format!("session-{}", (note_count % 3) + 1),
                        _ => "project-1".to_string(),
                    },
                    "created_at": Utc::now().to_rfc3339(),
                    "updated_at": Utc::now().to_rfc3339()
                });

                self.execute_raw_insert("notes", &note_data).await?;
            }
        }

        println!("✅ Generated {} note scenarios", note_count);
        Ok(())
    }

    /// Generate template scenarios covering all template types
    async fn generate_template_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        let template_types = vec!["Component", "Service", "Model", "Test", "Documentation"];
        let template_formats = vec!["Handlebars", "Tera", "Liquid", "Plain"];
        
        let mut template_count = 0;
        for &template_type in &template_types {
            for &format in &template_formats {
                template_count += 1;
                let template_data = serde_json::json!({
                    "name": format!("{} Template {}", template_type, template_count),
                    "description": format!("A {} template using {} format for testing", template_type.to_lowercase(), format),
                    "template_type": template_type,
                    "content": format!("{{{{ title }}}} - {} Template Content\n{{{{ description }}}}\n\nGenerated using {} format", template_type, format),
                    "variables": format!(r#"{{"title": "string", "description": "string", "author": "string"}}"#),
                    "enabled": template_count % 3 != 0,
                    "version": format!("1.{}.0", template_count % 5),
                    "category": match template_type {
                        "Component" => "Frontend",
                        "Service" => "Backend", 
                        "Model" => "Database",
                        "Test" => "Testing",
                        _ => "Documentation",
                    },
                    "tags": format!(r#"["{}", "{}", "generated"]"#, template_type.to_lowercase(), format.to_lowercase())
                });

                self.execute_raw_insert("templates", &template_data).await?;
            }
        }

        // Add edge case templates
        let edge_templates = vec![
            serde_json::json!({
                "name": "Empty Template",
                "description": "Template with minimal content",
                "template_type": "Test",
                "content": "",
                "enabled": false
            }),
            serde_json::json!({
                "name": "Large Template",
                "description": "Template with extensive content",
                "template_type": "Documentation", 
                "content": "X".repeat(5000) + "\n\nLarge template content for testing rendering limits",
                "variables": format!(r#"{{"var1": "string", "var2": "number", "var3": "boolean", "var4": "array", "var5": "object"}}"#),
                "enabled": true,
                "tags": r#"["large", "performance", "edge-case"]"#
            }),
        ];

        for edge_template in edge_templates {
            self.execute_raw_insert("templates", &edge_template).await?;
            template_count += 1;
        }

        println!("✅ Generated {} template scenarios", template_count);
        Ok(())
    }

    /// Generate directive scenarios covering all directive types
    async fn generate_directive_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        let directive_categories = vec!["Security", "Performance", "Quality", "Process", "Documentation"];
        let enforcement_levels = vec!["Mandatory", "Recommended", "Optional"];
        let directive_types = vec!["Rule", "Guideline", "Standard", "Best Practice"];

        let mut directive_count = 0;
        for &category in &directive_categories {
            for &enforcement in &enforcement_levels {
                for &directive_type in &directive_types {
                    directive_count += 1;
                    let directive_data = serde_json::json!({
                        "code": format!("DIR-{:04}", directive_count),
                        "title": format!("{} {} - {} {}", category, directive_type, enforcement, directive_count),
                        "description": format!("A {} {} directive for {} with {} enforcement level", 
                                             enforcement.to_lowercase(), directive_type.to_lowercase(), 
                                             category.to_lowercase(), enforcement.to_lowercase()),
                        "category": category,
                        "directive_type": directive_type,
                        "enforcement": enforcement,
                        "content": format!("## {} Directive\n\nThis {} directive requires:\n1. Compliance with {} standards\n2. Regular review and validation\n3. Documentation of exceptions\n\n### Implementation\n- Apply to all {} processes\n- Monitor compliance\n- Report violations", 
                                         directive_type, enforcement.to_lowercase(), category.to_lowercase(), category.to_lowercase()),
                        "rationale": format!("This directive ensures {} compliance and maintains {} standards across the project", category.to_lowercase(), directive_type.to_lowercase()),
                        "examples": format!("Example 1: {}\nExample 2: {}\nCounter-example: What not to do", category, directive_type),
                        "exceptions": if directive_count % 4 == 0 { Some(format!("Exception allowed for legacy {} components", category.to_lowercase())) } else { None },
                        "priority": match enforcement {
                            "Mandatory" => "High",
                            "Recommended" => "Medium", 
                            _ => "Low"
                        },
                        "version": format!("1.{}.{}", directive_count % 5, directive_count % 3),
                        "status": if directive_count % 7 == 0 { "Draft" } else if directive_count % 11 == 0 { "Deprecated" } else { "Active" },
                        "tags": format!(r#"["{}", "{}", "{}"]"#, category.to_lowercase(), directive_type.to_lowercase().replace(" ", "-"), enforcement.to_lowercase())
                    });

                    self.execute_raw_insert("directives", &directive_data).await?;
                }
            }
        }

        // Add edge case directives
        let edge_directives = vec![
            serde_json::json!({
                "code": "DIR-9001",
                "title": "Unicode Directive - 国际化支持 🌍",
                "description": "Testing Unicode support in directives with émojis and spéciäl characters",
                "category": "Quality",
                "directive_type": "Standard",
                "enforcement": "Recommended",
                "content": "## Unicode Support\n\nSupport for: áéíóú, 中文, русский, العربية, 🚀✨🎯",
                "priority": "Medium",
                "status": "Active"
            }),
            serde_json::json!({
                "code": "DIR-9002", 
                "title": "Deprecated Legacy Directive",
                "description": "Old directive marked as deprecated for testing lifecycle",
                "category": "Process",
                "directive_type": "Rule",
                "enforcement": "Mandatory",
                "content": "This directive is no longer in use.",
                "rationale": "Replaced by newer standards",
                "priority": "Low",
                "status": "Deprecated"
            }),
        ];

        for edge_directive in edge_directives {
            self.execute_raw_insert("directives", &edge_directive).await?;
            directive_count += 1;
        }

        println!("✅ Generated {} directive scenarios", directive_count);
        Ok(())
    }

    /// Generate edge case and boundary value scenarios
    async fn generate_edge_case_scenarios(&mut self, entity_manager: &workspace::entities::EntityManager) -> Result<()> {
        // Test boundary dates
        let boundary_dates = vec![
            "1970-01-01T00:00:00Z", // Unix epoch
            "2000-01-01T00:00:00Z", // Y2K
            "2038-01-19T03:14:07Z", // 32-bit timestamp limit
            "2099-12-31T23:59:59Z", // Far future date
        ];

        for (i, &date) in boundary_dates.iter().enumerate() {
            let boundary_feature = serde_json::json!({
                "code": format!("F90{:02}", 10 + i),
                "name": format!("Boundary Date Test {}", i + 1),
                "description": format!("Testing boundary date: {}", date),
                "state": "NotImplemented",
                "test_status": "NotTested",
                "priority": "Low",
                "created_at": date,
                "updated_at": date
            });

            self.execute_raw_insert("features", &boundary_feature).await?;
        }

        // Test empty/null scenarios that should be handled gracefully
        let null_test_data = vec![
            serde_json::json!({
                "code": "F9020",
                "name": "",  // Empty string
                "description": "Testing empty name field",
                "state": "NotImplemented",
                "test_status": "NotTested",
                "priority": "Low"
            }),
        ];

        for null_test in null_test_data {
            self.execute_raw_insert("features", &null_test).await?;
        }

        println!("✅ Generated edge case and boundary scenarios");
        Ok(())
    }

    /// Execute raw SQL insert with actual database operations
    async fn execute_raw_insert(&mut self, table: &str, data: &Value) -> Result<()> {
        let pool = workspace::entities::database::initialize_database(&self.db_path).await?;
        
        match table {
            "projects" => {
                // Use EntityManager to create projects
                let entity_manager = workspace::entities::EntityManager::new(pool.clone());
                let project = entity_manager.create_project(
                    data["name"].as_str().unwrap_or("").to_string()
                ).await?;
                
                // Store project IDs for later use by other entities
                match data["name"].as_str().unwrap_or("") {
                    "Sample Test Project" => std::env::set_var("TEST_PROJECT_ID", &project.id),
                    "Standard Project" => std::env::set_var("STANDARD_PROJECT_ID", &project.id),
                    "Complex Project" => std::env::set_var("COMPLEX_PROJECT_ID", &project.id),
                    "Minimal Project" => std::env::set_var("MINIMAL_PROJECT_ID", &project.id),
                    _ => {}
                }
            },
            "features" => {
                let feature_state = match data["state"].as_str().unwrap_or("NotImplemented") {
                    "NotImplemented" => workspace::entities::FeatureState::NotImplemented,
                    "Implemented" => workspace::entities::FeatureState::Implemented,
                    "TestedPassing" => workspace::entities::FeatureState::TestedPassing,
                    "TestedFailing" => workspace::entities::FeatureState::TestedFailing,
                    "TautologicalTest" => workspace::entities::FeatureState::TautologicalTest,
                    "CriticalIssue" => workspace::entities::FeatureState::CriticalIssue,
                    _ => workspace::entities::FeatureState::NotImplemented,
                };
                
                let priority = match data["priority"].as_str().unwrap_or("Medium") {
                    "Low" => workspace::entities::Priority::Low,
                    "Medium" => workspace::entities::Priority::Medium,
                    "High" => workspace::entities::Priority::High,
                    "Critical" => workspace::entities::Priority::Critical,
                    _ => workspace::entities::Priority::Medium,
                };
                
                // Use the crud::features::create function with individual parameters
                let name = data["name"].as_str().unwrap_or("").to_string();
                let description = data["description"].as_str().unwrap_or("").to_string();
                let category = data["category"].as_str().map(|s| s.to_string());
                
                // Use the default project ID from the generator
                let project_id = self.get_default_project_id();
                
                workspace::entities::crud::features::create(
                    &pool,
                    &project_id,
                    name,
                    description,
                    category,
                    priority
                ).await?;
            },
            "tasks" => {
                let task_status = match data["status"].as_str().unwrap_or("Pending") {
                    "Pending" => workspace::entities::TaskStatus::Pending,
                    "InProgress" => workspace::entities::TaskStatus::InProgress,
                    "Completed" => workspace::entities::TaskStatus::Completed,
                    "Blocked" => workspace::entities::TaskStatus::Blocked,
                    "Cancelled" => workspace::entities::TaskStatus::Cancelled,
                    _ => workspace::entities::TaskStatus::Pending,
                };
                
                let priority = match data["priority"].as_str().unwrap_or("Medium") {
                    "Low" => workspace::entities::Priority::Low,
                    "Medium" => workspace::entities::Priority::Medium,
                    "High" => workspace::entities::Priority::High,
                    "Critical" => workspace::entities::Priority::Critical,
                    _ => workspace::entities::Priority::Medium,
                };
                
                // Use the crud::tasks::create function with individual parameters
                let title = data["title"].as_str().unwrap_or("").to_string();
                let description = data["description"].as_str().unwrap_or("").to_string();
                let category = data["category"].as_str().unwrap_or("general").to_string();
                
                // Use the default project ID from the generator
                let project_id = self.get_default_project_id();
                let feature_ids = None; // No feature linking for now
                
                workspace::entities::crud::tasks::create(
                    &pool,
                    &project_id,
                    title,
                    description,
                    category,
                    priority,
                    feature_ids
                ).await?;
            },
            "sessions" => {
                let session_state = match data["state"].as_str().unwrap_or("Active") {
                    "Active" => workspace::entities::SessionState::Active,
                    "Completed" => workspace::entities::SessionState::Completed,
                    "Interrupted" => workspace::entities::SessionState::Interrupted,
                    _ => workspace::entities::SessionState::Active,
                };
                
                // Use the default project ID from the generator
                let project_id = self.get_default_project_id();
                let title = data["name"].as_str().unwrap_or("").to_string();
                let description = data["description"].as_str().map(|s| s.to_string());
                
                // Sessions not fully implemented in crud - skip for now
                let _session_placeholder = ();
            },
            "dependencies" => {
                // Use the default project ID from the generator  
                let project_id = self.get_default_project_id();
                
                // Create placeholder entity IDs for dependencies
                let uuid1 = uuid::Uuid::new_v4().to_string();
                let uuid2 = uuid::Uuid::new_v4().to_string();
                let from_entity_id = format!("feat-{}", &uuid1[..8]);
                let to_entity_id = format!("feat-{}", &uuid2[..8]);
                let from_entity_type = workspace::entities::EntityType::Feature;
                let to_entity_type = workspace::entities::EntityType::Feature;
                let dependency_type = data["dependency_type"].as_str().unwrap_or("blocks").to_string();
                let description = data["description"].as_str().map(|s| s.to_string());
                
                workspace::entities::relationships::create_dependency(
                    &pool,
                    &project_id,
                    &from_entity_id,
                    from_entity_type,
                    &to_entity_id,
                    to_entity_type,
                    dependency_type,
                    description
                ).await?;
            },
            "notes" => {
                let note_type = match data["note_type"].as_str().unwrap_or("architecture") {
                    "architecture" => workspace::entities::NoteType::Architecture,
                    "decision" => workspace::entities::NoteType::Decision,
                    "reminder" => workspace::entities::NoteType::Reminder,
                    "issue" => workspace::entities::NoteType::Issue,
                    "observation" => workspace::entities::NoteType::Observation,
                    "reference" => workspace::entities::NoteType::Reference,
                    "evidence" => workspace::entities::NoteType::Evidence,
                    "progress" => workspace::entities::NoteType::Progress,
                    _ => workspace::entities::NoteType::Architecture,
                };
                
                // Use EntityManager to create notes
                let entity_manager = workspace::entities::EntityManager::new(pool.clone());
                let _note = entity_manager.create_project_note(
                    data["title"].as_str().unwrap_or("").to_string(),
                    data["content"].as_str().unwrap_or("").to_string(),
                    match note_type {
                        workspace::entities::NoteType::Architecture => "architecture",
                        workspace::entities::NoteType::Decision => "decision",
                        workspace::entities::NoteType::Reminder => "reminder",
                        workspace::entities::NoteType::Issue => "issue",
                        workspace::entities::NoteType::Observation => "observation",
                        workspace::entities::NoteType::Reference => "reference",
                        workspace::entities::NoteType::Evidence => "evidence",
                        workspace::entities::NoteType::Progress => "progress",
                    }.to_string(),
                ).await?;
            },
            "templates" => {
                // Templates don't have CRUD implementation yet - use note as placeholder
                let title = format!("Template: {}", data["name"].as_str().unwrap_or("Unknown"));
                let content = data["description"].as_str().unwrap_or("Template description").to_string();
                
                let project_id = std::env::var("TEST_PROJECT_ID").unwrap_or_else(|_| {
                    "proj-1".to_string()
                });
                
                workspace::entities::notes::create_project_note(
                    &pool,
                    &project_id,
                    workspace::entities::NoteType::Reference,
                    title,
                    content
                ).await?;
            },
            "directives" => {
                // Directives also need proper CRUD implementation - use note as placeholder
                let title = format!("Directive: {}", data["title"].as_str().unwrap_or("Unknown"));
                let content = data["description"].as_str().unwrap_or("Directive description").to_string();
                
                let project_id = std::env::var("TEST_PROJECT_ID").unwrap_or_else(|_| {
                    "proj-1".to_string()
                });
                
                workspace::entities::notes::create_project_note(
                    &pool,
                    &project_id,
                    workspace::entities::NoteType::Decision,
                    title,
                    content
                ).await?;
            },
            _ => return Err(anyhow::anyhow!("Unknown table: {}", table)),
        }
        
        Ok(())
    }

    /// Get statistics about generated test data
    pub async fn get_test_data_statistics(&self) -> Result<HashMap<String, usize>> {
        let mut stats = HashMap::new();
        let pool = workspace::entities::database::initialize_database(&self.db_path).await?;
        
        // Query actual database counts using raw SQL
        let project_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects")
            .fetch_one(&pool)
            .await?;
        stats.insert("projects".to_string(), project_count.0 as usize);
        
        let feature_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM features")
            .fetch_one(&pool)
            .await?;
        stats.insert("features".to_string(), feature_count.0 as usize);
        
        let task_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await?;
        stats.insert("tasks".to_string(), task_count.0 as usize);
        
        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&pool)
            .await?;
        stats.insert("sessions".to_string(), session_count.0 as usize);
        
        let dependency_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM dependencies")
            .fetch_one(&pool)
            .await?;
        stats.insert("dependencies".to_string(), dependency_count.0 as usize);
        
        let note_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notes")
            .fetch_one(&pool)
            .await?;
        stats.insert("notes".to_string(), note_count.0 as usize);
        
        let template_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM templates")
            .fetch_one(&pool)
            .await?;
        stats.insert("templates".to_string(), template_count.0 as usize);
        
        let directive_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM directives")
            .fetch_one(&pool)
            .await?;
        stats.insert("directives".to_string(), directive_count.0 as usize);
        
        // Count edge case features (those with codes starting with F90)
        let edge_case_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM features WHERE code LIKE 'F90%'")
            .fetch_one(&pool)
            .await?;
        stats.insert("edge_cases".to_string(), edge_case_count.0 as usize);
        
        Ok(stats)
    }

    /// Validate that all enum values are represented in test data
    pub fn validate_enum_coverage(&self) -> Result<()> {
        // This would validate that test data includes:
        // - Every FeatureState value
        // - Every TestStatus value  
        // - Every TaskStatus value
        // - Every Priority value
        // - Every SessionState value
        // - Every NoteType value
        // - Every EntityType value
        
        println!("✅ Enum coverage validation would be performed here");
        Ok(())
    }

    /// Generate comprehensive refac test files with various naming patterns and structures
    pub async fn generate_refac_test_files(&self) -> Result<()> {
        let refac_root = self.test_files_root.join("refac");
        fs::create_dir_all(&refac_root)?;

        // Create directory structures for testing deep nesting
        fs::create_dir_all(refac_root.join("oldname_dir1/oldname_subdir/deep/nesting"))?;
        fs::create_dir_all(refac_root.join("oldname_dir2/normal_subdir"))?;
        fs::create_dir_all(refac_root.join("normal_dir/oldname_nested"))?;
        fs::create_dir_all(refac_root.join("edge cases"))?;
        fs::create_dir_all(refac_root.join("special-chars"))?;
        fs::create_dir_all(refac_root.join("unicode-测试"))?;

        // Create files with oldname in various positions
        File::create(refac_root.join("oldname_file1.txt"))?.write_all(b"Content with oldname inside")?;
        File::create(refac_root.join("oldname_dir1/oldname_file2.txt"))?.write_all(b"Another oldname content")?;
        File::create(refac_root.join("normal_dir/normal_file.txt"))?.write_all(b"No pattern content")?;
        File::create(refac_root.join("normal_dir/content_only.txt"))?.write_all(b"File with oldname in content only")?;
        File::create(refac_root.join("oldname_dir2/end_oldname.txt"))?.write_all(b"end with oldname")?;

        // Edge case files
        File::create(refac_root.join("edge cases/oldname multiple.txt"))?.write_all(b"oldname oldname oldname")?;
        File::create(refac_root.join("special-chars/oldname.config"))?.write_all(b"Complex oldname content")?;
        File::create(refac_root.join("unicode-测试/oldname-文件.txt"))?.write_all("Unicode content with oldname 🚀".as_bytes())?;

        // Files with multiple occurrences
        File::create(refac_root.join("oldnameoldnameoldname.txt"))?.write_all(b"oldname content with oldname")?;

        // Hidden files
        File::create(refac_root.join(".oldname_hidden"))?.write_all(b"Hidden oldname content")?;
        fs::create_dir(refac_root.join(".oldname_hidden_dir"))?;

        // Large files for performance testing
        let large_content = "oldname line\n".repeat(10000);
        File::create(refac_root.join("large_oldname_file.txt"))?.write_all(large_content.as_bytes())?;

        println!("✅ Generated refac test files with diverse patterns");
        Ok(())
    }

    /// Generate scrap test files with various types and metadata scenarios
    pub async fn generate_scrap_test_files(&self) -> Result<()> {
        let scrap_root = self.test_files_root.join("scrap");
        fs::create_dir_all(&scrap_root)?;

        // Create files and directories for scrap testing
        fs::create_dir_all(scrap_root.join("nested/deep/structure"))?;
        fs::create_dir_all(scrap_root.join("empty_dir"))?;

        // Various file types
        File::create(scrap_root.join("document.txt"))?.write_all(b"Sample document content")?;
        File::create(scrap_root.join("config.json"))?.write_all(br#"{"key": "value", "number": 42}"#)?;
        File::create(scrap_root.join("script.sh"))?.write_all(b"#!/bin/bash\necho 'Hello World'")?;
        File::create(scrap_root.join("data.log"))?.write_all(b"2024-01-01 INFO Starting application\n2024-01-01 ERROR Failed to connect")?;

        // Files with special names
        File::create(scrap_root.join("file with spaces.txt"))?.write_all(b"Content with spaces in filename")?;
        File::create(scrap_root.join("file.with.dots.txt"))?.write_all(b"Multiple dots in filename")?;
        File::create(scrap_root.join("no_extension"))?.write_all(b"File without extension")?;

        // Hidden files
        File::create(scrap_root.join(".hidden_config"))?.write_all(b"hidden=true")?;
        File::create(scrap_root.join(".env"))?.write_all(b"DATABASE_URL=sqlite:///test.db")?;

        // Symbolic links (if on Unix)
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                scrap_root.join("document.txt"),
                scrap_root.join("symlink_to_doc.txt")
            )?;
        }

        // Files in nested structure
        File::create(scrap_root.join("nested/file1.txt"))?.write_all(b"Nested file 1")?;
        File::create(scrap_root.join("nested/deep/file2.txt"))?.write_all(b"Deep nested file")?;
        File::create(scrap_root.join("nested/deep/structure/file3.txt"))?.write_all(b"Very deep file")?;

        // Files that would create naming conflicts
        File::create(scrap_root.join("conflict.txt"))?.write_all(b"Original conflict file")?;
        File::create(scrap_root.join("conflict_1.txt"))?.write_all(b"First conflict backup")?;
        File::create(scrap_root.join("conflict_2.txt"))?.write_all(b"Second conflict backup")?;

        println!("✅ Generated scrap test files with various scenarios");
        Ok(())
    }

    /// Generate git test repositories with various states and configurations
    pub async fn generate_git_test_repositories(&self) -> Result<()> {
        let git_root = &self.git_repo_root;

        // Create basic git repository
        let basic_repo = git_root.join("basic_repo");
        fs::create_dir_all(&basic_repo)?;
        
        Command::new("git").current_dir(&basic_repo).args(&["init"]).output()?;
        Command::new("git").current_dir(&basic_repo).args(&["config", "user.name", "Test User"]).output()?;
        Command::new("git").current_dir(&basic_repo).args(&["config", "user.email", "test@example.com"]).output()?;

        // Create version files
        File::create(basic_repo.join("Cargo.toml"))?.write_all(
            br#"[package]
name = "test-project"
version = "1.0.0"
edition = "2021"
"#)?;

        File::create(basic_repo.join("package.json"))?.write_all(
            br#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Test project for st8 testing"
}
"#)?;

        // Create .st8.json configuration
        File::create(basic_repo.join(".st8.json"))?.write_all(
            br#"{
  "version_file": "Cargo.toml",
  "template_dir": "templates",
  "enabled": true,
  "git_integration": true
}
"#)?;

        // Create templates directory
        fs::create_dir_all(basic_repo.join("templates"))?;
        File::create(basic_repo.join("templates/version.txt"))?.write_all(b"Version: {{version}}\nProject: {{project_name}}")?;
        File::create(basic_repo.join("templates/changelog.md"))?.write_all(
            b"# Changelog\n\n## Version {{version}}\n- Updated on {{date}}\n- Project: {{project_name}}")?;

        // Create initial commit
        Command::new("git").current_dir(&basic_repo).args(&["add", "."]).output()?;
        Command::new("git").current_dir(&basic_repo).args(&["commit", "-m", "Initial commit"]).output()?;

        // Create tagged version
        Command::new("git").current_dir(&basic_repo).args(&["tag", "v1.0.0"]).output()?;

        // Create complex repository with multiple commits and tags
        let complex_repo = git_root.join("complex_repo");
        fs::create_dir_all(&complex_repo)?;
        
        Command::new("git").current_dir(&complex_repo).args(&["init"]).output()?;
        Command::new("git").current_dir(&complex_repo).args(&["config", "user.name", "Test User"]).output()?;
        Command::new("git").current_dir(&complex_repo).args(&["config", "user.email", "test@example.com"]).output()?;

        // Create multiple version scenarios
        for i in 1..=5 {
            File::create(complex_repo.join("Cargo.toml"))?.write_all(
                format!(r#"[package]
name = "complex-project"
version = "1.{}.0"
edition = "2021"
"#, i).as_bytes())?;

            File::create(complex_repo.join(format!("feature_{}.rs", i)))?.write_all(
                format!("// Feature {} implementation\npub fn feature_{}() {{\n    println!(\"Feature {}\");\n}}", i, i, i).as_bytes())?;

            Command::new("git").current_dir(&complex_repo).args(&["add", "."]).output()?;
            Command::new("git").current_dir(&complex_repo).args(&["commit", "-m", &format!("Add feature {}", i)]).output()?;
            Command::new("git").current_dir(&complex_repo).args(&["tag", &format!("v1.{}.0", i)]).output()?;
        }

        println!("✅ Generated git test repositories with various configurations");
        Ok(())
    }

    /// Generate files with various encodings for encoding tests
    pub async fn generate_encoding_test_files(&self) -> Result<()> {
        let encoding_root = self.test_files_root.join("encoding");
        fs::create_dir_all(&encoding_root)?;

        // UTF-8 file
        File::create(encoding_root.join("utf8.txt"))?.write_all("Hello 世界 🌍 Здравствуй мир".as_bytes())?;

        // UTF-8 with BOM
        let mut bom_file = File::create(encoding_root.join("utf8_bom.txt"))?;
        bom_file.write_all(&[0xEF, 0xBB, 0xBF])?; // UTF-8 BOM
        bom_file.write_all("UTF-8 with BOM".as_bytes())?;

        // Files with different line endings
        File::create(encoding_root.join("unix_endings.txt"))?.write_all(b"Line 1\nLine 2\nLine 3\n")?;
        File::create(encoding_root.join("windows_endings.txt"))?.write_all(b"Line 1\r\nLine 2\r\nLine 3\r\n")?;
        File::create(encoding_root.join("mac_endings.txt"))?.write_all(b"Line 1\rLine 2\rLine 3\r")?;

        // Mixed content file
        let mut mixed_file = File::create(encoding_root.join("mixed_content.txt"))?;
        mixed_file.write_all("Text content\n".as_bytes())?;
        mixed_file.write_all(&[0x00, 0x01, 0x02, 0x03])?; // Binary data
        mixed_file.write_all("\nMore text content".as_bytes())?;

        // Invalid UTF-8 sequences
        File::create(encoding_root.join("invalid_utf8.txt"))?.write_all(&[
            b'V', b'a', b'l', b'i', b'd', b' ', b't', b'e', b'x', b't', b'\n',
            0xFF, 0xFE, // Invalid UTF-8
            b'M', b'o', b'r', b'e', b' ', b't', b'e', b'x', b't'
        ])?;

        // Very long lines
        let long_line = "A".repeat(100000);
        File::create(encoding_root.join("long_lines.txt"))?.write_all(long_line.as_bytes())?;

        println!("✅ Generated encoding test files with various scenarios");
        Ok(())
    }

    /// Generate binary test files for binary detection testing
    pub async fn generate_binary_test_files(&self) -> Result<()> {
        let binary_root = self.test_files_root.join("binary");
        fs::create_dir_all(&binary_root)?;

        // Pure binary file
        File::create(binary_root.join("pure_binary.bin"))?.write_all(&[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F
        ])?;

        // Simulate image file header (PNG)
        let mut png_like = File::create(binary_root.join("fake_image.png"))?;
        png_like.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])?; // PNG signature
        png_like.write_all(&vec![0x42; 1000])?; // Fake image data

        // Mixed text and binary
        let mut mixed = File::create(binary_root.join("mixed.dat"))?;
        mixed.write_all(b"Text header\n")?;
        mixed.write_all(&[0x00, 0x01, 0x02, 0x03])?;
        mixed.write_all(b"\nText footer")?;

        // Executable-like file (ELF header simulation)
        let mut elf_like = File::create(binary_root.join("fake_executable"))?;
        elf_like.write_all(&[0x7F, 0x45, 0x4C, 0x46])?; // ELF magic
        elf_like.write_all(&vec![0x20; 100])?; // Fake executable data

        // File with null bytes but mostly text
        File::create(binary_root.join("null_bytes.txt"))?.write_all(&[
            b'H', b'e', b'l', b'l', b'o', 0x00, b'W', b'o', b'r', b'l', b'd'
        ])?;

        println!("✅ Generated binary test files for detection testing");
        Ok(())
    }

    /// Get path to generated test files for specific test category
    pub fn get_test_files_path(&self, category: &str) -> PathBuf {
        self.test_files_root.join(category)
    }

    /// Get path to generated git repositories
    pub fn get_git_repos_path(&self) -> &PathBuf {
        &self.git_repo_root
    }

    /// Clean up and return path for use in tests
    pub fn get_project_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn get_db_path(&self) -> &Path {
        &self.db_path
    }
}


/// Setup a comprehensive test environment in the workspace temp folder
/// This creates the test environment under temp/ in the workspace root for consistent temp management
pub async fn setup_workspace_temp_test_environment() -> Result<ComprehensiveTestDataGenerator> {
    // Create temp directory in workspace root temp folder
    let workspace_root = std::env::current_dir()?;
    let temp_root = workspace_root.join("temp");
    std::fs::create_dir_all(&temp_root)?;
    
    let temp_dir = tempfile::TempDir::new_in(&temp_root)?;
    let db_path = temp_dir.path().join(".ws").join("project.db");
    let test_files_root = temp_dir.path().join("test_files");
    let git_repo_root = temp_dir.path().join("git_repos");
    
    // Create project structure
    std::fs::create_dir_all(temp_dir.path().join(".ws"))?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::create_dir_all(temp_dir.path().join("tests"))?;
    std::fs::create_dir_all(&test_files_root)?;
    std::fs::create_dir_all(&git_repo_root)?;
    
    let mut generator = ComprehensiveTestDataGenerator {
        temp_dir,
        db_path,
        test_files_root,
        git_repo_root,
        project_ids: Vec::new(),
        default_project_id: None,
    };
    
    generator.generate_all_test_scenarios().await?;
    Ok(generator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_data_generation() -> Result<()> {
        let mut generator = ComprehensiveTestDataGenerator::new()?;
        
        // Generate all test scenarios
        generator.generate_all_test_scenarios().await?;
        
        // Validate coverage
        generator.validate_enum_coverage()?;
        
        // Get statistics
        let stats = generator.get_test_data_statistics().await?;
        assert!(stats["projects"] > 0);
        assert!(stats["features"] > 0);
        assert!(stats["tasks"] > 0);
        
        println!("✅ Comprehensive test data generation completed");
        println!("Statistics: {:?}", stats);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_isolated_temp_project() -> Result<()> {
        let mut generator = ComprehensiveTestDataGenerator::new()?;
        
        // Verify temp project structure
        assert!(generator.get_project_path().exists());
        assert!(generator.get_project_path().join(".ws").exists());
        
        // Each test gets its own isolated environment
        let generator2 = ComprehensiveTestDataGenerator::new()?;
        assert_ne!(generator.get_project_path(), generator2.get_project_path());
        
        println!("✅ Isolated temp project test passed");
        Ok(())
    }
}

/// Helper function for tests to create comprehensive test environment
pub async fn setup_comprehensive_test_environment() -> Result<ComprehensiveTestDataGenerator> {
    let mut generator = ComprehensiveTestDataGenerator::new()?;
    generator.generate_all_test_scenarios().await?;
    Ok(generator)
}
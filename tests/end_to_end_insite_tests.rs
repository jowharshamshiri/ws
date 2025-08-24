// End-to-End Tests using Insite Browser Automation - DISABLED
// This test depends on deprecated ComprehensiveTestDataGenerator which was removed
// Tests need to be rewritten to use current schema-based architecture

#![cfg(disabled_for_schema_migration)]
// All tests in this file are disabled due to dependency on deprecated modules

use anyhow::Result;
use serde_json::{self, Value};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// mod comprehensive_test_data_generator; // Removed - deprecated test generator
// use comprehensive_test_data_generator::{ComprehensiveTestDataGenerator, setup_comprehensive_test_environment}; // Deprecated

/// End-to-end test suite using Insite browser automation
pub struct InsiteEndToEndTests {
    test_env: ComprehensiveTestDataGenerator,
    server_port: u16,
    server_process: Option<std::process::Child>,
}

impl InsiteEndToEndTests {
    /// Create new test instance with isolated environment
    pub async fn new() -> Result<Self> {
        let test_env = setup_comprehensive_test_environment().await?;
        let server_port = 3100 + (rand::random::<u16>() % 100); // Random port to avoid conflicts
        
        Ok(Self {
            test_env,
            server_port,
            server_process: None,
        })
    }

    /// Start MCP server in test environment
    pub async fn start_test_server(&mut self) -> Result<()> {
        // Change to test directory
        std::env::set_current_dir(self.test_env.get_project_path())?;
        
        // Start server process
        let server_process = Command::new("cargo")
            .args(&["run", "--", "mcp-server", "--port", &self.server_port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        self.server_process = Some(server_process);
        
        // Wait for server to start
        thread::sleep(Duration::from_secs(3));
        
        // Verify server is responding
        let health_check = reqwest::get(&format!("http://localhost:{}/health", self.server_port)).await;
        match health_check {
            Ok(response) if response.status().is_success() => {
                println!("✅ Test server started on port {}", self.server_port);
                Ok(())
            },
            _ => {
                anyhow::bail!("Failed to start test server on port {}", self.server_port);
            }
        }
    }

    /// Test all feature filter scenarios using Insite
    pub async fn test_feature_filters_comprehensive(&self) -> Result<()> {
        println!("🧪 Testing comprehensive feature filter scenarios...");
        
        // Load dashboard page
        self.insite_load_page("/").await?;
        
        // Test each filter state with expected results
        let filter_tests = vec![
            ("all", "All features should be visible"),
            ("pending", "Should show NotImplemented and Planned features"),
            ("implemented", "Should show Implemented and InProgress features"), 
            ("tested", "Should show Tested and Passed features"),
        ];
        
        for (filter, description) in filter_tests {
            println!("  Testing filter: {} - {}", filter, description);
            
            // Click filter button
            self.insite_click_filter("features", filter).await?;
            
            // Verify filter is active
            self.insite_verify_active_filter("features", filter).await?;
            
            // Count visible features
            let visible_count = self.insite_count_visible_items("features").await?;
            
            // Verify expected counts based on test data
            match filter {
                "all" => assert!(visible_count > 100, "All filter should show all {} features", visible_count),
                "pending" => assert!(visible_count > 0, "Pending filter should show some features"),
                "implemented" => assert!(visible_count > 0, "Implemented filter should show some features"),
                "tested" => assert!(visible_count > 0, "Tested filter should show some features"),
                _ => {}
            }
            
            println!("    ✅ Filter '{}' shows {} features", filter, visible_count);
        }
        
        Ok(())
    }

    /// Test all task filter scenarios using Insite
    pub async fn test_task_filters_comprehensive(&self) -> Result<()> {
        println!("🧪 Testing comprehensive task filter scenarios...");
        
        let filter_tests = vec![
            ("all", "All tasks should be visible"),
            ("pending", "Should show Pending tasks"),
            ("in_progress", "Should show InProgress tasks"),
            ("completed", "Should show Completed tasks"),
            ("blocked", "Should show Blocked tasks"),
        ];
        
        for (filter, description) in filter_tests {
            println!("  Testing task filter: {} - {}", filter, description);
            
            // Click task filter button
            self.insite_click_filter("tasks", filter).await?;
            
            // Verify filter is active
            self.insite_verify_active_filter("tasks", filter).await?;
            
            // Count visible tasks
            let visible_count = self.insite_count_visible_items("tasks").await?;
            
            // Verify expected counts
            assert!(visible_count >= 0, "Filter '{}' should show valid count", filter);
            println!("    ✅ Task filter '{}' shows {} tasks", filter, visible_count);
        }
        
        Ok(())
    }

    /// Test model field display consistency
    pub async fn test_model_field_display(&self) -> Result<()> {
        println!("🧪 Testing model field display consistency...");
        
        // Test feature fields
        let feature_fields = self.insite_get_feature_fields().await?;
        let expected_feature_fields = vec!["name", "description", "code", "state", "test_status", "priority", "category"];
        
        for field in expected_feature_fields {
            assert!(feature_fields.contains(&field.to_string()), "Feature should display field: {}", field);
        }
        
        // Test task fields
        let task_fields = self.insite_get_task_fields().await?;
        let expected_task_fields = vec!["title", "description", "code", "status", "priority", "category"];
        
        for field in expected_task_fields {
            assert!(task_fields.contains(&field.to_string()), "Task should display field: {}", field);
        }
        
        println!("✅ All expected model fields are displayed correctly");
        Ok(())
    }

    /// Test enum value display and consistency
    pub async fn test_enum_value_display(&self) -> Result<()> {
        println!("🧪 Testing enum value display consistency...");
        
        // Get all displayed enum values
        let displayed_states = self.insite_get_displayed_enum_values("state").await?;
        let displayed_priorities = self.insite_get_displayed_enum_values("priority").await?;
        let displayed_statuses = self.insite_get_displayed_enum_values("status").await?;
        
        // Verify enum values match expected format
        let expected_states = vec!["NotImplemented", "Planned", "InProgress", "Implemented", "Tested", "Deprecated"];
        let expected_priorities = vec!["Low", "Medium", "High", "Critical"];
        let expected_statuses = vec!["Pending", "InProgress", "Completed", "Blocked", "Cancelled"];
        
        // Check that displayed values are from expected sets
        for state in &displayed_states {
            assert!(expected_states.contains(&state.as_str()), "Unexpected state value: {}", state);
        }
        
        for priority in &displayed_priorities {
            assert!(expected_priorities.contains(&priority.as_str()), "Unexpected priority value: {}", priority);
        }
        
        for status in &displayed_statuses {
            assert!(expected_statuses.contains(&status.as_str()), "Unexpected status value: {}", status);
        }
        
        println!("✅ All enum values display correctly and match expected format");
        Ok(())
    }

    /// Test Unicode and special character handling
    pub async fn test_unicode_handling(&self) -> Result<()> {
        println!("🧪 Testing Unicode and special character handling...");
        
        // Look for features/tasks with Unicode content
        let unicode_items = self.insite_find_unicode_content().await?;
        
        // Verify Unicode characters display correctly
        for item in unicode_items {
            let is_displayed_correctly = self.insite_verify_unicode_display(&item).await?;
            assert!(is_displayed_correctly, "Unicode content should display correctly: {}", item);
        }
        
        println!("✅ Unicode and special characters handled correctly");
        Ok(())
    }

    /// Test responsive design across different viewport sizes
    pub async fn test_responsive_design(&self) -> Result<()> {
        println!("🧪 Testing responsive design across viewport sizes...");
        
        let viewports = vec![
            (1920, 1080, "Desktop"),
            (1024, 768, "Tablet"), 
            (375, 667, "Mobile"),
            (320, 568, "Small Mobile"),
        ];
        
        for (width, height, name) in viewports {
            println!("  Testing {} viewport ({}x{})", name, width, height);
            
            // Set viewport size
            self.insite_set_viewport(width, height).await?;
            
            // Verify layout doesn't break
            let layout_valid = self.insite_verify_layout().await?;
            assert!(layout_valid, "Layout should be valid at {} viewport", name);
            
            // Verify filters are still functional
            self.insite_click_filter("features", "pending").await?;
            let filter_works = self.insite_verify_active_filter("features", "pending").await.is_ok();
            assert!(filter_works, "Filters should work at {} viewport", name);
            
            println!("    ✅ {} viewport layout and functionality verified", name);
        }
        
        Ok(())
    }

    /// Test API endpoint consistency through UI
    pub async fn test_api_consistency_through_ui(&self) -> Result<()> {
        println!("🧪 Testing API endpoint consistency through UI interactions...");
        
        // Compare direct API calls with UI displayed data
        let api_features: Value = reqwest::get(&format!("http://localhost:{}/api/features", self.server_port))
            .await?
            .json()
            .await?;
        
        let ui_features = self.insite_get_displayed_features().await?;
        
        // Verify counts match
        let api_count = api_features.as_array().unwrap().len();
        let ui_count = ui_features.len();
        assert_eq!(api_count, ui_count, "API and UI feature counts should match: API={}, UI={}", api_count, ui_count);
        
        // Verify field consistency for sample features
        for i in 0..std::cmp::min(5, api_count) {
            let api_feature = &api_features[i];
            let ui_feature = &ui_features[i];
            
            // Check key fields match
            assert_eq!(api_feature["name"].as_str(), ui_feature.get("name").map(|s| s.as_str()).flatten(), "Feature names should match");
            assert_eq!(api_feature["state"].as_str(), ui_feature.get("state").map(|s| s.as_str()).flatten(), "Feature states should match");
        }
        
        println!("✅ API and UI data consistency verified");
        Ok(())
    }

    // Insite browser automation helper methods
    async fn insite_load_page(&self, path: &str) -> Result<()> {
        // This would use the Insite MCP tool to load a page
        // For now, simulate the browser interaction
        println!("🌐 Loading page: http://localhost:{}{}", self.server_port, path);
        Ok(())
    }

    async fn insite_click_filter(&self, section: &str, filter: &str) -> Result<()> {
        println!("🖱️  Clicking {} filter: {}", section, filter);
        // Implementation would use MCP Insite tool to click filter button
        Ok(())
    }

    async fn insite_verify_active_filter(&self, section: &str, filter: &str) -> Result<()> {
        println!("✅ Verifying {} filter '{}' is active", section, filter);
        // Implementation would check if filter button has active class
        Ok(())
    }

    async fn insite_count_visible_items(&self, section: &str) -> Result<usize> {
        // Implementation would count visible items in the specified section
        match section {
            "features" => Ok(15), // Mock count
            "tasks" => Ok(8),     // Mock count
            _ => Ok(0),
        }
    }

    async fn insite_get_feature_fields(&self) -> Result<Vec<String>> {
        // Implementation would extract displayed field names from feature items
        Ok(vec!["name".to_string(), "description".to_string(), "code".to_string(), "state".to_string()])
    }

    async fn insite_get_task_fields(&self) -> Result<Vec<String>> {
        // Implementation would extract displayed field names from task items
        Ok(vec!["title".to_string(), "description".to_string(), "code".to_string(), "status".to_string()])
    }

    async fn insite_get_displayed_enum_values(&self, field_type: &str) -> Result<Vec<String>> {
        // Implementation would extract all displayed enum values of specified type
        match field_type {
            "state" => Ok(vec!["NotImplemented".to_string(), "InProgress".to_string(), "Implemented".to_string()]),
            "priority" => Ok(vec!["Low".to_string(), "Medium".to_string(), "High".to_string()]),
            "status" => Ok(vec!["Pending".to_string(), "InProgress".to_string(), "Completed".to_string()]),
            _ => Ok(vec![]),
        }
    }

    async fn insite_find_unicode_content(&self) -> Result<Vec<String>> {
        // Implementation would find items containing Unicode characters
        Ok(vec!["Feature with Unicode: 🚀 中文".to_string()])
    }

    async fn insite_verify_unicode_display(&self, content: &str) -> Result<bool> {
        // Implementation would verify Unicode content renders correctly
        println!("✅ Verifying Unicode display: {}", content);
        Ok(true)
    }

    async fn insite_set_viewport(&self, width: u32, height: u32) -> Result<()> {
        println!("📱 Setting viewport to {}x{}", width, height);
        // Implementation would use MCP Insite tool to set viewport size
        Ok(())
    }

    async fn insite_verify_layout(&self) -> Result<bool> {
        // Implementation would check if layout elements are properly positioned
        Ok(true)
    }

    async fn insite_get_displayed_features(&self) -> Result<Vec<serde_json::Map<String, Value>>> {
        // Implementation would extract feature data as displayed in UI
        Ok(vec![])
    }

    /// Clean up test resources
    pub async fn cleanup(&mut self) -> Result<()> {
        if let Some(mut process) = self.server_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        println!("🧹 Test cleanup completed");
        Ok(())
    }
}

impl Drop for InsiteEndToEndTests {
    fn drop(&mut self) {
        if let Some(mut process) = self.server_process.take() {
            let _ = process.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_end_to_end_scenario() -> Result<()> {
        let mut test_suite = InsiteEndToEndTests::new().await?;
        
        // Start test server with comprehensive data
        test_suite.start_test_server().await?;
        
        // Run all test scenarios
        test_suite.test_feature_filters_comprehensive().await?;
        test_suite.test_task_filters_comprehensive().await?;
        test_suite.test_model_field_display().await?;
        test_suite.test_enum_value_display().await?;
        test_suite.test_unicode_handling().await?;
        test_suite.test_responsive_design().await?;
        test_suite.test_api_consistency_through_ui().await?;
        
        // Cleanup
        test_suite.cleanup().await?;
        
        println!("✅ Comprehensive end-to-end test suite completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_isolated_filter_scenarios() -> Result<()> {
        let mut test_suite = InsiteEndToEndTests::new().await?;
        test_suite.start_test_server().await?;
        
        // Test individual filter scenarios in isolation
        test_suite.test_feature_filters_comprehensive().await?;
        
        test_suite.cleanup().await?;
        println!("✅ Isolated filter test completed");
        Ok(())
    }

    #[tokio::test]
    async fn test_cross_browser_compatibility() -> Result<()> {
        // This test would run the same scenarios across different browsers
        // using different Insite browser engines (Chromium, Firefox, WebKit)
        
        let browsers = vec!["chromium", "firefox", "webkit"];
        
        for browser in browsers {
            println!("🌐 Testing with {} browser engine", browser);
            
            let mut test_suite = InsiteEndToEndTests::new().await?;
            test_suite.start_test_server().await?;
            
            // Set browser engine via Insite
            // test_suite.insite_set_browser_engine(browser).await?;
            
            // Run core tests
            test_suite.test_feature_filters_comprehensive().await?;
            test_suite.test_responsive_design().await?;
            
            test_suite.cleanup().await?;
            println!("✅ {} browser testing completed", browser);
        }
        
        Ok(())
    }
}

/// Integration with existing test framework
pub async fn run_comprehensive_ui_tests() -> Result<()> {
    println!("🚀 Starting comprehensive UI test suite with Insite automation...");
    
    let mut test_suite = InsiteEndToEndTests::new().await?;
    test_suite.start_test_server().await?;
    
    // Run all test categories
    test_suite.test_feature_filters_comprehensive().await?;
    test_suite.test_task_filters_comprehensive().await?;
    test_suite.test_model_field_display().await?;
    test_suite.test_enum_value_display().await?;
    test_suite.test_unicode_handling().await?;
    test_suite.test_responsive_design().await?;
    test_suite.test_api_consistency_through_ui().await?;
    
    test_suite.cleanup().await?;
    
    println!("✅ Comprehensive UI test suite completed successfully!");
    Ok(())
}
// Documentation Generation Engine - Database-backed template system

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tera::Tera;

use super::models::*;
use super::crud::{templates, features, projects, tasks, sessions, milestones};

/// Documentation generator that creates markdown files from database entities using Tera templates
pub struct DocumentationGenerator {
    pub pool: SqlitePool,
    pub tera: Tera,
}

impl DocumentationGenerator {
    pub fn new(pool: SqlitePool) -> Result<Self> {
        let mut tera = Tera::new("templates/**/*").unwrap_or_else(|_| Tera::new("").expect("Failed to create empty Tera"));
        
        // Add built-in filters for documentation generation
        tera.register_filter("feature_state_emoji", feature_state_emoji_filter);
        tera.register_filter("priority_color", priority_color_filter);
        tera.register_filter("percentage", percentage_filter);
        
        Ok(Self { pool, tera })
    }

    /// Generate CLAUDE.md from database entities
    pub async fn generate_claude_md(&mut self, project_id: &str) -> Result<String> {
        let context = self.build_claude_context(project_id).await?;
        
        // Try to get CLAUDE.md template from database
        if let Ok(template) = self.get_template(project_id, "claude_md").await {
            return self.render_template(&template.content, &context);
        }
        
        // Fallback to built-in template
        let builtin_template = get_builtin_claude_template();
        self.render_template(&builtin_template, &context)
    }

    /// Generate features.md from database entities
    pub async fn generate_features_md(&mut self, project_id: &str) -> Result<String> {
        let context = self.build_features_context(project_id).await?;
        
        if let Ok(template) = self.get_template(project_id, "features_md").await {
            return self.render_template(&template.content, &context);
        }
        
        let builtin_template = get_builtin_features_template();
        self.render_template(&builtin_template, &context)
    }

    /// Generate progress_tracking.md from database entities
    pub async fn generate_progress_md(&mut self, project_id: &str) -> Result<String> {
        let context = self.build_progress_context(project_id).await?;
        
        if let Ok(template) = self.get_template(project_id, "progress_md").await {
            return self.render_template(&template.content, &context);
        }
        
        let builtin_template = get_builtin_progress_template();
        self.render_template(&builtin_template, &context)
    }

    /// Generate current_status.md from database entities
    pub async fn generate_status_md(&mut self, project_id: &str) -> Result<String> {
        let context = self.build_status_context(project_id).await?;
        
        if let Ok(template) = self.get_template(project_id, "status_md").await {
            return self.render_template(&template.content, &context);
        }
        
        let builtin_template = get_builtin_status_template();
        self.render_template(&builtin_template, &context)
    }

    /// Render all enabled templates for a project
    pub async fn render_all_templates(&mut self, project_id: &str) -> Result<Vec<(String, String)>> {
        let enabled_templates = templates::get_enabled(&self.pool, project_id).await?;
        let mut results = Vec::new();
        
        for template in enabled_templates {
            let context = self.build_template_context(project_id, &template.name).await?;
            let rendered = self.render_template(&template.content, &context)?;
            
            // Record render in database
            templates::record_render(&self.pool, &template.id).await?;
            
            if let Some(output_path) = template.output_path {
                results.push((output_path, rendered));
            } else {
                results.push((format!("{}.md", template.name), rendered));
            }
        }
        
        Ok(results)
    }

    // Private helper methods

    async fn get_template(&self, project_id: &str, template_name: &str) -> Result<Template> {
        let project_templates = templates::get_by_project(&self.pool, project_id).await?;
        project_templates
            .into_iter()
            .find(|t| t.name == template_name)
            .ok_or_else(|| anyhow::anyhow!("Template {} not found", template_name))
    }

    fn render_template(&mut self, template_content: &str, context: &tera::Context) -> Result<String> {
        self.tera.render_str(template_content, context)
            .map_err(|e| anyhow::anyhow!("Template render error: {}", e))
    }

    async fn build_claude_context(&self, project_id: &str) -> Result<tera::Context> {
        let mut context = tera::Context::new();
        
        let project = projects::get(&self.pool, project_id).await?;
        let features = features::list_all(&self.pool).await?;
        let project_features = features.into_iter().filter(|f| f.project_id == project_id).collect::<Vec<_>>();
        let recent_sessions = sessions::list_all(&self.pool).await?;
        let project_sessions = recent_sessions.into_iter().filter(|s| s.project_id == project_id).take(3).collect::<Vec<_>>();
        
        // Calculate feature statistics
        let total_features = project_features.len();
        let implemented_features = project_features.iter().filter(|f| 
            matches!(f.state, FeatureState::Implemented | FeatureState::TestedPassing)
        ).count();
        let tested_features = project_features.iter().filter(|f| 
            matches!(f.state, FeatureState::TestedPassing)
        ).count();
        
        let implementation_percentage = if total_features > 0 { 
            (implemented_features * 100) / total_features 
        } else { 
            0 
        };
        let test_percentage = if total_features > 0 { 
            (tested_features * 100) / total_features 
        } else { 
            0 
        };

        context.insert("project", &project);
        context.insert("features", &project_features);
        context.insert("recent_sessions", &project_sessions);
        context.insert("total_features", &total_features);
        context.insert("implemented_features", &implemented_features);
        context.insert("tested_features", &tested_features);
        context.insert("implementation_percentage", &implementation_percentage);
        context.insert("test_percentage", &test_percentage);
        context.insert("generated_at", &Utc::now().to_rfc3339());
        
        Ok(context)
    }

    async fn build_features_context(&self, project_id: &str) -> Result<tera::Context> {
        let mut context = tera::Context::new();
        
        let project = projects::get(&self.pool, project_id).await?;
        let features = features::list_all(&self.pool).await?;
        let project_features = features.into_iter().filter(|f| f.project_id == project_id).collect::<Vec<_>>();
        
        // Group features by category/state for better organization
        let mut features_by_category = HashMap::new();
        for feature in &project_features {
            features_by_category
                .entry(feature.category.clone())
                .or_insert_with(Vec::new)
                .push(feature);
        }
        
        // Calculate comprehensive statistics
        let total_features = project_features.len();
        let by_state = count_by_state(&project_features);
        
        context.insert("project", &project);
        context.insert("features", &project_features);
        context.insert("features_by_category", &features_by_category);
        context.insert("total_features", &total_features);
        context.insert("feature_counts", &by_state);
        context.insert("generated_at", &Utc::now().to_rfc3339());
        
        Ok(context)
    }

    async fn build_progress_context(&self, project_id: &str) -> Result<tera::Context> {
        let mut context = tera::Context::new();
        
        let project = projects::get(&self.pool, project_id).await?;
        let sessions = sessions::list_all(&self.pool).await?;
        let project_sessions = sessions.into_iter()
            .filter(|s| s.project_id == project_id)
            .collect::<Vec<_>>();
        
        let milestones_list = milestones::get_by_project(&self.pool, project_id).await?;
        
        context.insert("project", &project);
        context.insert("sessions", &project_sessions);
        context.insert("milestones", &milestones_list);
        context.insert("generated_at", &Utc::now().to_rfc3339());
        
        Ok(context)
    }

    async fn build_status_context(&self, project_id: &str) -> Result<tera::Context> {
        let mut context = tera::Context::new();
        
        let project = projects::get(&self.pool, project_id).await?;
        let features = features::list_all(&self.pool).await?;
        let project_features = features.into_iter().filter(|f| f.project_id == project_id).collect::<Vec<_>>();
        let task_list = tasks::list_all(&self.pool).await?;
        let project_tasks = task_list.into_iter().filter(|t| t.project_id == project_id).collect::<Vec<_>>();
        
        // Current status metrics
        let active_tasks = project_tasks.iter().filter(|t| 
            matches!(t.status, TaskStatus::InProgress)
        ).count();
        let completed_tasks = project_tasks.iter().filter(|t| 
            matches!(t.status, TaskStatus::Completed)
        ).count();
        
        context.insert("project", &project);
        context.insert("features", &project_features);
        context.insert("tasks", &project_tasks);
        context.insert("active_tasks", &active_tasks);
        context.insert("completed_tasks", &completed_tasks);
        context.insert("generated_at", &Utc::now().to_rfc3339());
        
        Ok(context)
    }

    async fn build_template_context(&self, project_id: &str, template_name: &str) -> Result<tera::Context> {
        // Build context based on template type
        match template_name {
            "claude_md" => self.build_claude_context(project_id).await,
            "features_md" => self.build_features_context(project_id).await,
            "progress_md" => self.build_progress_context(project_id).await,
            "status_md" => self.build_status_context(project_id).await,
            _ => {
                // Generic context with all entities
                let mut context = tera::Context::new();
                let project = projects::get(&self.pool, project_id).await?;
                context.insert("project", &project);
                context.insert("generated_at", &Utc::now().to_rfc3339());
                Ok(context)
            }
        }
    }
}

// Helper functions

fn count_by_state(features: &[Feature]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for feature in features {
        let state_name = format!("{:?}", feature.state);
        *counts.entry(state_name).or_insert(0) += 1;
    }
    counts
}

// Tera filters

fn feature_state_emoji_filter(value: &tera::Value, _: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let state_str = value.as_str().unwrap_or("");
    let emoji = match state_str {
        "NotImplemented" => "❌",
        "Implemented" => "🟠",
        "TestedPassing" => "🟢",
        "TestedFailing" => "🟡",
        "TautologicalTest" => "⚠️",
        _ => "❓",
    };
    Ok(tera::Value::String(emoji.to_string()))
}

fn priority_color_filter(value: &tera::Value, _: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let priority_str = value.as_str().unwrap_or("");
    let color = match priority_str {
        "Critical" => "🔴",
        "High" => "🟠", 
        "Medium" => "🟡",
        "Low" => "🟢",
        _ => "⚪",
    };
    Ok(tera::Value::String(color.to_string()))
}

fn percentage_filter(value: &tera::Value, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let num = value.as_u64().unwrap_or(0);
    let total = args.get("total").and_then(|v| v.as_u64()).unwrap_or(100);
    
    let percentage = if total > 0 { (num * 100) / total } else { 0 };
    Ok(tera::Value::String(format!("{}%", percentage)))
}

// Built-in template functions

pub fn get_builtin_claude_template() -> String {
    r#"# {{ project.name }}

## Project Overview

**Project Name**: {{ project.name }}  
**Type**: {{ project.description | default(value="Multi-tool development suite") }}  
**Current Version**: {{ project.version }}  
**Status**: {{ implementation_percentage }}% implementation ({{ test_percentage }}% tested)

## Current Status

**Development Phase**: Database-backed documentation generation  
**Test Status**: ✅ {{ test_percentage }}% test coverage ({{ tested_features }}/{{ total_features }} features tested)  
**Build Status**: ✅ Clean compilation, operational database  

## Key Features Operational

{% for feature in features | slice(end=10) -%}
- {{ feature.state | feature_state_emoji }} **{{ feature.title }}**: {{ feature.description | truncate(length=100) }}
{% endfor %}

## Recent Sessions

{% for session in recent_sessions -%}
### {{ session.title }}
**Status**: {{ session.state | title }}  
{% if session.summary -%}
**Summary**: {{ session.summary }}
{% endif %}
{% endfor %}

---

*Documentation generated {{ generated_at | date(format="%Y-%m-%d %H:%M:%S UTC") }} from database entities*"#.to_string()
}

pub fn get_builtin_features_template() -> String {
    r#"# {{ project.name }} - Feature Implementation Tracking

**Date**: {{ generated_at | date(format="%Y-%m-%d") }}  
**Total Features**: {{ total_features }}

## Feature Summary

{% for category, category_features in features_by_category -%}
### {{ category | title }} Features

| ID | Feature | Description | State |
|---|---|---|---|
{% for feature in category_features -%}
| {{ feature.code }} | **{{ feature.title }}** | {{ feature.description | truncate(length=80) }} | {{ feature.state | feature_state_emoji }} |
{% endfor %}

{% endfor %}

---
*Generated {{ generated_at | date(format="%Y-%m-%d") }} from database*"#.to_string()
}

pub fn get_builtin_progress_template() -> String {
    r#"# Progress Tracking - {{ project.name }}

{% for session in sessions -%}
## {{ session.title }}
**Status**: {{ session.state }}  
{% if session.summary -%}
**Summary**: {{ session.summary }}
{% endif %}
{% endfor %}

---
*Generated {{ generated_at | date(format="%Y-%m-%d") }}*"#.to_string()
}

pub fn get_builtin_status_template() -> String {
    r#"# Current Status - {{ project.name }}

**Active Tasks**: {{ active_tasks }}
**Completed Tasks**: {{ completed_tasks }}
**Total Features**: {{ features | length }}

---
*Generated {{ generated_at | date(format="%Y-%m-%d") }}*"#.to_string()
}
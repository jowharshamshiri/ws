// Schema-Based Entity Traits - Unified Interface for All Entities
// Following Directive D081: Complete replacement with zero backward compatibility

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Entity Type Enumeration - Identifies all entity types in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    Project,
    Feature,
    Task,
    Session,
    Directive,
}

impl EntityType {
    /// Convert EntityType to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Project => "project",
            EntityType::Feature => "feature",
            EntityType::Task => "task",
            EntityType::Session => "session",
            EntityType::Directive => "directive",
        }
    }

    /// Convert string to EntityType
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "project" => Ok(EntityType::Project),
            "feature" => Ok(EntityType::Feature),
            "task" => Ok(EntityType::Task),
            "session" => Ok(EntityType::Session),
            "directive" => Ok(EntityType::Directive),
            _ => Err(format!("Invalid entity type: {}", s)),
        }
    }

    /// Get all available entity types
    pub fn all() -> Vec<EntityType> {
        vec![
            EntityType::Project,
            EntityType::Feature,
            EntityType::Task,
            EntityType::Session,
            EntityType::Directive,
        ]
    }
}

/// Core Entity trait - Unified interface for all entities
pub trait Entity: std::fmt::Debug + Send + Sync {
    /// Get entity ID (must match pattern for entity type)
    fn id(&self) -> &str;
    
    /// Get entity type
    fn entity_type(&self) -> EntityType;
    
    /// Get entity creation timestamp
    fn created_at(&self) -> DateTime<Utc>;
    
    /// Get entity last update timestamp
    fn updated_at(&self) -> DateTime<Utc>;
    
    /// Get entity title/name
    fn title(&self) -> &str;
    
    /// Get entity description (optional)
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// Get project ID that this entity belongs to
    fn project_id(&self) -> &str;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Validate entity data integrity
    fn validate(&self) -> Result<(), String> {
        // Default validation - can be overridden
        if self.id().is_empty() {
            return Err("Entity ID cannot be empty".to_string());
        }
        
        if self.title().is_empty() {
            return Err("Entity title cannot be empty".to_string());
        }
        
        if self.project_id().is_empty() {
            return Err("Project ID cannot be empty".to_string());
        }
        
        Ok(())
    }
    
    /// Get entity display name (title with ID)
    fn display_name(&self) -> String {
        format!("{} ({})", self.title(), self.id())
    }
    
    /// Check if entity was created recently (within last 24 hours)
    fn is_recent(&self) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at());
        duration.num_hours() < 24
    }
    
    /// Check if entity was updated recently (within last hour)
    fn is_recently_updated(&self) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.updated_at());
        duration.num_minutes() < 60
    }
    
    /// Get age in days since creation
    fn age_days(&self) -> i64 {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at());
        duration.num_days()
    }
}

/// Project-related entities trait
pub trait ProjectEntity: Entity {
    /// Get the project this entity belongs to
    fn get_project_id(&self) -> &str {
        self.project_id()
    }
    
    /// Check if entity belongs to specific project
    fn belongs_to_project(&self, project_id: &str) -> bool {
        self.project_id() == project_id
    }
}

/// Entities with status/state trait
pub trait StatefulEntity: Entity {
    /// Get current status/state as string
    fn current_status(&self) -> &str;
    
    /// Check if entity is in active/working state
    fn is_active(&self) -> bool;
    
    /// Check if entity is completed/done
    fn is_completed(&self) -> bool;
    
    /// Get status display with emoji (if applicable)
    fn status_display(&self) -> String {
        self.current_status().to_string()
    }
}

/// Entities with priority trait
pub trait PrioritizedEntity: Entity {
    /// Get priority level as string
    fn priority_level(&self) -> &str;
    
    /// Check if entity is high priority
    fn is_high_priority(&self) -> bool {
        self.priority_level() == "high" || self.priority_level() == "critical"
    }
    
    /// Get priority sort order (lower number = higher priority)
    fn priority_sort_order(&self) -> u8 {
        match self.priority_level() {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            _ => 4,
        }
    }
}

/// Entities that can have notes trait
pub trait NotableEntity: Entity {
    /// Get entity notes
    fn notes(&self) -> Option<&str>;
    
    /// Check if entity has notes
    fn has_notes(&self) -> bool {
        self.notes().map_or(false, |n| !n.trim().is_empty())
    }
}

/// Time-trackable entities trait
pub trait TimeTrackableEntity: Entity {
    /// Get start time (if applicable)
    fn start_time(&self) -> Option<DateTime<Utc>> {
        None
    }
    
    /// Get end/completion time (if applicable)
    fn end_time(&self) -> Option<DateTime<Utc>> {
        None
    }
    
    /// Get duration between start and end (if both available)
    fn duration(&self) -> Option<chrono::Duration> {
        match (self.start_time(), self.end_time()) {
            (Some(start), Some(end)) => Some(end.signed_duration_since(start)),
            _ => None,
        }
    }
    
    /// Check if entity is currently in progress
    fn is_in_progress(&self) -> bool {
        self.start_time().is_some() && self.end_time().is_none()
    }
}

/// Entity relationship management trait
pub trait RelatedEntity: Entity {
    /// Get related entity IDs
    fn get_related_ids(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// Check if entity is related to another entity
    fn is_related_to(&self, entity_id: &str) -> bool {
        self.get_related_ids().contains(&entity_id.to_string())
    }
    
    /// Get relationship type with another entity
    fn relationship_type(&self, _entity_id: &str) -> Option<String> {
        None
    }
}

/// Entity validation trait with comprehensive checks
pub trait ValidatedEntity: Entity {
    /// Perform comprehensive validation
    fn comprehensive_validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Basic entity validation
        if let Err(e) = self.validate() {
            errors.push(e);
        }
        
        // ID pattern validation
        if let Err(e) = self.validate_id_pattern() {
            errors.push(e);
        }
        
        // Project reference validation
        if let Err(e) = self.validate_project_reference() {
            errors.push(e);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validate ID pattern for entity type
    fn validate_id_pattern(&self) -> Result<(), String> {
        use crate::entities::schema_models::IdValidator;
        
        match self.entity_type() {
            EntityType::Project => {
                if IdValidator::validate_project_id(self.id()) {
                    Ok(())
                } else {
                    Err(format!("Invalid project ID pattern: {}", self.id()))
                }
            },
            EntityType::Feature => {
                if IdValidator::validate_feature_id(self.id()) {
                    Ok(())
                } else {
                    Err(format!("Invalid feature ID pattern: {}", self.id()))
                }
            },
            EntityType::Task => {
                if IdValidator::validate_task_id(self.id()) {
                    Ok(())
                } else {
                    Err(format!("Invalid task ID pattern: {}", self.id()))
                }
            },
            EntityType::Session => {
                if IdValidator::validate_session_id(self.id()) {
                    Ok(())
                } else {
                    Err(format!("Invalid session ID pattern: {}", self.id()))
                }
            },
            EntityType::Directive => {
                if IdValidator::validate_directive_id(self.id()) {
                    Ok(())
                } else {
                    Err(format!("Invalid directive ID pattern: {}", self.id()))
                }
            },
        }
    }
    
    /// Validate project reference
    fn validate_project_reference(&self) -> Result<(), String> {
        use crate::entities::schema_models::IdValidator;
        
        if IdValidator::validate_project_id(self.project_id()) {
            Ok(())
        } else {
            Err(format!("Invalid project ID reference: {}", self.project_id()))
        }
    }
}

/// Entity metadata trait for JSON metadata fields
pub trait MetadataEntity: Entity {
    /// Get metadata as JSON string
    fn metadata_json(&self) -> Option<&str> {
        None
    }
    
    /// Get parsed metadata
    fn get_metadata<T>(&self) -> Result<Option<T>, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.metadata_json() {
            Some(json) => {
                if json.trim().is_empty() {
                    Ok(None)
                } else {
                    serde_json::from_str(json)
                        .map(Some)
                        .map_err(|e| format!("Failed to parse metadata JSON: {}", e))
                }
            },
            None => Ok(None),
        }
    }
    
    /// Set metadata from serializable object
    fn set_metadata<T>(&mut self, _metadata: &T) -> Result<(), String>
    where
        T: Serialize,
    {
        // This is a default implementation that can be overridden
        Err("Metadata setting not implemented for this entity".to_string())
    }
}

/// Entity search and filtering trait
pub trait SearchableEntity: Entity {
    /// Search entity by text query
    fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        // Search in ID
        if self.id().to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Search in title
        if self.title().to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Search in description if available
        if let Some(desc) = self.description() {
            if desc.to_lowercase().contains(&query_lower) {
                return true;
            }
        }
        
        false
    }
    
    /// Get searchable content for full-text search
    fn searchable_content(&self) -> String {
        let mut content = vec![self.id(), self.title()];
        
        if let Some(desc) = self.description() {
            content.push(desc);
        }
        
        content.join(" ")
    }
    
    /// Get search keywords/tags
    fn search_keywords(&self) -> Vec<String> {
        vec![
            self.entity_type().as_str().to_string(),
            self.id().to_string(),
        ]
    }
}

/// Entity comparison and sorting trait
pub trait ComparableEntity: Entity {
    /// Compare entities for sorting by update time (most recent first)
    fn compare_by_update_time(&self, other: &dyn Entity) -> std::cmp::Ordering {
        other.updated_at().cmp(&self.updated_at()) // Reverse for most recent first
    }
    
    /// Compare entities for sorting by creation time (newest first)
    fn compare_by_creation_time(&self, other: &dyn Entity) -> std::cmp::Ordering {
        other.created_at().cmp(&self.created_at()) // Reverse for newest first
    }
    
    /// Compare entities for sorting by title (alphabetical)
    fn compare_by_title(&self, other: &dyn Entity) -> std::cmp::Ordering {
        self.title().cmp(other.title())
    }
    
    /// Compare entities for sorting by ID (alphabetical)
    fn compare_by_id(&self, other: &dyn Entity) -> std::cmp::Ordering {
        self.id().cmp(other.id())
    }
}

/// Entity collection management trait
pub trait EntityCollection<T: Entity> {
    /// Get all entities in collection
    fn all(&self) -> &[T];
    
    /// Find entity by ID
    fn find_by_id(&self, id: &str) -> Option<&T> {
        self.all().iter().find(|e| e.id() == id)
    }
    
    /// Filter entities by project
    fn filter_by_project(&self, project_id: &str) -> Vec<&T> {
        self.all()
            .iter()
            .filter(|e| e.project_id() == project_id)
            .collect()
    }
    
    /// Filter entities by type
    fn filter_by_type(&self, entity_type: EntityType) -> Vec<&T> {
        self.all()
            .iter()
            .filter(|e| e.entity_type() == entity_type)
            .collect()
    }
    
    /// Search entities
    fn search(&self, query: &str) -> Vec<&T>
    where
        T: SearchableEntity,
    {
        self.all()
            .iter()
            .filter(|e| e.matches_search(query))
            .collect()
    }
    
    /// Get recent entities (created in last 24 hours)
    fn recent(&self) -> Vec<&T> {
        self.all()
            .iter()
            .filter(|e| e.is_recent())
            .collect()
    }
    
    /// Get recently updated entities (updated in last hour)
    fn recently_updated(&self) -> Vec<&T> {
        self.all()
            .iter()
            .filter(|e| e.is_recently_updated())
            .collect()
    }
    
    /// Count entities
    fn count(&self) -> usize {
        self.all().len()
    }
    
    /// Check if collection is empty
    fn is_empty(&self) -> bool {
        self.all().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_conversion() {
        assert_eq!(EntityType::Project.as_str(), "project");
        assert_eq!(EntityType::Feature.as_str(), "feature");
        assert_eq!(EntityType::Task.as_str(), "task");
        
        assert_eq!(EntityType::from_str("project").unwrap(), EntityType::Project);
        assert_eq!(EntityType::from_str("feature").unwrap(), EntityType::Feature);
        assert_eq!(EntityType::from_str("task").unwrap(), EntityType::Task);
        
        assert!(EntityType::from_str("invalid").is_err());
    }

    #[test]
    fn test_entity_type_all() {
        let all_types = EntityType::all();
        assert!(all_types.contains(&EntityType::Project));
        assert!(all_types.contains(&EntityType::Feature));
        assert!(all_types.contains(&EntityType::Task));
        assert!(all_types.contains(&EntityType::Session));
        assert!(all_types.contains(&EntityType::Directive));
        assert_eq!(all_types.len(), 5);
    }

    #[test]
    fn test_priority_sort_order() {
        #[derive(Debug)]
        struct MockPriorityEntity {
            priority: String,
        }
        
        impl MockPriorityEntity {
            fn new(priority: &str) -> Self {
                MockPriorityEntity {
                    priority: priority.to_string(),
                }
            }
        }
        
        impl Entity for MockPriorityEntity {
            fn id(&self) -> &str { "test" }
            fn entity_type(&self) -> EntityType { EntityType::Task }
            fn created_at(&self) -> DateTime<Utc> { Utc::now() }
            fn updated_at(&self) -> DateTime<Utc> { Utc::now() }
            fn title(&self) -> &str { "test" }
            fn project_id(&self) -> &str { "P001" }
            fn as_any(&self) -> &dyn Any { self }
        }
        
        impl PrioritizedEntity for MockPriorityEntity {
            fn priority_level(&self) -> &str {
                &self.priority
            }
        }
        
        let critical = MockPriorityEntity::new("critical");
        let high = MockPriorityEntity::new("high");
        let medium = MockPriorityEntity::new("medium");
        let low = MockPriorityEntity::new("low");
        
        assert_eq!(critical.priority_sort_order(), 0);
        assert_eq!(high.priority_sort_order(), 1);
        assert_eq!(medium.priority_sort_order(), 2);
        assert_eq!(low.priority_sort_order(), 3);
        
        assert!(critical.is_high_priority());
        assert!(high.is_high_priority());
        assert!(!medium.is_high_priority());
        assert!(!low.is_high_priority());
    }
}
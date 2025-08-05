use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::models::*;

/// Operation request for constraint enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRequest {
    pub operation_type: String,     // create_feature, update_task, complete_feature, etc.
    pub entity_type: String,        // feature, task, session, etc.
    pub entity_id: Option<String>,  // ID of entity being operated on
    pub data: Option<String>,       // JSON data for the operation
    pub user_context: Option<String>, // Context about user intent
}

/// Validation result with detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub entity_type: String,
    pub entity_id: Option<String>,
}

/// Structured validation error with actionable feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_code: String,
    pub field: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
    pub severity: ValidationSeverity,
    pub rule_type: ValidationRuleType,
}

/// Validation warning for non-blocking issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_code: String,
    pub field: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Validation error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    Critical,    // Blocks operation completely
    Error,       // Blocks operation, but can be fixed
    Warning,     // Operation continues but should be addressed
    Info,        // Informational only
}

/// Types of validation rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationRuleType {
    DatabaseConstraint,      // DB-level validation
    BusinessRule,           // Business logic validation  
    MethodologyCompliance,  // Development methodology rules
    CrossEntity,           // Multi-entity validation
    DataIntegrity,         // Data consistency validation
    StateTransition,       // State machine validation
    MethodologyConstraint, // Methodology constraint violation
}

/// Entity validation framework
pub struct EntityValidator {
    rules: Vec<Box<dyn ValidationRule>>,
    cross_entity_rules: Vec<Box<dyn CrossEntityValidationRule>>,
}

/// Individual validation rule trait
pub trait ValidationRule: Send + Sync {
    fn rule_name(&self) -> &str;
    fn rule_type(&self) -> ValidationRuleType;
    fn validate(&self, entity: &dyn Entity, context: &ValidationContext) -> Result<Vec<ValidationError>>;
}

/// Cross-entity validation rule trait
pub trait CrossEntityValidationRule: Send + Sync {
    fn rule_name(&self) -> &str;
    fn validate(&self, entities: &[&dyn Entity], context: &ValidationContext) -> Result<Vec<ValidationError>>;
}

/// Validation context with project and entity information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationContext {
    pub project_id: String,
    pub operation: ValidationOperation,
    pub current_user: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Type of operation being validated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ValidationOperation {
    #[default]
    Create,
    Update,
    Delete,
    StateTransition,
    BulkOperation,
}

impl EntityValidator {
    /// Create new validator with default rules
    pub fn new() -> Self {
        let mut validator = EntityValidator {
            rules: Vec::new(),
            cross_entity_rules: Vec::new(),
        };
        
        validator.register_default_rules();
        validator
    }

    /// Register all default validation rules
    fn register_default_rules(&mut self) {
        // Feature validation rules
        self.register_rule(Box::new(FeatureCodeFormatRule));
        self.register_rule(Box::new(FeatureStateConsistencyRule));
        self.register_rule(Box::new(FeatureTestEvidenceRule));
        self.register_rule(Box::new(FeatureImplementationNotesRule));
        
        // Task validation rules
        self.register_rule(Box::new(TaskCodeFormatRule));
        self.register_rule(Box::new(TaskFeatureLinkageRule));
        self.register_rule(Box::new(TaskDateLogicRule));
        
        // Session validation rules
        self.register_rule(Box::new(SessionWorkflowRule));
        self.register_rule(Box::new(SessionContextRule));
        
        // Milestone validation rules
        self.register_rule(Box::new(MilestoneProgressRule));
        self.register_rule(Box::new(MilestoneFeatureLinkageRule));
        
        // Cross-entity validation rules
        self.register_cross_entity_rule(Box::new(ThreeAccessMethodRule));
        self.register_cross_entity_rule(Box::new(OrphanedEntityRule));
        self.register_cross_entity_rule(Box::new(CircularDependencyRule));
    }

    /// Register a validation rule
    pub fn register_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Register a cross-entity validation rule
    pub fn register_cross_entity_rule(&mut self, rule: Box<dyn CrossEntityValidationRule>) {
        self.cross_entity_rules.push(rule);
    }

    /// Validate a single entity
    pub fn validate_entity(&self, entity: &dyn Entity, context: &ValidationContext) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Run all validation rules
        for rule in &self.rules {
            match rule.validate(entity, context) {
                Ok(rule_errors) => errors.extend(rule_errors),
                Err(e) => {
                    errors.push(ValidationError {
                        error_code: "VALIDATION_RULE_ERROR".to_string(),
                        field: None,
                        message: format!("Validation rule '{}' failed: {}", rule.rule_name(), e),
                        suggestion: Some("Check validation rule implementation".to_string()),
                        severity: ValidationSeverity::Error,
                        rule_type: ValidationRuleType::DataIntegrity,
                    });
                }
            }
        }

        // Convert some errors to warnings based on severity
        let (critical_errors, warning_errors): (Vec<_>, Vec<_>) = errors.into_iter()
            .partition(|e| e.severity == ValidationSeverity::Critical || e.severity == ValidationSeverity::Error);

        warnings.extend(warning_errors.into_iter().map(|e| ValidationWarning {
            warning_code: e.error_code,
            field: e.field,
            message: e.message,
            suggestion: e.suggestion,
        }));

        ValidationResult {
            is_valid: critical_errors.is_empty(),
            errors: critical_errors,
            warnings,
            entity_type: entity.entity_type().to_string(),
            entity_id: Some(entity.id().to_string()),
        }
    }

    /// Validate multiple entities with cross-entity rules
    pub fn validate_entities(&self, entities: &[&dyn Entity], context: &ValidationContext) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Validate each entity individually
        for entity in entities {
            results.push(self.validate_entity(*entity, context));
        }

        // Run cross-entity validation rules
        let mut cross_entity_errors = Vec::new();
        for rule in &self.cross_entity_rules {
            match rule.validate(entities, context) {
                Ok(rule_errors) => cross_entity_errors.extend(rule_errors),
                Err(e) => {
                    cross_entity_errors.push(ValidationError {
                        error_code: "CROSS_ENTITY_VALIDATION_ERROR".to_string(),
                        field: None,
                        message: format!("Cross-entity rule '{}' failed: {}", rule.rule_name(), e),
                        suggestion: Some("Check cross-entity validation rule implementation".to_string()),
                        severity: ValidationSeverity::Error,
                        rule_type: ValidationRuleType::CrossEntity,
                    });
                }
            }
        }

        // Add cross-entity errors to results
        if !cross_entity_errors.is_empty() {
            results.push(ValidationResult {
                is_valid: false,
                errors: cross_entity_errors,
                warnings: Vec::new(),
                entity_type: "CrossEntity".to_string(),
                entity_id: None,
            });
        }

        results
    }

    /// Enforce constraints before entity operations - F0130 implementation
    pub fn enforce_constraints(&self, operation: &OperationRequest, context: &ValidationContext) -> Result<()> {
        let validation_result = self.validate_operation(operation, context);
        
        if !validation_result.is_valid {
            let error_messages: Vec<String> = validation_result.errors
                .into_iter()
                .map(|e| format!("{}: {}", e.error_code, e.message))
                .collect();
            
            return Err(anyhow::anyhow!(
                "Operation blocked by validation constraints:\n{}",
                error_messages.join("\n")
            ));
        }
        
        Ok(())
    }

    /// Validate operation request before execution
    pub fn validate_operation(&self, operation: &OperationRequest, context: &ValidationContext) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check methodology compliance
        if let Err(methodology_errors) = self.check_methodology_compliance(operation, context) {
            errors.extend(methodology_errors);
        }

        // Check directive compliance
        if let Err(directive_errors) = self.check_directive_compliance(operation, context) {
            errors.extend(directive_errors);
        }

        // Check three-access-method rule for feature operations
        if operation.operation_type == "create_feature" || operation.operation_type == "update_feature" {
            if let Err(access_errors) = self.check_three_access_method_compliance(operation, context) {
                errors.extend(access_errors);
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            entity_type: operation.entity_type.clone(),
            entity_id: operation.entity_id.clone(),
        }
    }

    /// Check methodology compliance from directives.md rules
    fn check_methodology_compliance(&self, operation: &OperationRequest, _context: &ValidationContext) -> Result<Vec<ValidationError>, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // DIR-20250804-001: Three access method enforcement
        if operation.operation_type == "complete_feature" {
            if let Some(data) = &operation.data {
                if let Ok(feature_data) = serde_json::from_str::<serde_json::Value>(data) {
                    let has_cli = feature_data.get("has_cli_command").and_then(|v| v.as_bool()).unwrap_or(false);
                    let has_web_ui = feature_data.get("has_web_ui").and_then(|v| v.as_bool()).unwrap_or(false);
                    let has_mcp = feature_data.get("has_mcp_tool").and_then(|v| v.as_bool()).unwrap_or(false);

                    if !has_cli || !has_web_ui || !has_mcp {
                        errors.push(ValidationError {
                            error_code: "DIR_20250804_001_VIOLATION".to_string(),
                            field: Some("access_methods".to_string()),
                            message: "Feature completion blocked: missing required access methods".to_string(),
                            suggestion: Some(format!(
                                "Implement missing access methods: CLI({}), Web UI({}), MCP({})",
                                if has_cli { "✅" } else { "❌" },
                                if has_web_ui { "✅" } else { "❌" },
                                if has_mcp { "✅" } else { "❌" }
                            )),
                            severity: ValidationSeverity::Critical,
                            rule_type: ValidationRuleType::MethodologyConstraint,
                        });
                    }
                }
            }
        }

        if errors.is_empty() { Ok(Vec::new()) } else { Err(errors) }
    }

    /// Check directive compliance from database
    fn check_directive_compliance(&self, _operation: &OperationRequest, _context: &ValidationContext) -> Result<Vec<ValidationError>, Vec<ValidationError>> {
        // TODO: Query database for active directives and validate against them
        Ok(Vec::new())
    }

    /// Check three access method compliance for features
    fn check_three_access_method_compliance(&self, _operation: &OperationRequest, _context: &ValidationContext) -> Result<Vec<ValidationError>, Vec<ValidationError>> {
        // TODO: Validate that features have CLI, Web UI, and MCP implementations
        Ok(Vec::new())
    }
}

// Feature validation rules
struct FeatureCodeFormatRule;
impl ValidationRule for FeatureCodeFormatRule {
    fn rule_name(&self) -> &str { "feature_code_format" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::BusinessRule }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        if entity.entity_type() != EntityType::Feature {
            return Ok(Vec::new());
        }

        if let Some(feature) = entity.as_any().downcast_ref::<Feature>() {
            let mut errors = Vec::new();
            
            // Validate F0001 format
            if !feature.code.starts_with('F') || feature.code.len() != 5 {
                errors.push(ValidationError {
                    error_code: "INVALID_FEATURE_CODE".to_string(),
                    field: Some("code".to_string()),
                    message: format!("Feature code '{}' must follow F0001 format", feature.code),
                    suggestion: Some("Use format F followed by 4 digits (e.g., F0001, F0002)".to_string()),
                    severity: ValidationSeverity::Error,
                    rule_type: ValidationRuleType::BusinessRule,
                });
            }

            // Validate code is numeric after F
            if let Some(number_part) = feature.code.strip_prefix('F') {
                if number_part.parse::<u32>().is_err() {
                    errors.push(ValidationError {
                        error_code: "INVALID_FEATURE_CODE_FORMAT".to_string(),
                        field: Some("code".to_string()),
                        message: format!("Feature code '{}' must have numeric suffix", feature.code),
                        suggestion: Some("Ensure the part after 'F' is a 4-digit number".to_string()),
                        severity: ValidationSeverity::Error,
                        rule_type: ValidationRuleType::BusinessRule,
                    });
                }
            }

            Ok(errors)
        } else {
            Ok(Vec::new())
        }
    }
}

struct FeatureStateConsistencyRule;
impl ValidationRule for FeatureStateConsistencyRule {
    fn rule_name(&self) -> &str { "feature_state_consistency" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::MethodologyCompliance }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        if entity.entity_type() != EntityType::Feature {
            return Ok(Vec::new());
        }

        if let Some(feature) = entity.as_any().downcast_ref::<Feature>() {
            let mut errors = Vec::new();

            // TestedPassing requires test evidence
            if feature.state == FeatureState::TestedPassing && feature.test_evidence.is_none() {
                errors.push(ValidationError {
                    error_code: "MISSING_TEST_EVIDENCE".to_string(),
                    field: Some("test_evidence".to_string()),
                    message: "Features in TestedPassing state must have test evidence".to_string(),
                    suggestion: Some("Add test evidence before marking as TestedPassing".to_string()),
                    severity: ValidationSeverity::Error,
                    rule_type: ValidationRuleType::MethodologyCompliance,
                });
            }

            // Implemented state requires implementation notes
            if feature.state == FeatureState::Implemented && feature.implementation_notes.is_none() {
                errors.push(ValidationError {
                    error_code: "MISSING_IMPLEMENTATION_NOTES".to_string(),
                    field: Some("implementation_notes".to_string()),
                    message: "Features in Implemented state should have implementation notes".to_string(),
                    suggestion: Some("Add implementation notes describing the feature implementation".to_string()),
                    severity: ValidationSeverity::Warning,
                    rule_type: ValidationRuleType::MethodologyCompliance,
                });
            }

            Ok(errors)
        } else {
            Ok(Vec::new())
        }
    }
}

// Additional validation rules would be implemented here...
// TaskCodeFormatRule, SessionWorkflowRule, etc.

struct FeatureTestEvidenceRule;
impl ValidationRule for FeatureTestEvidenceRule {
    fn rule_name(&self) -> &str { "feature_test_evidence" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::MethodologyCompliance }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for test evidence validation
        Ok(Vec::new())
    }
}

struct FeatureImplementationNotesRule;
impl ValidationRule for FeatureImplementationNotesRule {
    fn rule_name(&self) -> &str { "feature_implementation_notes" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::MethodologyCompliance }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for implementation notes validation
        Ok(Vec::new())
    }
}

struct TaskCodeFormatRule;
impl ValidationRule for TaskCodeFormatRule {
    fn rule_name(&self) -> &str { "task_code_format" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::BusinessRule }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for task code format validation
        Ok(Vec::new())
    }
}

struct TaskFeatureLinkageRule;
impl ValidationRule for TaskFeatureLinkageRule {
    fn rule_name(&self) -> &str { "task_feature_linkage" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::CrossEntity }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for task-feature linkage validation
        Ok(Vec::new())
    }
}

struct TaskDateLogicRule;
impl ValidationRule for TaskDateLogicRule {
    fn rule_name(&self) -> &str { "task_date_logic" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::BusinessRule }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for task date logic validation
        Ok(Vec::new())
    }
}

struct SessionWorkflowRule;
impl ValidationRule for SessionWorkflowRule {
    fn rule_name(&self) -> &str { "session_workflow" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::MethodologyCompliance }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for session workflow validation
        Ok(Vec::new())
    }
}

struct SessionContextRule;
impl ValidationRule for SessionContextRule {
    fn rule_name(&self) -> &str { "session_context" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::MethodologyCompliance }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for session context validation
        Ok(Vec::new())
    }
}

struct MilestoneProgressRule;
impl ValidationRule for MilestoneProgressRule {
    fn rule_name(&self) -> &str { "milestone_progress" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::BusinessRule }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for milestone progress validation
        Ok(Vec::new())
    }
}

struct MilestoneFeatureLinkageRule;
impl ValidationRule for MilestoneFeatureLinkageRule {
    fn rule_name(&self) -> &str { "milestone_feature_linkage" }
    fn rule_type(&self) -> ValidationRuleType { ValidationRuleType::CrossEntity }
    
    fn validate(&self, entity: &dyn Entity, _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for milestone-feature linkage validation
        Ok(Vec::new())
    }
}

// Cross-entity validation rules
struct ThreeAccessMethodRule;
impl CrossEntityValidationRule for ThreeAccessMethodRule {
    fn rule_name(&self) -> &str { "three_access_method" }
    
    fn validate(&self, entities: &[&dyn Entity], _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for three-access-method validation
        Ok(Vec::new())
    }
}

struct OrphanedEntityRule;
impl CrossEntityValidationRule for OrphanedEntityRule {
    fn rule_name(&self) -> &str { "orphaned_entity" }
    
    fn validate(&self, entities: &[&dyn Entity], _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for orphaned entity detection
        Ok(Vec::new())
    }
}

struct CircularDependencyRule;
impl CrossEntityValidationRule for CircularDependencyRule {
    fn rule_name(&self) -> &str { "circular_dependency" }
    
    fn validate(&self, entities: &[&dyn Entity], _context: &ValidationContext) -> Result<Vec<ValidationError>> {
        // Implementation for circular dependency detection
        Ok(Vec::new())
    }
}

impl Default for EntityValidator {
    fn default() -> Self {
        Self::new()
    }
}
// JavaScript/TypeScript Model Definitions
// These models match the Rust backend models for type safety and API consistency

/**
 * @typedef {Object} Feature
 * @property {string} id - UUID identifier
 * @property {string} project_id - Project UUID
 * @property {string} code - Feature code (e.g., F0001)
 * @property {string} name - Feature name/title
 * @property {string} description - Feature description
 * @property {string|null} category - Feature category
 * @property {FeatureState} state - Implementation state
 * @property {TestStatus} test_status - Testing status
 * @property {Priority} priority - Feature priority
 * @property {string|null} implementation_notes - Implementation notes
 * @property {string|null} test_evidence - Test evidence/documentation
 * @property {string|null} dependencies - Dependencies (JSON string)
 * @property {string} created_at - ISO timestamp
 * @property {string} updated_at - ISO timestamp
 * @property {string|null} completed_at - Completion timestamp
 * @property {number|null} estimated_effort - Estimated effort in hours
 * @property {number|null} actual_effort - Actual effort in hours
 */

/**
 * @typedef {Object} Task
 * @property {string} id - UUID identifier
 * @property {string} project_id - Project UUID
 * @property {string} code - Task code (e.g., TASK-001)
 * @property {string} title - Task title
 * @property {string} description - Task description
 * @property {string} category - Task category
 * @property {TaskStatus} status - Task status
 * @property {Priority} priority - Task priority
 * @property {string|null} feature_ids - Related feature IDs (JSON string)
 * @property {string|null} depends_on - Task dependencies (JSON string)
 * @property {string|null} acceptance_criteria - Acceptance criteria
 * @property {string|null} validation_steps - Validation steps
 * @property {string|null} evidence - Task completion evidence
 * @property {string|null} session_id - Associated session ID
 * @property {string|null} assigned_to - Assigned user
 * @property {string} created_at - ISO timestamp
 * @property {string} updated_at - ISO timestamp
 * @property {string|null} started_at - Start timestamp
 * @property {string|null} completed_at - Completion timestamp
 * @property {number|null} estimated_effort - Estimated effort in hours
 * @property {number|null} actual_effort - Actual effort in hours
 * @property {string|null} tags - Task tags (JSON string)
 */

/**
 * @typedef {Object} Project
 * @property {string} id - UUID identifier
 * @property {string} name - Project name
 * @property {string|null} description - Project description
 * @property {string|null} repository_url - Repository URL
 * @property {string} version - Project version
 * @property {string} created_at - ISO timestamp
 * @property {string} updated_at - ISO timestamp
 * @property {boolean} archived - Whether project is archived
 * @property {string|null} metadata - Project metadata (JSON string)
 */

/**
 * @typedef {'NotImplemented'|'Planned'|'InProgress'|'Implemented'|'Tested'|'Deprecated'} FeatureState
 */

/**
 * @typedef {'NotTested'|'InProgress'|'Passed'|'Failed'|'Skipped'} TestStatus
 */

/**
 * @typedef {'Pending'|'InProgress'|'Completed'|'Blocked'|'Cancelled'} TaskStatus
 */

/**
 * @typedef {'Low'|'Medium'|'High'|'Critical'} Priority
 */

/**
 * Model validation and serialization utilities
 */
class ModelValidator {
    /**
     * Validate Feature object against schema
     * @param {any} obj - Object to validate
     * @returns {boolean} - Whether object is valid Feature
     */
    static isValidFeature(obj) {
        return obj && 
               typeof obj.id === 'string' &&
               typeof obj.project_id === 'string' &&
               typeof obj.code === 'string' &&
               typeof obj.name === 'string' &&
               typeof obj.description === 'string' &&
               ['NotImplemented', 'Planned', 'InProgress', 'Implemented', 'Tested', 'Deprecated'].includes(obj.state) &&
               ['NotTested', 'InProgress', 'Passed', 'Failed', 'Skipped'].includes(obj.test_status) &&
               ['Low', 'Medium', 'High', 'Critical'].includes(obj.priority);
    }

    /**
     * Validate Task object against schema
     * @param {any} obj - Object to validate
     * @returns {boolean} - Whether object is valid Task
     */
    static isValidTask(obj) {
        return obj && 
               typeof obj.id === 'string' &&
               typeof obj.project_id === 'string' &&
               typeof obj.code === 'string' &&
               typeof obj.title === 'string' &&
               typeof obj.description === 'string' &&
               ['Pending', 'InProgress', 'Completed', 'Blocked', 'Cancelled'].includes(obj.status) &&
               ['Low', 'Medium', 'High', 'Critical'].includes(obj.priority);
    }

    /**
     * Validate Project object against schema
     * @param {any} obj - Object to validate
     * @returns {boolean} - Whether object is valid Project
     */
    static isValidProject(obj) {
        return obj && 
               typeof obj.id === 'string' &&
               typeof obj.name === 'string' &&
               typeof obj.version === 'string' &&
               typeof obj.archived === 'boolean';
    }

    /**
     * Convert Rust-style enum values to display names
     * @param {string} value - Enum value
     * @returns {string} - Display name
     */
    static formatEnumValue(value) {
        const formatMap = {
            // Feature states
            'NotImplemented': 'Not Implemented',
            'InProgress': 'In Progress',
            'Implemented': 'Implemented',
            'Tested': 'Tested',
            'Planned': 'Planned',
            'Deprecated': 'Deprecated',
            
            // Test statuses
            'NotTested': 'Not Tested',
            'Passed': 'Passed',
            'Failed': 'Failed',
            'Skipped': 'Skipped',
            
            // Task statuses
            'Pending': 'Pending',
            'Completed': 'Completed',
            'Blocked': 'Blocked',
            'Cancelled': 'Cancelled',
            
            // Priorities
            'Low': 'Low',
            'Medium': 'Medium',
            'High': 'High',
            'Critical': 'Critical'
        };
        
        return formatMap[value] || value;
    }

    /**
     * Convert display names back to Rust-style enum values
     * @param {string} displayName - Display name
     * @returns {string} - Enum value
     */
    static parseEnumValue(displayName) {
        const parseMap = {
            'Not Implemented': 'NotImplemented',
            'In Progress': 'InProgress',
            'Not Tested': 'NotTested'
        };
        
        return parseMap[displayName] || displayName.replace(/\s+/g, '');
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        ModelValidator
    };
}
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::analyzer::SchemaChange;

/// Represents compatibility analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    /// List of detected changes
    pub changes: Vec<SchemaChange>,
    /// Overall compatibility score (0-100)
    pub compatibility_score: u8,
    /// Whether the schema is compatible
    pub is_compatible: bool,
    /// List of compatibility issues
    pub issues: Vec<CompatibilityIssue>,
    /// Additional metadata about the comparison
    pub metadata: HashMap<String, String>,
}

/// Represents a specific compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    /// Severity level of the issue
    pub severity: IssueSeverity,
    /// Description of the issue
    pub description: String,
    /// Location of the affected element
    pub location: String,
}

/// Represents the severity of a compatibility issue
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Breaking changes that must be addressed
    Error,
    /// Potentially problematic changes that should be reviewed
    Warning,
    /// Informational changes that are generally safe
    Info,
}

/// Represents validation results for schema changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// Additional validation context
    pub context: HashMap<String, String>,
}

/// Represents a validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Path to the invalid element
    pub path: String,
    /// Error code for programmatic handling
    pub code: String,
}

/// Represents a migration plan between schema versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// List of migration steps
    pub steps: Vec<String>,
    /// Additional metadata about the migration
    pub metadata: HashMap<String, String>,
} 
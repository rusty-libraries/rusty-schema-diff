//! Core analyzer traits and types for schema comparison
//!
//! This module provides the base traits and types used by all schema analyzers.

use crate::{Schema, CompatibilityReport, MigrationPlan, ValidationResult, error::Result};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub mod json_schema;
pub mod protobuf;
pub mod openapi;
pub mod sql;

/// Core trait for implementing schema analyzers
///
/// This trait defines the required functionality for analyzing schema changes
/// and generating compatibility reports and migration paths.
pub trait SchemaAnalyzer {
    /// Analyzes compatibility between two schema versions
    ///
    /// # Arguments
    /// * `old` - The original schema version
    /// * `new` - The new schema version to compare against
    ///
    /// # Returns
    /// A compatibility report detailing the differences and compatibility status
    fn analyze_compatibility(&self, old: &Schema, new: &Schema) -> Result<CompatibilityReport>;

    /// Generates a migration path between schema versions
    ///
    /// # Arguments
    /// * `old` - The source schema version
    /// * `new` - The target schema version
    ///
    /// # Returns
    /// A migration plan detailing the required changes
    fn generate_migration_path(&self, old: &Schema, new: &Schema) -> Result<MigrationPlan>;

    /// Validates proposed schema changes
    ///
    /// # Arguments
    /// * `changes` - List of proposed schema changes to validate
    ///
    /// # Returns
    /// Validation results indicating if the changes are safe
    fn validate_changes(&self, changes: &[SchemaChange]) -> Result<ValidationResult>;
}

/// Represents a single schema change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// Type of change (Addition, Removal, etc.)
    pub change_type: ChangeType,
    /// Path to the changed element
    pub location: String,
    /// Human-readable description of the change
    pub description: String,
    /// Metadata associated with the change
    pub metadata: HashMap<String, String>,
}

impl SchemaChange {
    /// Creates a new SchemaChange instance
    ///
    /// # Arguments
    /// * `change_type` - The type of change
    /// * `location` - The location of the change
    /// * `description` - The description of the change
    /// * `metadata` - The metadata associated with the change
    ///
    /// # Returns
    /// A new SchemaChange instance
    pub fn new(
        change_type: ChangeType,
        location: impl Into<String>,
        description: impl Into<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        SchemaChange {
            change_type,
            location: location.into(),
            description: description.into(),
            metadata,
        }
    }
}

/// Types of schema changes that can occur
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeType {
    /// New element added
    Addition,
    /// Existing element removed
    Removal,
    /// Element modified
    Modification,
    /// Element renamed
    Rename,
} 
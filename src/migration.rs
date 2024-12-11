//! Migration plan generation and management
//!
//! This module provides types and functionality for generating and managing
//! schema migration plans.

use serde::{Serialize, Deserialize};
use crate::analyzer::SchemaChange;

/// Represents a plan for migrating between schema versions
///
/// A migration plan contains all the necessary changes required to evolve
/// from one schema version to another, along with impact analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Version of the source schema
    pub source_version: String,
    
    /// Version of the target schema
    pub target_version: String,
    
    /// List of changes required for migration
    pub changes: Vec<SchemaChange>,
    
    /// Impact score (0-100) indicating the magnitude of changes
    pub impact_score: u8,
    
    /// Indicates whether this migration contains breaking changes
    pub is_breaking: bool,
}

impl MigrationPlan {
    /// Creates a new migration plan
    ///
    /// # Arguments
    /// * `source_version` - Version identifier of the source schema
    /// * `target_version` - Version identifier of the target schema
    /// * `changes` - List of schema changes required for migration
    ///
    /// # Returns
    /// A new MigrationPlan instance with calculated impact scores
    pub fn new(source_version: String, target_version: String, changes: Vec<SchemaChange>) -> Self {
        let impact_score = Self::calculate_impact(&changes);
        let is_breaking = Self::detect_breaking_changes(&changes);

        Self {
            source_version,
            target_version,
            changes,
            impact_score,
            is_breaking,
        }
    }

    /// Calculates the impact score of the migration
    ///
    /// # Arguments
    /// * `changes` - List of schema changes to analyze
    ///
    /// # Returns
    /// An impact score between 0 and 100
    fn calculate_impact(changes: &[SchemaChange]) -> u8 {
        // Simple scoring algorithm
        let score = changes.iter().map(|change| match change.change_type {
            crate::analyzer::ChangeType::Addition => 25,
            crate::analyzer::ChangeType::Removal => 100,
            crate::analyzer::ChangeType::Modification => 50,
            crate::analyzer::ChangeType::Rename => 30,
        }).max().unwrap_or(0);

        score.min(100) as u8
    }

    /// Detects if the migration contains breaking changes
    ///
    /// # Arguments
    /// * `changes` - List of schema changes to analyze
    ///
    /// # Returns
    /// true if breaking changes are detected, false otherwise
    fn detect_breaking_changes(changes: &[SchemaChange]) -> bool {
        changes.iter().any(|change| matches!(
            change.change_type,
            crate::analyzer::ChangeType::Removal | crate::analyzer::ChangeType::Modification
        ))
    }

    /// Gets a list of breaking changes in the migration plan
    ///
    /// # Returns
    /// A vector of references to breaking changes
    pub fn breaking_changes(&self) -> Vec<&SchemaChange> {
        self.changes.iter()
            .filter(|change| matches!(
                change.change_type,
                crate::analyzer::ChangeType::Removal | crate::analyzer::ChangeType::Modification
            ))
            .collect()
    }
} 
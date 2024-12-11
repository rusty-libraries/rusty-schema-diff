//! Protobuf specific analyzer implementation
//!
//! This module provides functionality for analyzing Protobuf changes and
//! generating compatibility reports and migration paths.

use protobuf::descriptor::{FileDescriptorProto, DescriptorProto};
use crate::analyzer::{SchemaAnalyzer, SchemaChange, ChangeType};
use crate::{Schema, CompatibilityReport, MigrationPlan, ValidationResult, SchemaDiffError};
use crate::error::Result;
use crate::report::{CompatibilityIssue, IssueSeverity, ValidationError};
use std::collections::HashMap;

/// Analyzes Protobuf changes and generates compatibility reports.
pub struct ProtobufAnalyzer;

impl SchemaAnalyzer for ProtobufAnalyzer {
    /// Analyzes compatibility between two Protobuf versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The original Protobuf version.
    /// * `new` - The new Protobuf version to compare against.
    ///
    /// # Returns
    ///
    /// A `CompatibilityReport` detailing the differences and compatibility status.
    fn analyze_compatibility(&self, old: &Schema, new: &Schema) -> Result<CompatibilityReport> {
        let old_desc = self.parse_proto(&old.content)?;
        let new_desc = self.parse_proto(&new.content)?;

        let mut changes = Vec::new();
        self.compare_descriptors(&old_desc, &new_desc, "", &mut changes)?;

        let compatibility_score = self.calculate_compatibility_score(&changes);
        let is_compatible = compatibility_score >= 80;

        Ok(CompatibilityReport {
            compatibility_score: compatibility_score.try_into().unwrap(),
            is_compatible,
            changes: changes,
            issues: vec![],
            metadata: Default::default(),
        })
    }

    /// Generates a migration path between Protobuf versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The source Protobuf version.
    /// * `new` - The target Protobuf version.
    ///
    /// # Returns
    ///
    /// A `MigrationPlan` detailing the required changes.
    fn generate_migration_path(&self, old: &Schema, new: &Schema) -> Result<MigrationPlan> {
        let old_desc = self.parse_proto(&old.content)?;
        let new_desc = self.parse_proto(&new.content)?;

        let mut changes = Vec::new();
        self.compare_descriptors(&old_desc, &new_desc, "", &mut changes)?;

        Ok(MigrationPlan::new(
            old.version.to_string(),
            new.version.to_string(),
            changes,
        ))
    }

    fn validate_changes(&self, changes: &[SchemaChange]) -> Result<ValidationResult> {
        let errors: Vec<ValidationError> = changes
            .iter()
            .filter_map(|change| {
                self.validate_change(change).map(|issue| ValidationError {
                    message: issue.description,
                    path: issue.location,
                    code: format!("PROTO{}", match issue.severity {
                        IssueSeverity::Error => "001",
                        IssueSeverity::Warning => "002",
                        IssueSeverity::Info => "003",
                    }),
                })
            })
            .collect();

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            context: Default::default(),
        })
    }
}

impl ProtobufAnalyzer {
    /// Parses protobuf content into a FileDescriptorProto
    fn parse_proto(&self, content: &str) -> Result<FileDescriptorProto> {
        // Basic implementation using protobuf parser
        match protobuf::text_format::parse_from_str(content) {
            Ok(desc) => Ok(desc),
            Err(e) => Err(SchemaDiffError::ProtobufError(e.to_string()))
        }
    }

    /// Compares two protobuf descriptors
    fn compare_descriptors(
        &self,
        old: &FileDescriptorProto,
        new: &FileDescriptorProto,
        path: &str,
        changes: &mut Vec<SchemaChange>,
    ) -> Result<()> {
        // Compare messages
        for old_msg in &old.message_type {
            if let Some(new_msg) = new.message_type.iter().find(|m| m.name() == old_msg.name()) {
                self.compare_messages(old_msg, new_msg, path, changes)?;
            } else {
                changes.push(SchemaChange {
                    change_type: ChangeType::Removal,
                    location: format!("{}/{}", path, old_msg.name()),
                    description: format!("Message '{}' was removed", old_msg.name()),
                    metadata: Default::default(),
                });
            }
        }

        // Check for new messages
        for new_msg in &new.message_type {
            if !old.message_type.iter().any(|m| m.name() == new_msg.name()) {
                changes.push(SchemaChange {
                    change_type: ChangeType::Addition,
                    location: format!("{}/{}", path, new_msg.name()),
                    description: format!("Message '{}' was added", new_msg.name()),
                    metadata: Default::default(),
                });
            }
        }

        Ok(())
    }

    /// Compares two protobuf messages
    fn compare_messages(
        &self,
        old_msg: &DescriptorProto,
        new_msg: &DescriptorProto,
        path: &str,
        changes: &mut Vec<SchemaChange>,
    ) -> Result<()> {
        self.compare_fields(path, old_msg, new_msg, changes);
        Ok(())
    }

    fn compare_fields(
        &self,
        path: &str,
        old_msg: &DescriptorProto,
        new_msg: &DescriptorProto,
        changes: &mut Vec<SchemaChange>,
    ) {
        for old_field in old_msg.field.iter() {
            if let Some(new_field) = new_msg.field.iter().find(|f| f.name() == old_field.name()) {
                if old_field.type_() != new_field.type_() {
                    let mut metadata = HashMap::new();
                    metadata.insert("message".to_string(), old_msg.name().to_string());
                    metadata.insert("field".to_string(), old_field.name().to_string());
                    metadata.insert("old_type".to_string(), format!("{:?}", old_field.type_()));
                    metadata.insert("new_type".to_string(), format!("{:?}", new_field.type_()));
                    
                    changes.push(SchemaChange::new(
                        ChangeType::Modification,
                        format!("{}/{}/{}", path, old_msg.name(), old_field.name()),
                        format!(
                            "Field '{}' type changed from {:?} to {:?}",
                            old_field.name(),
                            old_field.type_(),
                            new_field.type_()
                        ),
                        metadata,
                    ));
                }
            } else {
                let mut metadata = HashMap::new();
                metadata.insert("message".to_string(), old_msg.name().to_string());
                metadata.insert("field".to_string(), old_field.name().to_string());
                
                changes.push(SchemaChange::new(
                    ChangeType::Removal,
                    format!("{}/{}/{}", path, old_msg.name(), old_field.name()),
                    format!("Field '{}' was removed", old_field.name()),
                    metadata,
                ));
            }
        }

        // Check for new fields
        for new_field in new_msg.field.iter() {
            if !old_msg.field.iter().any(|f| f.name() == new_field.name()) {
                let mut metadata = HashMap::new();
                metadata.insert("message".to_string(), new_msg.name().to_string());
                metadata.insert("field".to_string(), new_field.name().to_string());
                
                changes.push(SchemaChange::new(
                    ChangeType::Addition,
                    format!("{}/{}/{}", path, new_msg.name(), new_field.name()),
                    format!("New field '{}' was added", new_field.name()),
                    metadata,
                ));
            }
        }
    }

    /// Validates a single schema change
    fn validate_change(&self, change: &SchemaChange) -> Option<CompatibilityIssue> {
        match change.change_type {
            ChangeType::Removal => Some(CompatibilityIssue {
                severity: IssueSeverity::Error,
                description: format!("Breaking change: {}", change.description),
                location: change.location.clone(),
            }),
            ChangeType::Modification => Some(CompatibilityIssue {
                severity: IssueSeverity::Warning,
                description: format!("Potential compatibility issue: {}", change.description),
                location: change.location.clone(),
            }),
            ChangeType::Rename => {
                todo!("Implement handling for Rename change type");
            },
            _ => None,
        }
    }

    /// Calculates compatibility score for protobuf changes
    fn calculate_compatibility_score(&self, changes: &[SchemaChange]) -> i32 {
        let base_score: i32 = 100;
        let mut deductions: i32 = 0;
        
        for change in changes {
            match change.change_type {
                ChangeType::Addition => (),
                ChangeType::Removal => deductions += 20,
                ChangeType::Modification => deductions += 10,
                ChangeType::Rename => {
                    todo!("Implement handling for Rename change type");
                },
            }
        }
        
        base_score.saturating_sub(deductions)
    }
} 
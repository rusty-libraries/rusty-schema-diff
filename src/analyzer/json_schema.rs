//! JSON Schema specific analyzer implementation
//!
//! This module provides functionality for analyzing JSON Schema changes and
//! generating compatibility reports and migration paths.

use crate::analyzer::{SchemaAnalyzer, SchemaChange, ChangeType};
use crate::{Schema, CompatibilityReport, MigrationPlan, ValidationResult};
use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Analyzes JSON Schema changes and generates compatibility reports.
pub struct JsonSchemaAnalyzer;

impl SchemaAnalyzer for JsonSchemaAnalyzer {
    /// Analyzes compatibility between two JSON Schema versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The original JSON Schema version.
    /// * `new` - The new JSON Schema version to compare against.
    ///
    /// # Returns
    ///
    /// A `CompatibilityReport` detailing the differences and compatibility status.
    fn analyze_compatibility(&self, old: &Schema, new: &Schema) -> Result<CompatibilityReport> {
        let old_schema: Value = serde_json::from_str(&old.content)?;
        let new_schema: Value = serde_json::from_str(&new.content)?;

        let mut changes = Vec::new();
        self.compare_schemas(&old_schema, &new_schema, "", &mut changes);

        let compatibility_score = self.calculate_compatibility_score(&changes);
        let is_compatible = compatibility_score >= 80;

        Ok(CompatibilityReport {
            changes,
            compatibility_score,
            is_compatible,
            issues: vec![],  // TODO: Implement issue detection
            metadata: Default::default(),
        })
    }

    /// Generates a migration path between JSON Schema versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The source JSON Schema version.
    /// * `new` - The target JSON Schema version.
    ///
    /// # Returns
    ///
    /// A `MigrationPlan` detailing the required changes.
    fn generate_migration_path(&self, old: &Schema, new: &Schema) -> Result<MigrationPlan> {
        let mut changes = Vec::new();
        let old_schema: Value = serde_json::from_str(&old.content)?;
        let new_schema: Value = serde_json::from_str(&new.content)?;

        self.compare_schemas(&old_schema, &new_schema, "", &mut changes);

        Ok(MigrationPlan::new(
            old.version.to_string(),
            new.version.to_string(),
            changes,
        ))
    }

    fn validate_changes(&self, _changes: &[SchemaChange]) -> Result<ValidationResult> {
        Ok(ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            context: HashMap::new(),
        })
    }
}

impl JsonSchemaAnalyzer {
    /// Compares two JSON schemas and collects changes
    fn compare_schemas(&self, old: &Value, new: &Value, path: &str, changes: &mut Vec<SchemaChange>) {
        match (old, new) {
            (Value::Object(old_obj), Value::Object(new_obj)) => {
                self.compare_objects(old_obj, new_obj, path, changes);
            }
            (Value::Array(old_arr), Value::Array(new_arr)) => {
                self.compare_arrays(old_arr, new_arr, path, changes);
            }
            _ if old != new => {
                let mut metadata = HashMap::new();
                metadata.insert("old_value".to_string(), old.to_string());
                metadata.insert("new_value".to_string(), new.to_string());
                
                changes.push(SchemaChange::new(
                    ChangeType::Modification,
                    path.to_string(),
                    format!("Value changed from {:?} to {:?}", old, new),
                    metadata,
                ));
            }
            _ => {}
        }
    }

    fn calculate_compatibility_score(&self, changes: &[SchemaChange]) -> u8 {
        let base_score: u8 = 100;
        let mut deductions: u8 = 0;
        
        for change in changes {
            match change.change_type {
                ChangeType::Addition => deductions = deductions.saturating_add(5),
                ChangeType::Removal => deductions = deductions.saturating_add(20),
                ChangeType::Modification => deductions = deductions.saturating_add(10),
                ChangeType::Rename => deductions = deductions.saturating_add(8),
            }
        }
        
        base_score.saturating_sub(deductions)
    }

    #[allow(dead_code)]
    fn detect_schema_changes(&self, path: &str, old_schema: &Value, new_schema: &Value, changes: &mut Vec<SchemaChange>) {
        match (old_schema, new_schema) {
            (Value::Object(old_obj), Value::Object(new_obj)) => {
                // Compare properties
                for (key, old_value) in old_obj {
                    if let Some(new_value) = new_obj.get(key) {
                        if old_value != new_value {
                            let mut metadata = HashMap::new();
                            metadata.insert("property".to_string(), key.clone());
                            
                            changes.push(SchemaChange::new(
                                ChangeType::Modification,
                                format!("{}/{}", path, key),
                                format!("Property '{}' was modified", key),
                                metadata,
                            ));
                        }
                    } else {
                        let mut metadata = HashMap::new();
                        metadata.insert("property".to_string(), key.clone());
                        
                        changes.push(SchemaChange::new(
                            ChangeType::Removal,
                            format!("{}/{}", path, key),
                            format!("Property '{}' was removed", key),
                            metadata,
                        ));
                    }
                }

                // Check for new properties
                for key in new_obj.keys() {
                    if !old_obj.contains_key(key) {
                        let mut metadata = HashMap::new();
                        metadata.insert("property".to_string(), key.clone());
                        
                        changes.push(SchemaChange::new(
                            ChangeType::Addition,
                            format!("{}/{}", path, key),
                            format!("New property '{}' was added", key),
                            metadata,
                        ));
                    }
                }
            }
            (old_val, new_val) if old_val != new_val => {
                let mut metadata = HashMap::new();
                metadata.insert("old_value".to_string(), old_val.to_string());
                metadata.insert("new_value".to_string(), new_val.to_string());
                
                changes.push(SchemaChange::new(
                    ChangeType::Modification,
                    path.to_string(),
                    format!("Value changed from {:?} to {:?}", old_val, new_val),
                    metadata,
                ));
            }
            _ => {}
        }
    }

    fn compare_objects(&self, old_obj: &serde_json::Map<String, Value>, new_obj: &serde_json::Map<String, Value>, path: &str, changes: &mut Vec<SchemaChange>) {
        // Compare properties
        for (key, old_value) in old_obj {
            if let Some(new_value) = new_obj.get(key) {
                self.compare_schemas(old_value, new_value, &format!("{}/{}", path, key), changes);
            } else {
                let mut metadata = HashMap::new();
                metadata.insert("property".to_string(), key.clone());
                
                changes.push(SchemaChange::new(
                    ChangeType::Removal,
                    format!("{}/{}", path, key),
                    format!("Property '{}' was removed", key),
                    metadata,
                ));
            }
        }

        // Check for new properties
        for key in new_obj.keys() {
            if !old_obj.contains_key(key) {
                let mut metadata = HashMap::new();
                metadata.insert("property".to_string(), key.clone());
                
                changes.push(SchemaChange::new(
                    ChangeType::Addition,
                    format!("{}/{}", path, key),
                    format!("New property '{}' was added", key),
                    metadata,
                ));
            }
        }
    }

    fn compare_arrays(&self, old_arr: &[Value], new_arr: &[Value], path: &str, changes: &mut Vec<SchemaChange>) {
        if old_arr.len() != new_arr.len() {
            let mut metadata = HashMap::new();
            metadata.insert("old_length".to_string(), old_arr.len().to_string());
            metadata.insert("new_length".to_string(), new_arr.len().to_string());
            
            changes.push(SchemaChange::new(
                ChangeType::Modification,
                path.to_string(),
                format!("Array length changed from {} to {}", old_arr.len(), new_arr.len()),
                metadata,
            ));
        }

        for (i, (old_value, new_value)) in old_arr.iter().zip(new_arr.iter()).enumerate() {
            self.compare_schemas(old_value, new_value, &format!("{}/{}", path, i), changes);
        }
    }
} 
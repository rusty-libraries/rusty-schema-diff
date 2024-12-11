//! SQL specific analyzer implementation
//!
//! This module provides functionality for analyzing SQL DDL changes and
//! generating compatibility reports and migration paths.

use sqlparser::ast::{Statement, ColumnDef, ColumnOption};
use crate::analyzer::{SchemaAnalyzer, SchemaChange, ChangeType};
use crate::{Schema, CompatibilityReport, MigrationPlan, ValidationResult, SchemaDiffError};
use crate::error::Result;
use crate::report::{CompatibilityIssue, IssueSeverity, ValidationError};
use std::collections::HashMap;

/// Analyzes SQL DDL changes and generates compatibility reports.
pub struct SqlAnalyzer;

impl SchemaAnalyzer for SqlAnalyzer {
    /// Analyzes compatibility between two SQL DDL versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The original SQL DDL version.
    /// * `new` - The new SQL DDL version to compare against.
    ///
    /// # Returns
    ///
    /// A `CompatibilityReport` detailing the differences and compatibility status.
    fn analyze_compatibility(&self, old: &Schema, new: &Schema) -> Result<CompatibilityReport> {
        let metadata = HashMap::new();

        let mut changes = Vec::new();
        self.compare_schemas(old, new, &mut changes);

        let compatibility_score = self.calculate_compatibility_score(&changes);
        let validation_result = self.validate_changes(&changes)?;

        Ok(CompatibilityReport {
            changes,
            compatibility_score,
            is_compatible: compatibility_score >= 80,
            issues: validation_result.errors.into_iter().map(|err| CompatibilityIssue {
                severity: match err.code.as_str() {
                    "SQL001" => IssueSeverity::Error,
                    "SQL002" => IssueSeverity::Warning,
                    _ => IssueSeverity::Info,
                },
                description: err.message,
                location: err.path,
            }).collect(),
            metadata,
        })
    }

    /// Generates a migration path between SQL DDL versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The source SQL DDL version.
    /// * `new` - The target SQL DDL version.
    ///
    /// # Returns
    ///
    /// A `MigrationPlan` detailing the required changes.
    fn generate_migration_path(&self, old: &Schema, new: &Schema) -> Result<MigrationPlan> {
        let mut changes = Vec::new();
        self.compare_schemas(old, new, &mut changes);
        
        Ok(MigrationPlan::new(
            old.version.to_string(),
            new.version.to_string(),
            changes,
        ))
    }

    fn validate_changes(&self, changes: &[SchemaChange]) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        
        for change in changes {
            if let Some(issue) = self.validate_change(change) {
                errors.push(ValidationError {
                    message: issue.description,
                    path: issue.location,
                    code: match issue.severity {
                        IssueSeverity::Error => "SQL001",
                        IssueSeverity::Warning => "SQL002",
                        IssueSeverity::Info => "SQL003",
                    }.to_string(),
                });
            }
        }
        
        Ok(ValidationResult {
            errors: errors.clone(),
            is_valid: errors.is_empty(),
            context: HashMap::new(),
        })
    }
}

impl SqlAnalyzer {
    fn compare_schemas(&self, old: &Schema, new: &Schema, changes: &mut Vec<SchemaChange>) {
        if let (Ok(old_tables), Ok(new_tables)) = (
            self.parse_tables(&old.content),
            self.parse_tables(&new.content)
        ) {
            // Compare existing tables
            for old_table in old_tables.iter() {
                if let Statement::CreateTable(ref old_table_data) = old_table {
                    let name = &old_table_data.name;
                    let old_columns = &old_table_data.columns;
                    if let Some(new_table) = new_tables.iter().find(|t| {
                        if let Statement::CreateTable(ref new_table_data) = t {
                            &new_table_data.name == name
                        } else {
                            false
                        }
                    }) {
                        if let Statement::CreateTable(ref new_table_data) = new_table {
                            let new_columns = &new_table_data.columns;
                            self.compare_columns(name.to_string(), old_columns, new_columns, changes);
                        }
                    } else {
                        let mut metadata = HashMap::new();
                        metadata.insert("table".to_string(), name.to_string());
                        
                        changes.push(SchemaChange::new(
                            ChangeType::Removal,
                            format!("table/{}", name),
                            format!("Table '{}' was removed", name),
                            metadata,
                        ));
                    }
                }
            }

            // Check for new tables
            for new_table in new_tables.iter() {
                if let Statement::CreateTable(ref new_table_data) = new_table {
                    let table_name = &new_table_data.name;
                    if !old_tables.iter().any(|t| {
                        if let Statement::CreateTable(ref old_table_data) = t {
                            &old_table_data.name == table_name
                        } else {
                            false
                        }
                    }) {
                        let mut metadata = HashMap::new();
                        metadata.insert("table".to_string(), table_name.to_string());
                        
                        changes.push(SchemaChange::new(
                            ChangeType::Addition,
                            format!("table/{}", table_name),
                            format!("New table '{}' was added", table_name),
                            metadata,
                        ));
                    }
                }
            }
        }
    }

    fn compare_columns(&self, table_name: String, old_columns: &[ColumnDef], new_columns: &[ColumnDef], changes: &mut Vec<SchemaChange>) {
        for old_col in old_columns {
            if let Some(new_col) = new_columns.iter().find(|c| c.name == old_col.name) {
                // Compare data types
                if old_col.data_type != new_col.data_type {
                    let mut metadata = HashMap::new();
                    metadata.insert("table".to_string(), table_name.clone());
                    metadata.insert("column".to_string(), old_col.name.to_string());
                    metadata.insert("old_type".to_string(), format!("{:?}", old_col.data_type));
                    metadata.insert("new_type".to_string(), format!("{:?}", new_col.data_type));
                    
                    changes.push(SchemaChange::new(
                        ChangeType::Modification,
                        format!("{}/{}", table_name, old_col.name),
                        format!("Column '{}' type changed from {:?} to {:?}", 
                            old_col.name, old_col.data_type, new_col.data_type),
                        metadata,
                    ));
                }

                // Update this section to convert the types
                let old_opts: Vec<ColumnOption> = old_col.options.iter()
                    .map(|opt| opt.option.clone())
                    .collect();
                let new_opts: Vec<ColumnOption> = new_col.options.iter()
                    .map(|opt| opt.option.clone())
                    .collect();

                // Now pass the converted options
                self.compare_column_constraints(
                    &table_name,
                    &old_col.name.to_string(),
                    &old_opts,
                    &new_opts,
                    changes,
                );
            } else {
                let mut metadata = HashMap::new();
                metadata.insert("table".to_string(), table_name.clone());
                metadata.insert("column".to_string(), old_col.name.to_string());
                
                changes.push(SchemaChange::new(
                    ChangeType::Removal,
                    format!("{}/{}", table_name, old_col.name),
                    format!("Column '{}' was removed", old_col.name),
                    metadata,
                ));
            }
        }

        // Check for new columns
        for new_col in new_columns {
            if !old_columns.iter().any(|c| c.name == new_col.name) {
                let mut metadata = HashMap::new();
                metadata.insert("table".to_string(), table_name.clone());
                metadata.insert("column".to_string(), new_col.name.to_string());
                
                changes.push(SchemaChange::new(
                    ChangeType::Addition,
                    format!("{}/{}", table_name, new_col.name),
                    format!("New column '{}' was added", new_col.name),
                    metadata,
                ));
            }
        }
    }

    fn compare_column_constraints(
        &self,
        table_name: &str,
        column_name: &str,
        old_options: &[ColumnOption],
        new_options: &[ColumnOption],
        changes: &mut Vec<SchemaChange>,
    ) {
        // Compare constraints
        for old_opt in old_options {
            let found_in_new = new_options.iter().any(|new_opt| {
                match (old_opt, new_opt) {
                    (ColumnOption::NotNull, ColumnOption::NotNull) => true,
                    (ColumnOption::Default(_), ColumnOption::Default(_)) => true,
                    (ColumnOption::Unique { is_primary, characteristics: _ }, 
                     ColumnOption::Unique { is_primary: new_primary, characteristics: _ }) => {
                        is_primary == new_primary
                    }
                    _ => false,
                }
            });

            if !found_in_new {
                let mut metadata = HashMap::new();
                metadata.insert("table".to_string(), table_name.to_string());
                metadata.insert("column".to_string(), column_name.to_string());
                metadata.insert("constraint".to_string(), format!("{:?}", old_opt));
                
                changes.push(SchemaChange::new(
                    ChangeType::Removal,
                    format!("{}/{}/constraints", table_name, column_name),
                    format!("Constraint removed from column '{}': {:?}", column_name, old_opt),
                    metadata,
                ));
            }
        }

        // Check for new constraints
        for new_opt in new_options {
            let found_in_old = old_options.iter().any(|old_opt| {
                match (old_opt, new_opt) {
                    (ColumnOption::NotNull, ColumnOption::NotNull) => true,
                    (ColumnOption::Default(_), ColumnOption::Default(_)) => true,
                    (ColumnOption::Unique { is_primary, characteristics: _ }, 
                     ColumnOption::Unique { is_primary: new_primary, characteristics: _ }) => {
                        is_primary == new_primary
                    }
                    _ => false,
                }
            });

            if !found_in_old {
                let mut metadata = HashMap::new();
                metadata.insert("table".to_string(), table_name.to_string());
                metadata.insert("column".to_string(), column_name.to_string());
                metadata.insert("constraint".to_string(), format!("{:?}", new_opt));
                
                changes.push(SchemaChange::new(
                    ChangeType::Addition,
                    format!("{}/{}/constraints", table_name, column_name),
                    format!("New constraint added to column '{}': {:?}", column_name, new_opt),
                    metadata,
                ));
            }
        }
    }

    fn calculate_compatibility_score(&self, changes: &[SchemaChange]) -> u8 {
        let base_score: u8 = 100;
        let mut deductions: u8 = 0;

        for change in changes {
            match change.change_type {
                ChangeType::Addition => deductions = deductions.saturating_add(5),
                ChangeType::Removal => deductions = deductions.saturating_add(15),
                ChangeType::Modification => deductions = deductions.saturating_add(10),
                ChangeType::Rename => deductions = deductions.saturating_add(8),
            }
        }

        base_score.saturating_sub(deductions)
    }

    fn validate_change(&self, change: &SchemaChange) -> Option<CompatibilityIssue> {
        match change.change_type {
            ChangeType::Removal => Some(CompatibilityIssue {
                severity: IssueSeverity::Error,
                description: format!("Breaking change: {}", change.description),
                location: change.location.clone(),
            }),
            ChangeType::Modification => {
                if change.location.contains("type") {
                    Some(CompatibilityIssue {
                        severity: IssueSeverity::Warning,
                        description: format!("Potential data loss: {}", change.description),
                        location: change.location.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_tables(&self, sql: &str) -> Result<Vec<Statement>> {
        use sqlparser::dialect::GenericDialect;
        use sqlparser::parser::Parser;
        
        let dialect = GenericDialect {};
        Parser::parse_sql(&dialect, sql)
            .map_err(|e| SchemaDiffError::ParseError(format!("Failed to parse SQL: {}", e)))
    }

    #[allow(dead_code)]
    fn generate_sql_for_change(&self, change: &SchemaChange) -> String {
        match change.change_type {
            ChangeType::Addition => {
                if change.location.starts_with("table/") {
                    format!("CREATE TABLE {} (...);", change.location.strip_prefix("table/").unwrap_or(""))
                } else {
                    format!("ALTER TABLE {} ADD COLUMN ...;", change.location)
                }
            }
            ChangeType::Removal => {
                if change.location.starts_with("table/") {
                    format!("DROP TABLE {};", change.location.strip_prefix("table/").unwrap_or(""))
                } else {
                    format!("ALTER TABLE {} DROP COLUMN ...;", change.location)
                }
            }
            ChangeType::Modification => {
                format!("ALTER TABLE {} MODIFY COLUMN ...;", change.location)
            }
            ChangeType::Rename => {
                format!("ALTER TABLE {} RENAME ...;", change.location)
            }
        }
    }
} 
//! OpenAPI specific analyzer implementation
//!
//! This module provides functionality for analyzing OpenAPI changes and
//! generating compatibility reports and migration paths.

use openapiv3::{OpenAPI, ReferenceOr, Parameter, RequestBody, Responses};
use crate::analyzer::{SchemaAnalyzer, SchemaChange, ChangeType};
use crate::report::{CompatibilityIssue, IssueSeverity, ValidationError};
use crate::{Schema, CompatibilityReport, MigrationPlan, ValidationResult};
use crate::error::Result;
use std::collections::HashMap;
use crate::error::SchemaDiffError;

/// Analyzes OpenAPI changes and generates compatibility reports.
pub struct OpenApiAnalyzer;

impl SchemaAnalyzer for OpenApiAnalyzer {
    /// Analyzes compatibility between two OpenAPI versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The original OpenAPI version.
    /// * `new` - The new OpenAPI version to compare against.
    ///
    /// # Returns
    ///
    /// A `CompatibilityReport` detailing the differences and compatibility status.
    fn analyze_compatibility(&self, old: &Schema, new: &Schema) -> Result<CompatibilityReport> {
        let mut changes = Vec::new();
        let mut metadata = HashMap::new();

        let old_spec: OpenAPI = serde_yaml::from_str(&old.content)
            .map_err(|e| SchemaDiffError::ParseError(format!("Failed to parse OpenAPI: {}", e)))?;
        let new_spec: OpenAPI = serde_yaml::from_str(&new.content)
            .map_err(|e| SchemaDiffError::ParseError(format!("Failed to parse OpenAPI: {}", e)))?;

        // Compare paths
        for (path, old_path_item) in old_spec.paths.paths.iter() {
            if let ReferenceOr::Item(old_item) = old_path_item {
                if let Some(new_path_item) = new_spec.paths.paths.get(path) {
                    if let ReferenceOr::Item(new_item) = new_path_item {
                        self.compare_operations(path, old_item, new_item, &mut changes);
                    }
                } else {
                    let mut metadata = HashMap::new();
                    metadata.insert("path".to_string(), path.to_string());
                    
                    changes.push(SchemaChange::new(
                        ChangeType::Removal,
                        format!("paths/{}", path),
                        format!("Path '{}' was removed", path),
                        metadata,
                    ));
                }
            }
        }

        // Check for new paths
        for (path, new_path_item) in new_spec.paths.paths.iter() {
            if let ReferenceOr::Item(_) = new_path_item {
                if !old_spec.paths.paths.contains_key(path) {
                    let mut metadata = HashMap::new();
                    metadata.insert("path".to_string(), path.to_string());
                    
                    changes.push(SchemaChange::new(
                        ChangeType::Addition,
                        format!("paths/{}", path),
                        format!("New path '{}' was added", path),
                        metadata,
                    ));
                }
            }
        }

        // Compare versions
        metadata.insert("new_version".to_string(), new_spec.info.version.to_string());
        metadata.insert("old_version".to_string(), old_spec.info.version.to_string());

        let compatibility_score = self.calculate_compatibility_score(&changes);
        let validation_result = self.validate_changes(&changes)?;

        Ok(CompatibilityReport {
            changes,
            compatibility_score: compatibility_score as u8,
            is_compatible: compatibility_score >= 80,
            metadata,
            issues: validation_result.errors.into_iter().map(|err| CompatibilityIssue {
                severity: match err.code.as_str() {
                    "API001" => IssueSeverity::Error,
                    "API002" => IssueSeverity::Warning,
                    _ => IssueSeverity::Info,
                },
                description: err.message,
                location: err.path.clone(),
            }).collect(),
        })
    }

    /// Generates a migration path between OpenAPI versions.
    ///
    /// # Arguments
    ///
    /// * `old` - The source OpenAPI version.
    /// * `new` - The target OpenAPI version.
    ///
    /// # Returns
    ///
    /// A `MigrationPlan` detailing the required changes.
    fn generate_migration_path(&self, old: &Schema, new: &Schema) -> Result<MigrationPlan> {
        let old_api = self.parse_openapi(&old.content)?;
        let new_api = self.parse_openapi(&new.content)?;

        let mut changes = Vec::new();
        self.compare_apis(&old_api, &new_api, &mut changes)?;

        Ok(MigrationPlan::new(
            old.version.to_string(),
            new.version.to_string(),
            changes,
        ))
    }

    fn validate_changes(&self, changes: &[SchemaChange]) -> Result<ValidationResult> {
        let errors = changes
            .iter()
            .filter_map::<ValidationError, _>(|change| self.validate_change(change))
            .collect::<Vec<ValidationError>>();

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            context: self.build_validation_context(changes),
        })
    }
}

impl OpenApiAnalyzer {
    /// Parses OpenAPI content
    fn parse_openapi(&self, content: &str) -> Result<OpenAPI> {
        serde_json::from_str(content)
            .map_err(|e| SchemaDiffError::ParseError(format!("Failed to parse OpenAPI: {}", e)))
    }

    /// Compares two OpenAPI specifications
    fn compare_apis(&self, old: &OpenAPI, new: &OpenAPI, changes: &mut Vec<SchemaChange>) -> Result<()> {
        // Compare paths
        self.compare_paths(old, new, changes);
        // Compare components
        self.compare_components(old, new, changes);
        // Compare security schemes
        self.compare_security(old, new, changes);
        
        Ok(())
    }

    /// Compares API paths
    fn compare_paths(&self, old: &OpenAPI, new: &OpenAPI, changes: &mut Vec<SchemaChange>) {
        for (path, old_item) in old.paths.paths.iter() {
            match new.paths.paths.get(path) {
                Some(new_item) => {
                    self.compare_path_items(path, old_item, new_item, changes);
                }
                None => {
                    changes.push(SchemaChange::new(
                        ChangeType::Removal,
                        format!("/paths/{}", path),
                        format!("Removed path: {}", path),
                        HashMap::new(),
                    ));
                }
            }
        }

        for path in new.paths.paths.keys() {
            if !old.paths.paths.contains_key(path) {
                changes.push(SchemaChange::new(
                    ChangeType::Addition,
                    format!("/paths/{}", path),
                    format!("Added path: {}", path),
                    HashMap::new(),
                ));
            }
        }
    }

    /// Compares API operations
    fn compare_operations(
        &self,
        path: &str,
        old_item: &openapiv3::PathItem,
        new_item: &openapiv3::PathItem,
        changes: &mut Vec<SchemaChange>,
    ) {
        // Compare HTTP methods
        let methods = ["get", "post", "put", "delete", "patch", "head", "options"];
        
        for method in methods.iter() {
            let old_op = self.get_operation(old_item, method);
            let new_op = self.get_operation(new_item, method);

            match (old_op, new_op) {
                (Some(old_op), Some(new_op)) => {
                    self.compare_parameters(path, method, &old_op.parameters, &new_op.parameters, changes);
                    self.compare_operation_details(path, method, old_op, new_op, changes);
                }
                (Some(_), None) => {
                    changes.push(SchemaChange::new(
                        ChangeType::Removal,
                        format!("/paths{}/{}", path, method),
                        format!("HTTP method '{}' was removed from '{}'", method, path),
                        HashMap::new()
                    ));
                }
                (None, Some(_)) => {
                    changes.push(SchemaChange::new(
                        ChangeType::Addition,
                        format!("/paths{}/{}", path, method),
                        format!("HTTP method '{}' was added to '{}'", method, path),
                        HashMap::new(),
                    ));
                }
                (None, None) => {}
            }
        }
    }

    /// Gets operation for a specific HTTP method
    fn get_operation<'a>(
        &self,
        item: &'a openapiv3::PathItem,
        method: &str,
    ) -> Option<&'a openapiv3::Operation> {
        match method {
            "get" => item.get.as_ref(),
            "post" => item.post.as_ref(),
            "put" => item.put.as_ref(),
            "delete" => item.delete.as_ref(),
            "patch" => item.patch.as_ref(),
            "head" => item.head.as_ref(),
            "options" => item.options.as_ref(),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn extract_metadata(&self, old: &OpenAPI, new: &OpenAPI) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        metadata.insert("new_version".to_string(), new.info.version.to_string());
        metadata.insert("old_version".to_string(), old.info.version.to_string());

        metadata
    }

    /// Calculates compatibility score
    fn calculate_compatibility_score(&self, changes: &[SchemaChange]) -> i32 {
        let base_score: i32 = 100;
        let mut deductions: i32 = 0;
        
        for change in changes {
            match change.change_type {
                ChangeType::Addition => deductions += 5,
                ChangeType::Removal => deductions += 20,
                ChangeType::Modification => {
                    if change.description.contains("optional to required") {
                        deductions += 25;
                    } else {
                        deductions += 10;
                    }
                }
                ChangeType::Rename => deductions += 8,
            }
        }
        
        base_score.saturating_sub(deductions)
    }

    #[allow(dead_code)]
    fn detect_issues(&self, changes: &[SchemaChange]) -> Vec<CompatibilityIssue> {
        changes.iter()
            .filter_map(|change| {
                let severity = match change.change_type {
                    ChangeType::Removal => IssueSeverity::Error,
                    ChangeType::Modification => IssueSeverity::Warning,
                    ChangeType::Rename => IssueSeverity::Info,
                    ChangeType::Addition => IssueSeverity::Info,
                };

                Some(CompatibilityIssue {
                    severity,
                    description: change.description.clone(),
                    location: change.location.clone(),
                })
            })
            .collect()
    }

    /// Builds validation context for changes
    fn build_validation_context(&self, changes: &[SchemaChange]) -> HashMap<String, String> {
        let mut context = HashMap::new();
        
        // Count changes by type
        let mut additions = 0;
        let mut removals = 0;
        let mut modifications = 0;
        let mut renames = 0;

        for change in changes {
            match change.change_type {
                ChangeType::Addition => additions += 1,
                ChangeType::Removal => removals += 1,
                ChangeType::Modification => modifications += 1,
                ChangeType::Rename => renames += 1,
            }
        }

        context.insert("additions".to_string(), additions.to_string());
        context.insert("removals".to_string(), removals.to_string());
        context.insert("modifications".to_string(), modifications.to_string());
        context.insert("renames".to_string(), renames.to_string());
        context.insert("total_changes".to_string(), changes.len().to_string());

        context
    }

    /// Compares components between OpenAPI versions
    fn compare_components(
        &self,
        old: &OpenAPI,
        new: &OpenAPI,
        changes: &mut Vec<SchemaChange>,
    ) {
        if let (Some(old_components), Some(new_components)) = (&old.components, &new.components) {
            // Compare schemas
            for (name, old_schema) in &old_components.schemas {
                match new_components.schemas.get(name) {
                    Some(new_schema) => {
                        if old_schema != new_schema {
                            changes.push(SchemaChange::new(
                                ChangeType::Modification,
                                format!("/components/schemas/{}", name),
                                format!("Schema '{}' was modified", name),
                                HashMap::new(),
                            ));
                        }
                    }
                    None => {
                        changes.push(SchemaChange::new(
                            ChangeType::Removal,
                            format!("/components/schemas/{}", name),
                            format!("Schema '{}' was removed", name),
                            HashMap::new(),
                        ));
                    }
                }
            }

            // Check for new schemas
            for name in new_components.schemas.keys() {
                if !old_components.schemas.contains_key(name) {
                    changes.push(SchemaChange::new(
                        ChangeType::Addition,
                        format!("/components/schemas/{}", name),
                        format!("Schema '{}' was added", name),
                        HashMap::new(),
                    ));
                }
            }
        }
    }

    /// Compares security schemes
    fn compare_security(
        &self,
        old: &OpenAPI,
        new: &OpenAPI,
        changes: &mut Vec<SchemaChange>,
    ) {
        if let (Some(old_components), Some(new_components)) = (&old.components, &new.components) {
            // Compare security schemes
            for (name, old_scheme) in &old_components.security_schemes {
                match new_components.security_schemes.get(name) {
                    Some(new_scheme) => {
                        if old_scheme != new_scheme {
                            changes.push(SchemaChange::new(
                                ChangeType::Modification,
                                format!("/components/securitySchemes/{}", name),
                                format!("Security scheme '{}' was modified", name),
                                HashMap::new(),
                            ));
                        }
                    }
                    None => {
                        changes.push(SchemaChange::new(
                            ChangeType::Removal,
                            format!("/components/securitySchemes/{}", name),
                            format!("Security scheme '{}' was removed", name),
                            HashMap::new(),
                        ));
                    }
                }
            }
        }
    }

    /// Compares operation details
    fn compare_operation_details(
        &self,
        path: &str,
        method: &str,
        old_op: &openapiv3::Operation,
        new_op: &openapiv3::Operation,
        changes: &mut Vec<SchemaChange>,
    ) {
        // Compare parameters
        self.compare_parameters(path, method, &old_op.parameters, &new_op.parameters, changes);

        // Compare request body
        self.compare_request_bodies(path, method, &old_op.request_body, &new_op.request_body, changes);

        // Compare responses
        self.compare_responses(path, method, &old_op.responses, &new_op.responses, changes);
    }

    /// Compares operation parameters
    fn compare_parameters(
        &self,
        path: &str,
        method: &str,
        old_params: &[ReferenceOr<Parameter>],
        new_params: &[ReferenceOr<Parameter>],
        changes: &mut Vec<SchemaChange>,
    ) {
        for old_param in old_params {
            if let ReferenceOr::Item(old_param) = old_param {
                let param_name = match old_param {
                    Parameter::Path { parameter_data, .. } |
                    Parameter::Query { parameter_data, .. } |
                    Parameter::Header { parameter_data, .. } |
                    Parameter::Cookie { parameter_data, .. } => &parameter_data.name,
                };

                if let Some(new_param) = new_params.iter().find(|p| {
                    if let ReferenceOr::Item(p) = p {
                        match p {
                            Parameter::Path { parameter_data, .. } |
                            Parameter::Query { parameter_data, .. } |
                            Parameter::Header { parameter_data, .. } |
                            Parameter::Cookie { parameter_data, .. } => &parameter_data.name == param_name
                        }
                    } else {
                        false
                    }
                }) {
                    if let ReferenceOr::Item(new_param) = new_param {
                        let old_required = match old_param {
                            Parameter::Path { parameter_data, .. } |
                            Parameter::Query { parameter_data, .. } |
                            Parameter::Header { parameter_data, .. } |
                            Parameter::Cookie { parameter_data, .. } => parameter_data.required,
                        };

                        let new_required = match new_param {
                            Parameter::Path { parameter_data, .. } |
                            Parameter::Query { parameter_data, .. } |
                            Parameter::Header { parameter_data, .. } |
                            Parameter::Cookie { parameter_data, .. } => parameter_data.required,
                        };

                        if !old_required && new_required {
                            let mut metadata = HashMap::new();
                            metadata.insert("path".to_string(), path.to_string());
                            metadata.insert("method".to_string(), method.to_string());
                            metadata.insert("parameter".to_string(), param_name.to_string());
                            
                            changes.push(SchemaChange::new(
                                ChangeType::Modification,
                                format!("paths/{}/{}/parameters/{}", path, method, param_name),
                                format!("Parameter '{}' changed from optional to required", param_name),
                                metadata,
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Compares request bodies
    fn compare_request_bodies(
        &self,
        path: &str,
        method: &str,
        old_body: &Option<ReferenceOr<RequestBody>>,
        new_body: &Option<ReferenceOr<RequestBody>>,
        changes: &mut Vec<SchemaChange>,
    ) {
        match (old_body, new_body) {
            (Some(_), None) => {
                changes.push(SchemaChange::new(
                    ChangeType::Removal,
                    format!("/paths{}/{}/requestBody", path, method),
                    "Request body was removed".to_string(),
                    HashMap::new(),
                ));
            }
            (None, Some(_)) => {
                changes.push(SchemaChange::new(
                    ChangeType::Addition,
                    format!("/paths{}/{}/requestBody", path, method),
                    "Request body was added".to_string(),
                    HashMap::new(),
                ));
            }
            (Some(old_body), Some(new_body)) => {
                if old_body != new_body {
                    changes.push(SchemaChange::new(
                        ChangeType::Modification,
                        format!("/paths{}/{}/requestBody", path, method),
                        "Request body was modified".to_string(),
                        HashMap::new(),
                    ));
                }
            }
            (None, None) => {}
        }
    }

    /// Compares operation responses
    fn compare_responses(
        &self,
        path: &str,
        method: &str,
        old_responses: &Responses,
        new_responses: &Responses,
        changes: &mut Vec<SchemaChange>,
    ) {
        // Compare responses
        for (status, old_response) in &old_responses.responses {
            match new_responses.responses.get(status) {
                Some(new_response) => {
                    if old_response != new_response {
                        changes.push(SchemaChange::new(
                            ChangeType::Modification,
                            format!("/paths{}/{}/responses/{}", path, method, status),
                            format!("Response '{}' was modified", status),
                            HashMap::new(),
                        ));
                    }
                }
                None => {
                    changes.push(SchemaChange::new(
                        ChangeType::Removal,
                        format!("/paths{}/{}/responses/{}", path, method, status),
                        format!("Response '{}' was removed", status),
                        HashMap::new(),
                    ));
                }
            }
        }

        // Check for new responses
        for status in new_responses.responses.keys() {
            if !old_responses.responses.contains_key(status) {
                changes.push(SchemaChange::new(
                    ChangeType::Addition,
                    format!("/paths{}/{}/responses/{}", path, method, status),
                    format!("Response '{}' was added", status),
                    HashMap::new(),
                ));
            }
        }
    }

    fn compare_path_items(
        &self,
        path: &str,
        old_item: &ReferenceOr<openapiv3::PathItem>,
        new_item: &ReferenceOr<openapiv3::PathItem>,
        changes: &mut Vec<SchemaChange>
    ) {
        match (old_item, new_item) {
            (ReferenceOr::Item(old_item), ReferenceOr::Item(new_item)) => {
                self.compare_operations(path, old_item, new_item, changes);
            }
            _ => {
                // Handle reference cases if needed
            }
        }
    }

    fn validate_change(&self, change: &SchemaChange) -> Option<ValidationError> {
        match change.change_type {
            ChangeType::Removal => Some(ValidationError {
                message: format!("Breaking change: {}", change.description),
                path: change.location.clone(),
                code: "API001".to_string(),
            }),
            ChangeType::Modification => {
                // Check if this is a parameter becoming required or type change
                if (change.location.contains("parameters") && change.description.contains("required")) ||
                   (change.location.contains("schema") && change.description.contains("type")) {
                    Some(ValidationError {
                        message: format!("Breaking change: {}", change.description),
                        path: change.location.clone(),
                        code: "API002".to_string(),
                    })
                } else {
                    None
                }
            },
            _ => None
        }
    }
}

#[cfg(test)]
mod tests; 
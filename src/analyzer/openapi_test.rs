#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schema;
    use semver::Version;

    fn create_schema(content: &str, version: &str) -> Schema {
        Schema::new(
            crate::SchemaFormat::OpenAPI,
            content.to_string(),
            Version::parse(version).unwrap(),
        )
    }

    #[test]
    fn test_basic_path_changes() {
        let old_api = r#"{
            "openapi": "3.0.0",
            "info": {
                "version": "1.0.0",
                "title": "Test API"
            },
            "paths": {
                "/users": {
                    "get": {
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            }
        }"#;

        let new_api = r#"{
            "openapi": "3.0.0",
            "info": {
                "version": "1.1.0",
                "title": "Test API"
            },
            "paths": {
                "/users": {
                    "get": {
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                },
                "/users/{id}": {
                    "get": {
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            }
        }"#;

        let analyzer = OpenApiAnalyzer;
        let old_schema = create_schema(old_api, "1.0.0");
        let new_schema = create_schema(new_api, "1.1.0");

        let result = analyzer.analyze_compatibility(&old_schema, &new_schema).unwrap();
        
        assert!(result.is_compatible);
        assert!(result.changes.iter().any(|c| matches!(c.change_type, ChangeType::Addition)));
    }

    #[test]
    fn test_breaking_changes() {
        let old_api = r#"{
            "openapi": "3.0.0",
            "info": {
                "version": "1.0.0",
                "title": "Test API"
            },
            "paths": {
                "/users/{id}": {
                    "get": {
                        "parameters": [
                            {
                                "name": "id",
                                "in": "path",
                                "required": true,
                                "schema": {
                                    "type": "integer"
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            }
        }"#;

        let new_api = r#"{
            "openapi": "3.0.0",
            "info": {
                "version": "2.0.0",
                "title": "Test API"
            },
            "paths": {
                "/users/{id}": {
                    "get": {
                        "parameters": [
                            {
                                "name": "id",
                                "in": "path",
                                "required": true,
                                "schema": {
                                    "type": "string"
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            }
        }"#;

        let analyzer = OpenApiAnalyzer;
        let old_schema = create_schema(old_api, "1.0.0");
        let new_schema = create_schema(new_api, "2.0.0");

        let result = analyzer.analyze_compatibility(&old_schema, &new_schema).unwrap();
        
        assert!(!result.is_compatible);
        assert!(result.issues.iter().any(|i| matches!(i.severity, IssueSeverity::Major)));
    }

    // Add more tests for other scenarios...
} 
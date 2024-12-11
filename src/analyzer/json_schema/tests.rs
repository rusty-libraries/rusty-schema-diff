#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schema;
    use semver::Version;

    fn create_schema(content: &str, version: &str) -> Schema {
        Schema::new(
            crate::SchemaFormat::JsonSchema,
            content.to_string(),
            Version::parse(version).unwrap(),
        )
    }

    #[test]
    fn test_property_changes() {
        let old_schema = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        }"#;

        let new_schema = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            }
        }"#;

        let analyzer = JsonSchemaAnalyzer;
        let result = analyzer.analyze_compatibility(
            &create_schema(old_schema, "1.0.0"),
            &create_schema(new_schema, "1.1.0")
        ).unwrap();

        assert!(result.is_compatible);
        assert!(result.changes.iter().any(|c| matches!(c.change_type, ChangeType::Addition)));
    }
} 
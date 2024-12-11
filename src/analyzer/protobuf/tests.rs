#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schema;
    use semver::Version;

    fn create_schema(content: &str, version: &str) -> Schema {
        Schema::new(
            crate::SchemaFormat::Protobuf,
            content.to_string(),
            Version::parse(version).unwrap(),
        )
    }

    #[test]
    fn test_message_changes() {
        let old_proto = r#"
            message User {
                int32 id = 1;
                string name = 2;
            }
        "#;

        let new_proto = r#"
            message User {
                int32 id = 1;
                string name = 2;
                string email = 3;
            }
        "#;

        let analyzer = ProtobufAnalyzer;
        let result = analyzer.analyze_compatibility(
            &create_schema(old_proto, "1.0.0"),
            &create_schema(new_proto, "1.1.0")
        ).unwrap();

        assert!(result.is_compatible);
        assert!(result.changes.iter().any(|c| matches!(c.change_type, ChangeType::Addition)));
    }
} 
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schema;
    use semver::Version;

    fn create_schema(content: &str, version: &str) -> Schema {
        Schema::new(
            crate::SchemaFormat::SqlDDL,
            content.to_string(),
            Version::parse(version).unwrap(),
        )
    }

    #[test]
    fn test_table_changes() {
        let old_sql = r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
        "#;

        let new_sql = r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT
            );
        "#;

        let analyzer = SqlAnalyzer;
        let result = analyzer.analyze_compatibility(
            &create_schema(old_sql, "1.0.0"),
            &create_schema(new_sql, "1.1.0")
        ).unwrap();

        assert!(result.is_compatible);
        assert!(result.changes.iter().any(|c| matches!(c.change_type, ChangeType::Addition)));
    }
} 
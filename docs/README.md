## Rusty Schema Diff - Function Documentation

### Schema Analysis API

#### Analyze Schema Compatibility

**Functionality:**  
Analyze compatibility between two schema versions, detecting breaking changes and generating detailed reports. Supports multiple schema formats including JSON Schema, OpenAPI, Protobuf, and SQL DDL.

**Parameters:**

- **old_schema (Schema, required):**  
  The original schema version to compare against.

- **new_schema (Schema, required):**  
  The new schema version being analyzed.

**Response:**

- **compatibility_report (CompatibilityReport):**  
  Detailed report containing:
  - **is_compatible (bool):** Overall compatibility status
  - **compatibility_score (u32):** Score from 0-100
  - **changes (Vec<SchemaChange>):** List of detected changes
  - **issues (Vec<CompatibilityIssue>):** Any compatibility issues found

### JSON Schema Analysis

#### Analyze JSON Schema Changes

**Functionality:**  
Analyze changes between JSON Schema versions, with support for complex nested structures and references.

**Usage Example:**

```rust
use rusty_schema_diff::{Schema, SchemaFormat, JsonSchemaAnalyzer, SchemaAnalyzer};
use semver::Version;

let old_schema = Schema::new(
    SchemaFormat::JsonSchema,
    r#"{
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        }
    }"#.to_string(),
    Version::parse("1.0.0").unwrap()
);

let new_schema = Schema::new(
    SchemaFormat::JsonSchema,
    r#"{
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"},
            "email": {"type": "string", "format": "email"}
        }
    }"#.to_string(),
    Version::parse("1.1.0").unwrap()
);

let analyzer = JsonSchemaAnalyzer;
let report = analyzer.analyze_compatibility(&old_schema, &new_schema)?;

println!("Compatibility Score: {}", report.compatibility_score);
```

### OpenAPI Analysis

#### Analyze OpenAPI Changes

**Functionality:**  
Analyze changes between OpenAPI specifications, including endpoints, parameters, request bodies, and responses.

**Usage Example:**

```rust
use rusty_schema_diff::prelude::*;

let old_api = Schema::new(
    SchemaFormat::OpenAPI,
    // Your OpenAPI spec here
    openapi_yaml.to_string(),
    Version::parse("1.0.0").unwrap()
);

let analyzer = OpenApiAnalyzer;
let report = analyzer.analyze_compatibility(&old_api, &new_api)?;

// Check for breaking changes in endpoints
for change in report.changes {
    if change.is_breaking {
        println!("Breaking change in {}: {}", change.location, change.description);
    }
}
```

### SQL DDL Analysis

#### Analyze SQL Schema Changes

**Functionality:**  
Analyze changes between SQL DDL schemas, including table structures, columns, constraints, and indexes.

**Usage Example:**

```rust
use rusty_schema_diff::prelude::*;

let old_ddl = Schema::new(
    SchemaFormat::SqlDDL,
    r#"
    CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        name VARCHAR(255) NOT NULL
    );
    "#.to_string(),
    Version::parse("1.0.0").unwrap()
);

let analyzer = SqlAnalyzer;
let report = analyzer.analyze_compatibility(&old_ddl, &new_ddl)?;

// Generate migration SQL
let plan = analyzer.generate_migration_path(&old_ddl, &new_ddl)?;
for step in plan.steps {
    println!("Migration SQL: {}", step);
}
```

### Protobuf Analysis

#### Analyze Protobuf Changes

**Functionality:**  
Analyze changes between Protobuf schemas, including messages, fields, and services.

**Usage Example:**

```rust
use rusty_schema_diff::prelude::*;

let old_proto = Schema::new(
    SchemaFormat::Protobuf,
    r#"
    syntax = "proto3";
    message User {
        string name = 1;
        int32 age = 2;
    }
    "#.to_string(),
    Version::parse("1.0.0").unwrap()
);

let analyzer = ProtobufAnalyzer;
let report = analyzer.analyze_compatibility(&old_proto, &new_proto)?;

// Check compatibility
if report.is_compatible {
    println!("Schemas are compatible");
    for change in report.changes {
        println!("Change: {}", change.description);
    }
}
```

### Migration Plan Generation

#### Generate Migration Path

**Functionality:**  
Generate step-by-step migration plans between schema versions.

**Parameters:**

- **old_schema (Schema, required):**  
  The source schema version.

- **new_schema (Schema, required):**  
  The target schema version.

**Response:**

- **migration_plan (MigrationPlan):**  
  Contains:
  - **steps (Vec<String>):** Ordered list of migration steps
  - **metadata (HashMap<String, String>):** Additional migration information

### Error Handling

The library uses a custom error type `SchemaDiffError` that covers various error cases:

```rust
pub enum SchemaDiffError {
    ParseError(String),
    ComparisonError(String),
    InvalidFormat(String),
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    ProtobufError(String),
}
```

### Best Practices

1. **Version Management:**
   - Always use semantic versioning for your schemas
   - Include version information in schema metadata

2. **Compatibility Analysis:**
   - Run compatibility checks before deploying schema changes
   - Review all breaking changes carefully
   - Consider backward compatibility requirements

3. **Migration Planning:**
   - Generate and review migration plans before implementation
   - Test migrations in a staging environment
   - Have rollback plans ready

4. **Error Handling:**
   - Implement proper error handling for all schema operations
   - Log and monitor schema analysis results
   - Validate schemas before analysis
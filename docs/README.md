# Rusty Schema Diff - API Documentation

## Core Types and Traits

### SchemaAnalyzer Trait
The foundational trait for implementing schema analysis functionality.

```rust
pub trait SchemaAnalyzer {
    fn analyze_compatibility(&self, old: &Schema, new: &Schema) -> Result<CompatibilityReport>;
    fn generate_migration_path(&self, old: &Schema, new: &Schema) -> Result<MigrationPlan>;
    fn validate_changes(&self, changes: &[SchemaChange]) -> Result<ValidationResult>;
}
```

#### Methods
- **analyze_compatibility**
  - Analyzes compatibility between two schema versions
  - Returns a detailed `CompatibilityReport`
  - Parameters:
    - `old`: Reference to the original schema
    - `new`: Reference to the new schema version
  - Errors: Returns `SchemaDiffError` for parsing or comparison failures

- **generate_migration_path**
  - Generates step-by-step migration instructions
  - Returns a `MigrationPlan` containing ordered migration steps
  - Parameters:
    - `old`: Source schema version
    - `new`: Target schema version
  - Errors: Returns `SchemaDiffError` for invalid migration paths

- **validate_changes**
  - Validates proposed schema changes
  - Returns a `ValidationResult` with validation status and issues
  - Parameters:
    - `changes`: Slice of proposed `SchemaChange` instances
  - Errors: Returns `SchemaDiffError` for validation failures

### Schema Type
Represents a versioned schema instance.

```rust
pub struct Schema {
    pub format: SchemaFormat,
    pub content: String,
    pub version: Version,
}
```

#### Fields
- **format**: The schema format (JsonSchema, OpenAPI, Protobuf, SqlDDL)
- **content**: Raw schema content as a string
- **version**: Semantic version of the schema

#### Methods
- **new(format: SchemaFormat, content: String, version: Version) -> Schema**
  - Creates a new schema instance
  - Validates format and content during construction

### CompatibilityReport
Detailed analysis of schema compatibility.

```rust
pub struct CompatibilityReport {
    pub is_compatible: bool,
    pub compatibility_score: u32,
    pub changes: Vec<SchemaChange>,
    pub issues: Vec<CompatibilityIssue>,
    pub metadata: HashMap<String, String>,
}
```

#### Fields
- **is_compatible**: Overall compatibility status
- **compatibility_score**: Numeric score (0-100) indicating compatibility level
- **changes**: Vector of detected schema changes
- **issues**: Vector of compatibility issues found
- **metadata**: Additional analysis metadata

## Schema Analysis APIs

### JSON Schema Analysis

#### JsonSchemaAnalyzer
Specialized analyzer for JSON Schema compatibility.

```rust
pub struct JsonSchemaAnalyzer;
```

##### Implementation Details
- Supports JSON Schema drafts 4, 6, 7, and 2019-09
- Handles nested schema structures
- Validates type compatibility
- Tracks property requirements
- Analyzes array constraints

##### Example Usage
```rust
use rusty_schema_diff::{Schema, SchemaFormat, JsonSchemaAnalyzer, SchemaAnalyzer};
use semver::Version;

let analyzer = JsonSchemaAnalyzer;
let report = analyzer.analyze_compatibility(&old_schema, &new_schema)?;

// Access detailed changes
for change in report.changes {
    println!("Location: {}", change.location);
    println!("Description: {}", change.description);
    println!("Type: {:?}", change.change_type);
}
```

### OpenAPI Analysis

#### OpenApiAnalyzer
Specialized analyzer for OpenAPI specifications.

```rust
pub struct OpenApiAnalyzer;
```

##### Analysis Coverage
- Endpoint paths and operations
- Request/response schemas
- Parameters and headers
- Security schemes
- Media types
- Server configurations
- API metadata

##### Breaking Change Detection
- Removed endpoints
- Changed parameter requirements
- Modified response structures
- Security requirement changes
- Schema incompatibilities

##### Example Usage
```rust
use rusty_schema_diff::prelude::*;

let analyzer = OpenApiAnalyzer;
let report = analyzer.analyze_compatibility(&old_api, &new_api)?;

// Filter breaking changes
let breaking_changes: Vec<_> = report.changes
    .iter()
    .filter(|c| c.is_breaking)
    .collect();
```

### SQL DDL Analysis

#### SqlAnalyzer
Analyzes changes in SQL database schemas.

```rust
pub struct SqlAnalyzer;
```

##### Supported Operations
- Table creation/deletion
- Column modifications
- Index changes
- Constraint updates
- View modifications
- Trigger changes

##### Migration Generation
```rust
let analyzer = SqlAnalyzer;
let plan = analyzer.generate_migration_path(&old_ddl, &new_ddl)?;

// Access migration steps
for step in plan.steps {
    println!("SQL: {}", step);
}
```

### Protobuf Analysis

#### ProtobufAnalyzer
Analyzes Protocol Buffer schema evolution.

```rust
pub struct ProtobufAnalyzer;
```

##### Analysis Features
- Message structure changes
- Field number validation
- Type compatibility checks
- Service definition changes
- Enum modifications
- Package organization

##### Wire Format Compatibility
- Checks field number reuse
- Validates type changes
- Ensures backward compatibility
- Verifies required fields

## Error Handling

### SchemaDiffError
Comprehensive error type for schema analysis operations.

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

#### Error Categories
- **ParseError**: Schema parsing failures
- **ComparisonError**: Analysis comparison issues
- **InvalidFormat**: Unsupported schema formats
- **IoError**: File system operations
- **JsonError**: JSON processing errors
- **ProtobufError**: Protobuf-specific issues

## Best Practices

### Version Management
1. Use semantic versioning consistently
2. Include version metadata in schemas
3. Track version dependencies

### Compatibility Analysis
1. Run analysis before deployments
2. Review breaking changes carefully
3. Maintain backward compatibility
4. Document version constraints

### Migration Planning
1. Generate and verify migration plans
2. Test migrations in staging
3. Prepare rollback procedures
4. Version migration scripts

### Error Handling
1. Implement comprehensive error handling
2. Log analysis results
3. Monitor migration execution
4. Validate input schemas
# Rusty Schema Diff

![Crates.io](https://img.shields.io/crates/v/rusty-schema-diff) ![docs.rs](https://img.shields.io/docsrs/rusty-schema-diff) ![License](https://img.shields.io/crates/l/rusty-schema-diff)

Welcome to Rusty Schema Diff, a powerful schema evolution analyzer that supports multiple schema formats including JSON Schema, OpenAPI, Protobuf, and SQL DDL. This library helps you analyze and manage schema changes across different versions, detect breaking changes, and generate migration paths.

## Table of Contents

- [Rusty Schema Diff](#rusty-schema-diff)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Installation](#installation)
  - [Getting Started](#getting-started)
    - [Basic Usage](#basic-usage)
    - [Analyzing JSON Schema Changes](#analyzing-json-schema-changes)
    - [Analyzing OpenAPI Changes](#analyzing-openapi-changes)
    - [Analyzing SQL DDL Changes](#analyzing-sql-ddl-changes)
  - [Documentation](#documentation)
  - [License](#license)

## Features

- Multi-format support (JSON Schema, OpenAPI, Protobuf, SQL DDL)
- Breaking change detection
- Compatibility scoring
- Migration path generation
- Detailed change analysis
- Validation and error reporting

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rusty-schema-diff = "0.1.0"
```

## Getting Started

### Basic Usage

```rust
use rusty_schema_diff::{Schema, SchemaFormat, JsonSchemaAnalyzer, SchemaAnalyzer};
use semver::Version;

// Create schema instances
let old_schema = Schema::new(
    SchemaFormat::JsonSchema,
    r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#.to_string(),
    Version::parse("1.0.0").unwrap()
);

let new_schema = Schema::new(
    SchemaFormat::JsonSchema,
    r#"{"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}"#.to_string(),
    Version::parse("1.1.0").unwrap()
);

// Analyze compatibility
let analyzer = JsonSchemaAnalyzer;
let report = analyzer.analyze_compatibility(&old_schema, &new_schema).unwrap();

println!("Compatible: {}", report.is_compatible);
println!("Score: {}", report.compatibility_score);
```

### Analyzing JSON Schema Changes

```rust
use rusty_schema_diff::prelude::*;

let analyzer = JsonSchemaAnalyzer;
let report = analyzer.analyze_compatibility(&old_schema, &new_schema).unwrap();

// Generate migration plan
let plan = analyzer.generate_migration_path(&old_schema, &new_schema).unwrap();

for step in plan.steps {
    println!("Migration step: {}", step);
}
```

### Analyzing OpenAPI Changes

```rust
use rusty_schema_diff::prelude::*;

let analyzer = OpenApiAnalyzer;
let report = analyzer.analyze_compatibility(&old_api, &new_api).unwrap();

println!("Breaking changes:");
for change in report.changes.iter().filter(|c| c.is_breaking) {
    println!("- {}: {}", change.location, change.description);
}
```

### Analyzing SQL DDL Changes

```rust
use rusty_schema_diff::prelude::*;

let analyzer = SqlAnalyzer;
let report = analyzer.analyze_compatibility(&old_ddl, &new_ddl).unwrap();

// Generate SQL migration statements
let plan = analyzer.generate_migration_path(&old_ddl, &new_ddl).unwrap();
for statement in plan.steps {
    println!("SQL: {}", statement);
}
```

## Documentation

For detailed information on all available analyzers and their functionality, please refer to the [API Documentation](https://docs.rs/rusty-schema-diff).

## License

This library is licensed under the MIT License. See the [LICENSE](LICENSE.md) file for details.
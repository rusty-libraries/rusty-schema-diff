//! Rusty Schema Diff - A library for analyzing and managing schema evolution
//! 
//! This library provides tools to analyze and manage schema changes across different versions
//! of data structures, APIs, and database schemas. It supports multiple schema formats including
//! JSON Schema, Protobuf, OpenAPI, and SQL DDL.
//!
//! # Features
//! - Schema compatibility analysis
//! - Migration path generation
//! - Breaking change detection
//! - Multi-format support
//!
//! # Example
//! ```rust
//! use rusty_schema_diff::{Schema, SchemaFormat, JsonSchemaAnalyzer, SchemaAnalyzer};
//! 
//! let old_schema = Schema::new(
//!     SchemaFormat::JsonSchema,
//!     r#"{"type": "object"}"#.to_string(),
//!     "1.0.0".parse().unwrap()
//! );
//! 
//! let new_schema = Schema::new(
//!     SchemaFormat::JsonSchema,
//!     r#"{"type": "object", "required": ["id"]}"#.to_string(),
//!     "1.1.0".parse().unwrap()
//! );
//! 
//! let analyzer = JsonSchemaAnalyzer;
//! let report = analyzer.analyze_compatibility(&old_schema, &new_schema).unwrap();
//! println!("Compatible: {}", report.is_compatible);
//! ```
mod analyzer;
mod schema;
mod migration;
mod report;
mod error;

pub use analyzer::{
    SchemaAnalyzer,
    json_schema::JsonSchemaAnalyzer,
    protobuf::ProtobufAnalyzer,
    openapi::OpenApiAnalyzer,
    sql::SqlAnalyzer,
};
pub use schema::{Schema, SchemaFormat};
pub use migration::MigrationPlan;
pub use report::{CompatibilityReport, ValidationResult};
pub use error::SchemaDiffError;

/// Re-exports of commonly used types
pub mod prelude {
    pub use crate::{
        SchemaAnalyzer,
        Schema,
        SchemaFormat,
        MigrationPlan,
        CompatibilityReport,
        ValidationResult,
        SchemaDiffError,
        JsonSchemaAnalyzer,
        ProtobufAnalyzer,
        OpenApiAnalyzer,
        SqlAnalyzer,
    };
} 
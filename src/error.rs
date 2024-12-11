//! Error types for schema analysis operations
//!
//! This module provides custom error types and a Result type alias for the library.

use thiserror::Error;

/// Represents errors that can occur during schema analysis operations
///
/// This enum covers various error cases that might occur during schema parsing,
/// comparison, and validation operations.
#[derive(Error, Debug)]
pub enum SchemaDiffError {
    /// Error that occurs during schema parsing
    #[error("Failed to parse schema: {0}")]
    ParseError(String),

    /// Error that occurs during schema comparison
    #[error("Schema comparison failed: {0}")]
    ComparisonError(String),

    /// Error that occurs when an invalid schema format is provided
    #[error("Invalid schema format: {0}")]
    InvalidFormat(String),

    /// Error that occurs during file I/O operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error that occurs during JSON parsing
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error that occurs during Protobuf operations
    #[error("Protobuf error: {0}")]
    ProtobufError(String),
}

/// A specialized Result type for schema analysis operations
///
/// This type alias helps simplify error handling throughout the library.
pub type Result<T> = std::result::Result<T, SchemaDiffError>; 
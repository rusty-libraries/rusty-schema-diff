use serde::{Serialize, Deserialize};
use semver::Version;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaFormat {
    JsonSchema,
    Protobuf,
    OpenAPI,
    SqlDDL,
    RustStruct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub format: SchemaFormat,
    pub content: String,
    pub version: Version,
}

impl Schema {
    pub fn new(format: SchemaFormat, content: String, version: Version) -> Self {
        Self {
            format,
            content,
            version,
        }
    }
} 
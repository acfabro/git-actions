use serde::{Deserialize, Serialize};

/// API version for configuration files
pub type ApiVersion = String;


/// Common metadata for configuration resources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// Name of the resource
    pub name: String,
}

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents an API version in the format "group/version"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiVersion {
    pub group: String,
    pub version: String,
}

impl TryFrom<&str> for ApiVersion {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() != 2 {
            return Err(format!(
                "API version must be in the format 'group/version', got: {}",
                s
            ));
        }

        Ok(ApiVersion {
            group: parts[0].to_string(),
            version: parts[1].to_string(),
        })
    }
}

impl ApiVersion {
    /// Creates a new ApiVersion with the specified group and version
    pub fn new(group: &str, version: &str) -> Self {
        Self {
            group: group.to_owned(),
            version: version.to_owned(),
        }
    }
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self {
            group: "git-actions".to_string(),
            version: "v1alpha1".to_string(),
        }
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.group, self.version)
    }
}

// Custom serialization/deserialization for ApiVersion
impl<'de> Deserialize<'de> for ApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        ApiVersion::try_from(String::deserialize(deserializer)?.as_str())
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for ApiVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_api_version_parse() {
        let api_version = ApiVersion::try_from("git-actionsx/v1x").unwrap();
        assert_eq!(api_version.group, "git-actionsx");
        assert_eq!(api_version.version, "v1x");
    }

    #[test]
    fn test_api_version_parse_error() {
        let result = ApiVersion::try_from("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_api_version_serialize() {
        let api_version = ApiVersion::new("git-actions", "v1");
        let serialized = serde_yaml::to_string(&api_version).unwrap();
        assert_eq!(serialized.trim(), "git-actions/v1");
    }

    #[test]
    fn test_api_version_deserialize() {
        let deserialized: ApiVersion = serde_yaml::from_str("git-actions/v1").unwrap();
        assert_eq!(deserialized.group, "git-actions");
        assert_eq!(deserialized.version, "v1");
    }
}

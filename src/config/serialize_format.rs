//! Defines the format of metadata files to be parsed.

/// Represents all the different metadata formats that are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerializeFormat {
    Yaml,
    Json,
}

impl Default for SerializeFormat {
    fn default() -> Self {
        Self::Yaml
    }
}

impl SerializeFormat {
    /// Returns the expected file name extension for files in this format.
    // TODO: When `match` is allowed in `const fn`s, make this `const fn`.
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Yaml => "yml",
            Self::Json => "json",
        }
    }
}

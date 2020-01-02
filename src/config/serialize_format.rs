//! Defines the format of metadata files to be parsed.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerializeFormat {
    Yaml,
    Json,
}

impl Default for SerializeFormat {
    fn default() -> Self {
        SerializeFormat::Yaml
    }
}

impl SerializeFormat {
    pub fn file_extension(&self) -> &'static str {
        match *self {
            Self::Yaml => "yml",
            Self::Json => "json",
        }
    }
}

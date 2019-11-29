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
    pub fn default_file_extension(&self) -> &'static str {
        match *self {
            SerializeFormat::Yaml => "yml",
            SerializeFormat::Json => "json",
        }
    }

    pub fn extra_file_extensions(&self) -> &'static[&'static str] {
        match *self {
            SerializeFormat::Yaml => &["yaml"],
            SerializeFormat::Json => &[],
        }
    }
}

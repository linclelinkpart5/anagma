//! Defines the format of metadata files to be parsed.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetaFormat {
    Yaml,
}

impl Default for MetaFormat {
    fn default() -> Self {
        MetaFormat::Yaml
    }
}

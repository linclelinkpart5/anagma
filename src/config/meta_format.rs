//! Defines the format of metadata files to be parsed.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetaFormat {
    Yaml,
    Json,
}

impl Default for MetaFormat {
    fn default() -> Self {
        MetaFormat::Yaml
    }
}

impl MetaFormat {
    pub fn default_file_extension(&self) -> &'static str {
        match *self {
            MetaFormat::Yaml => "yml",
            MetaFormat::Json => "json",
        }
    }

    pub fn extra_file_extensions(&self) -> &'static[&'static str] {
        match *self {
            MetaFormat::Yaml => &["yaml"],
            MetaFormat::Json => &[],
        }
    }
}

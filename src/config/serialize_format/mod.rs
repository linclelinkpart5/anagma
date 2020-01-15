//! Defines the format of metadata files to be parsed.

mod yaml;
mod json;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::metadata::target::Target;
use crate::metadata::structure::MetaStructure;

#[derive(Debug)]
pub enum Error {
    CannotOpenFile(std::io::Error),
    CannotReadFile(std::io::Error),
    YamlDeserializeError(serde_yaml::Error),
    JsonDeserializeError(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CannotOpenFile(ref err) =>
                write!(f, "cannot open metadata file: {}", err),
            Self::CannotReadFile(ref err) =>
                write!(f, "cannot read metadata file: {}", err),
            Self::YamlDeserializeError(ref err) =>
                write!(f, "cannot deserialize YAML: {}", err),
            Self::JsonDeserializeError(ref err) =>
                write!(f, "cannot deserialize JSON: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CannotOpenFile(ref err) => Some(err),
            Self::CannotReadFile(ref err) => Some(err),
            Self::YamlDeserializeError(ref err) => Some(err),
            Self::JsonDeserializeError(ref err) => Some(err),
        }
    }
}

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

    pub fn from_str<S: AsRef<str>>(&self, s: S, mt: Target) -> Result<MetaStructure, Error> {
        match self {
            Self::Yaml => yaml::read_str(s, mt),
            Self::Json => json::read_str(s, mt),
        }
    }

    pub fn from_file<P: AsRef<Path>>(&self, p: P, mt: Target) -> Result<MetaStructure, Error> {
        let p = p.as_ref();
        let mut f = File::open(p).map_err(Error::CannotOpenFile)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(Error::CannotReadFile)?;

        self.from_str(buffer, mt)
    }
}

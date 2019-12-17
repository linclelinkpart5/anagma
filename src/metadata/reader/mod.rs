//! Methodologies for parsing text representations of metadata in various formats into a usable form.

pub mod yaml;
pub mod json;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::config::serialize_format::SerializeFormat;
use crate::metadata::location::MetaLocation;
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
        match *self {
            Error::CannotOpenFile(ref err) => write!(f, "cannot open metadata file: {}", err),
            Error::CannotReadFile(ref err) => write!(f, "cannot read metadata file: {}", err),
            Error::YamlDeserializeError(ref err) => write!(f, "cannot deserialize YAML: {}", err),
            Error::JsonDeserializeError(ref err) => write!(f, "cannot deserialize JSON: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::CannotOpenFile(ref err) => Some(err),
            Error::CannotReadFile(ref err) => Some(err),
            Error::YamlDeserializeError(ref err) => Some(err),
            Error::JsonDeserializeError(ref err) => Some(err),
        }
    }
}

pub trait MetaReader {
    fn from_str<S: AsRef<str>>(&self, s: S, mt: MetaLocation) -> Result<MetaStructure, Error>;

    fn from_file<P: AsRef<Path>>(&self, p: P, mt: MetaLocation) -> Result<MetaStructure, Error> {
        let p = p.as_ref();
        let mut f = File::open(p).map_err(Error::CannotOpenFile)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(Error::CannotReadFile)?;

        self.from_str(buffer, mt)
    }
}

impl MetaReader for SerializeFormat {
    fn from_str<S: AsRef<str>>(&self, s: S, mt: MetaLocation) -> Result<MetaStructure, Error> {
        Ok(match self {
            SerializeFormat::Yaml => yaml::read_str(s, mt)?,
            SerializeFormat::Json => json::read_str(s, mt)?,
        })
    }
}

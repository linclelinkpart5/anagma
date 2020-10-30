use std::path::{Path, PathBuf};
use std::ffi::{OsStr, OsString};

use thiserror::Error;

use crate::util::SchemaFormat;
use crate::util::Util;

#[derive(Debug, Error)]
pub enum Error {
    #[error("source file name is invalid: {0}")]
    InvalidName(String),
    #[error("source file name does not have an extension: {0}")]
    EmptyExtension(String),
    #[error("unknown format for file extension: {}", .0.to_string_lossy())]
    UnknownExtension(OsString),
}

pub struct Source {
    pub file_name: String,
    pub format: SchemaFormat,
}

impl Source {
    fn _raw_new(file_name: String, opt_format: Option<SchemaFormat>) -> Result<Self, Error> {
        let path = Path::new(&file_name);

        // Ensure that we have a valid file name.
        if !Util::is_valid_item_name(&path) {
            return Err(Error::InvalidName(file_name));
        }

        let format = match opt_format {
            Some(format) => format,
            None => {
                if let Some(ext) = path.extension() {
                    if ext == "json" {
                        SchemaFormat::Json
                    } else if ext == "yml" || ext == "yaml" {
                        SchemaFormat::Yaml
                    } else {
                        return Err(Error::UnknownExtension(ext.to_owned()));
                    }
                } else {
                    return Err(Error::EmptyExtension(file_name));
                }
            },
        };

        Ok(Self { file_name, format, })
    }

    pub fn new(file_name: String, format: SchemaFormat) -> Result<Self, Error> {
        Self::_raw_new(file_name, Some(format))
    }

    pub fn from_name(file_name: String) -> Result<Self, Error> {
        Self::_raw_new(file_name, None)
    }
}

pub struct Sourcer {
    external: Vec<Source>,
    internal: Vec<Source>,
}

//! Methodologies for parsing text representations of metadata in various formats into a usable form.

pub mod yaml;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::BTreeMap;

use config::meta_format::MetaFormat;
use metadata::location::MetaLocation;
use metadata::types::MetaStructure;

#[derive(Debug)]
pub enum Error {
    CannotOpenFile(std::io::Error),
    CannotReadFile(std::io::Error),
    CannotParseMetadata,
    EmptyMetadata,
    CannotConvert(&'static str, &'static str),
    InvalidItemName(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CannotOpenFile(ref err) => write!(f, "cannot open metadata file: {}", err),
            Error::CannotReadFile(ref err) => write!(f, "cannot read metadata file: {}", err),
            Error::CannotParseMetadata => write!(f, "cannot parse metadata"),
            Error::EmptyMetadata => write!(f, "metadata is empty"),
            Error::CannotConvert(source, target) => write!(f, "cannot convert from {} to {}", source, target),
            Error::InvalidItemName(ref item_name) => write!(f, "invalid item name: {}", item_name),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::CannotOpenFile(ref err) => Some(err),
            Error::CannotReadFile(ref err) => Some(err),
            Error::CannotParseMetadata => None,
            Error::EmptyMetadata => None,
            Error::CannotConvert(..) => None,
            Error::InvalidItemName(..) => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash, Deserialize)]
pub enum MetaValRepr {
    Nil,
    Str(String),
    Seq(Vec<MetaValRepr>),
    Map(BTreeMap<String, MetaValRepr>),
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

impl MetaReader for MetaFormat {
    fn from_str<S: AsRef<str>>(&self, s: S, mt: MetaLocation) -> Result<MetaStructure, Error> {
        match *self {
            MetaFormat::Yaml => yaml::read_str(s, mt),
        }
    }

    fn from_file<P: AsRef<Path>>(&self, p: P, mt: MetaLocation) -> Result<MetaStructure, Error> {
        let p = p.as_ref();
        let mut f = File::open(p).map_err(Error::CannotOpenFile)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(Error::CannotReadFile)?;

        self.from_str(buffer, mt)
    }
}

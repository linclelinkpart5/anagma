//! Methodologies for parsing text representations of metadata in various formats into a usable form.

pub mod yaml;

use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::io::Error as IoError;

use failure::ResultExt;
use failure::Error;

use metadata::location::MetaLocation;
use metadata::structure::MetaStructure;

#[derive(Fail, Debug)]
pub enum MetaReaderError {
    #[fail(display = "cannot open file: {:?}", _0)]
    FileOpen(PathBuf, #[cause] IoError),
    #[fail(display = "cannot read file to string")]
    FileRead(#[cause] IoError),
    #[fail(display = "cannot parse metadata")]
    Parse,
    #[fail(display = "metadata is empty")]
    Empty,
    #[fail(display = "cannot convert from {} to {}", _0, _1)]
    Convert(&'static str, &'static str),
}

pub trait MetaReader {
    fn from_str<S: AsRef<str>>(s: S, mt: MetaLocation) -> Result<MetaStructure, MetaReaderError>;

    fn from_file<P: AsRef<Path>>(p: P, mt: MetaLocation) -> Result<MetaStructure, MetaReaderError> {
        let p = p.as_ref();
        let mut f = File::open(p).map_err(|e| MetaReaderError::FileOpen(p.to_path_buf(), e))?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(|e| MetaReaderError::FileRead(e))?;

        Self::from_str(buffer, mt)
    }
}

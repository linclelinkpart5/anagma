//! Methodologies for parsing text representations of metadata in various formats into a usable form.

pub mod yaml;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use failure::Error;

use metadata::location::MetaLocation;
use metadata::structure::MetaStructure;

pub trait MetaReader {
    fn from_str<S: AsRef<str>>(s: S, mt: &MetaLocation) -> Result<MetaStructure, Error>;

    fn from_file<P: AsRef<Path>>(p: P, mt: &MetaLocation) -> Result<MetaStructure, Error> {
        let p = p.as_ref();
        let mut f = File::open(p)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer)?;

        Ok(Self::from_str(buffer, mt)?)
    }
}

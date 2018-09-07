pub mod yaml;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use failure::Error;

use metadata::target::MetaTarget;
use metadata::types::Metadata;

pub trait MetaReader {
    fn from_str<S: AsRef<str>>(s: S, mt: MetaTarget) -> Result<Metadata, Error>;

    fn from_file<P: AsRef<Path>>(p: P, mt: MetaTarget) -> Result<Metadata, Error> {
        let p = p.as_ref();
        let mut f = File::open(p)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer)?;

        // Self::from_str(buffer, mt).chain_err(|| "unable to parse file")
        Ok(Self::from_str(buffer, mt)?)
    }
}

pub fn read_metadata_from_str<MR: MetaReader, S: AsRef<str>>(s: S, mt: MetaTarget) -> Result<Metadata, Error> {
    MR::from_str(s, mt)
}

pub fn read_metadata_from_file<MR: MetaReader, P: AsRef<Path>>(p: P, mt: MetaTarget) -> Result<Metadata, Error> {
    MR::from_file(p, mt)
}

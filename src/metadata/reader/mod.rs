//! Methodologies for parsing text representations of metadata in various formats into a usable form.

pub mod yaml;

use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;

use failure::Backtrace;
use failure::Context;
use failure::Fail;
use failure::ResultExt;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Fail, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    #[fail(display = "cannot open metadata file")]
    FileOpen,
    #[fail(display = "cannot read metadata file to string")]
    FileRead,
    #[fail(display = "cannot parse metadata")]
    Parse,
    #[fail(display = "metadata is empty")]
    Empty,
    #[fail(display = "cannot convert from {} to {}", _0, _1)]
    Convert(&'static str, &'static str),
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> { self.inner.cause() }
    fn backtrace(&self) -> Option<&Backtrace> { self.inner.backtrace() }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.inner, f) }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind { self.inner.get_context() }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error { Error { inner: Context::new(kind) } }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error { Error { inner: inner } }
}

use std::path::Path;
use std::fs::File;
use std::io::Read;

use metadata::location::MetaLocation;
use metadata::structure::MetaStructure;

pub trait MetaReader {
    fn from_str<S: AsRef<str>>(s: S, mt: MetaLocation) -> Result<MetaStructure, Error>;

    fn from_file<P: AsRef<Path>>(p: P, mt: MetaLocation) -> Result<MetaStructure, Error> {
        let p = p.as_ref();
        let mut f = File::open(p).context(ErrorKind::FileOpen)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).context(ErrorKind::FileRead)?;

        Self::from_str(buffer, mt)
    }
}

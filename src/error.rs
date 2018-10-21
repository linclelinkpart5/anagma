use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;
use std::path::PathBuf;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    #[fail(display = "invalid directory path: {:?}", _0)]
    InvalidDirPath(PathBuf),
    #[fail(display = "invalid file path: {:?}", _0)]
    InvalidFilePath(PathBuf),
    #[fail(display = "path does not exist: {:?}", _0)]
    NonexistentPath(PathBuf),
    #[fail(display = "path does not have a parent and/or is filesystem root: {:?}", _0)]
    NoPathParent(PathBuf),
    #[fail(display = "unable to read entries in directory: {:?}", _0)]
    CannotReadDir(PathBuf),
    #[fail(display = "unable to read directory entry")]
    CannotReadDirEntry,

    #[fail(display = "malformed metadata file: {:?}", _0)]
    MalformedMetadata(PathBuf),
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

use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

#[derive(Debug)]
struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
enum ErrorKind {
    #[fail(display = "item path is not a directory")]
    InvalidItemDirPath,
    #[fail(display = "item path is not a file")]
    InvalidItemFilePath,
    #[fail(display = "meta path is not a file")]
    InvalidMetaFilePath,
    #[fail(display = "item path does not exist")]
    NonexistentItemPath,
    #[fail(display = "meta path does not exist")]
    NonexistentMetaPath,
    #[fail(display = "path does not have a parent or is filesystem root")]
    NoParentPath,
    #[fail(display = "item path not found in processed metadata")]
    NoMetadataFound,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

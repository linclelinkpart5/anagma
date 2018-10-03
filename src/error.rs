use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "path is not a directory")]
    NotADirPath,
    #[fail(display = "path is not a file")]
    NotAFilePath,
    #[fail(display = "path does not exist")]
    NonexistentPath,
    #[fail(display = "path does not have a parent or is filesystem root")]
    NoPathParent,
    #[fail(display = "item path not found in processed metadata")]
    NoMetadataFound,
    #[fail(display = "unable to parse/read meatadata file")]
    CannotParseMetadata,
    #[fail(display = "unable to read entries in directory")]
    CannotReadDir,
    #[fail(display = "unable to read directory entry")]
    CannotReadDirEntry,
    #[fail(display = "unable to find meta path from item path")]
    CannotFindMetaPath,
    #[fail(display = "invalid glob pattern")]
    InvalidGlobPattern,
    #[fail(display = "unable to build selector")]
    CannotBuildSelector,
    #[fail(display = "unable to open metadata file")]
    CannotOpenMetadataFile,
    #[fail(display = "unable to read metadata file")]
    CannotReadMetadataFile,

    #[fail(display = "unable to read YAML file")]
    CannotReadYamlFile,
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

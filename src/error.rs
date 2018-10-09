use std::path::PathBuf;
use std::io::Error as IoError;

use failure::Fail;

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
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
}

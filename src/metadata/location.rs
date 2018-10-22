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
    #[fail(display = "invalid directory path")]
    InvalidDirPath,
    #[fail(display = "invalid file path")]
    InvalidFilePath,
    #[fail(display = "path does not exist")]
    NonexistentPath,
    #[fail(display = "path does not have a parent and/or is filesystem root")]
    NoPathParent,
    #[fail(display = "unable to read entries in directory")]
    CannotReadDir,
    #[fail(display = "unable to read directory entry")]
    CannotReadDirEntry,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FileTarget {
    Item,
    Meta,
}

use std::path::Path;
use std::path::PathBuf;
use std::fs;

use library::config::Config;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum MetaLocation {
    Contains,
    Siblings,
}

impl MetaLocation {
    pub fn get_meta_path<P: AsRef<Path>>(&self, item_path: P) -> Result<PathBuf, Error> {
        let item_path = item_path.as_ref();

        if !item_path.exists() {
            Err(ErrorKind::NonexistentPath)?
        }

        let meta_path = match *self {
            MetaLocation::Contains => {
                if !item_path.is_dir() {
                    Err(ErrorKind::InvalidDirPath)?
                }

                item_path.join("self.yml")
            },
            MetaLocation::Siblings => {
                match item_path.parent() {
                    Some(item_path_parent) => item_path_parent.join("item.yml"),
                    None => Err(ErrorKind::NoPathParent)?,
                }
            }
        };

        if !meta_path.exists() {
            Err(ErrorKind::NonexistentPath)?
        }
        if !meta_path.is_file() {
            Err(ErrorKind::InvalidFilePath)?
        }

        Ok(meta_path)
    }

    /// Provides the possible owned item paths of this location.
    /// This is a listing of the file paths that this meta location *could* provide metadata for.
    /// Note that this does NOT parse meta files, it only uses file system locations and presence.
    /// Also, no filtering or sorting of the returned item paths is performed.
    pub fn get_item_paths<P: AsRef<Path>>(&self, meta_path: P) -> Result<Vec<PathBuf>, Error> {
        let meta_path = meta_path.as_ref();

        if !meta_path.exists() {
            Err(ErrorKind::NonexistentPath)?
        }

        if !meta_path.is_file() {
            Err(ErrorKind::InvalidFilePath)?
        }

        // Get the parent directory of the meta file.
        // NOTE: This is only outside the pattern match because all branches currently use it.
        if let Some(meta_parent_dir_path) = meta_path.parent() {
            let mut po_item_paths = vec![];

            match *self {
                MetaLocation::Contains => {
                    // This is just the passed-in path, just push it on unchanged.
                    po_item_paths.push(meta_parent_dir_path.to_path_buf());
                },
                MetaLocation::Siblings => {
                    // Return all children of this directory.
                    for entry in fs::read_dir(&meta_parent_dir_path).context(ErrorKind::CannotReadDir)? {
                        po_item_paths.push(entry.map_err(|_| ErrorKind::CannotReadDirEntry)?.path());
                    }
                },
            }

            Ok(po_item_paths)
        }
        else {
            // This should never happen!
            Err(ErrorKind::NoPathParent)?
        }
    }

    // NOTE: No sorting is performed, sorting only occurs if needed during plexing.
    pub fn get_selected_item_paths<P: AsRef<Path>>(
        &self,
        meta_path: P,
        config: &Config,
        ) -> Result<Vec<PathBuf>, Error>
    {
        let item_paths = self.get_item_paths(meta_path)?;

        // Use the config object to select the item paths.
        Ok(config.select(item_paths).collect())
    }
}

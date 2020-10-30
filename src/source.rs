use std::borrow::Cow;
// use std::convert::TryFrom;
use std::ffi::OsString;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::ops::Deref;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::config::selection::Selection;

pub struct SourceName(PathBuf);

impl Deref for SourceName {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl<I> TryFrom<I> for SourceName
// where
//     I: Into<PathBuf>,
// {
//     type Error = NameError;

//     fn try_from(value: I) -> Result<Self, Self::Error> {

//     }
// }

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("source file name is invalid: {0}")]
    InvalidName(String),
    #[error("source file name does not have an extension: {0}")]
    EmptyExtension(String),
    #[error("unknown format for file extension: {}", .0.to_string_lossy())]
    UnknownExtension(OsString),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("not a directory: {}", .0.display())]
    NotADir(PathBuf),
    #[error("not a file: {}", .0.display())]
    NotAFile(PathBuf),

    #[error(r#"cannot access item path "{}": {1}"#, .0.display())]
    ItemAccess(PathBuf, #[source] IoError),
    #[error(r#"cannot access meta path "{}": {1}"#, .0.display())]
    MetaAccess(PathBuf, #[source] IoError),

    #[error("item path does not have a parent: {}", .0.display())]
    NoItemParentDir(PathBuf),
    #[error("meta path does not have a parent: {}", .0.display())]
    NoMetaParentDir(PathBuf),

    #[error("unable to read item directory entry: {0}")]
    IterDir(#[source] IoError),
    // IterDirEntry(IoError),

    // Bulk(IoError, Vec<IoError>),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Self::MetaAccess(_, io_error) => match io_error.kind() {
                IoErrorKind::NotFound => false,
                _ => true,
            },
            Self::NotADir(..) | Self::NoItemParentDir(..) => false,
            _ => true,
        }
    }
}

/// Represents a method of finding the location of a meta file given an item
/// file path.
#[derive(Clone, Copy)]
pub enum Anchor {
    /// The meta file is located in the same directory as the item file path.
    External,

    /// The meta file is located inside the item file path.
    /// Implies that the the item file path is a directory.
    Internal,
}

/// Defines a meta file source, consisting of an anchor (the target directory
/// to look in) and a file name (the meta file name in that target directory).
pub struct Source {
    pub file_name: String,
    pub anchor: Anchor,
}

impl Source {
    /// Given a concrete item file path, returns the meta file path that would
    /// provide metadata for that item path, according to the source rules.
    pub fn meta_path(&self, item_path: &Path) -> Result<PathBuf, Error> {
        // Get filesystem stat for item path.
        // This step is always done, even if the file/directory status does not
        // need to be checked, as it provides useful error information about
        // permissions and non-existence.
        let item_fs_stat =
            std::fs::metadata(&item_path).map_err(|io| Error::ItemAccess(item_path.into(), io))?;

        let meta_path_parent_dir = match self.anchor {
            Anchor::External => item_path
                .parent()
                .ok_or_else(|| Error::NoItemParentDir(item_path.into()))?,
            Anchor::Internal => {
                if !item_fs_stat.is_dir() {
                    return Err(Error::NotADir(item_path.into()));
                }

                item_path
            }
        };

        // Create the target meta file path.
        let meta_path = meta_path_parent_dir.join(&self.file_name);

        // Get filesystem stat for meta path.
        // NOTE: Using `match` in order to avoid a clone in the error case.
        let meta_fs_stat = match std::fs::metadata(&meta_path) {
            Ok(o) => o,
            Err(io_err) => return Err(Error::MetaAccess(meta_path, io_err)),
        };

        // Ensure that the meta path is indeed a file.
        if !meta_fs_stat.is_file() {
            // Found a directory with the meta file name, this would be an unusual error case.
            Err(Error::NotAFile(meta_path))
        } else {
            Ok(meta_path)
        }
    }

    /// Provides a listing of the item file paths that this meta target
    /// could/should provide metadata for. Note that this does NOT parse meta
    /// files, it only uses file system locations and presence. In addition, no
    /// filtering or sorting of the returned item paths is performed.
    pub fn item_paths<'a>(&self, meta_path: &'a Path) -> Result<ItemPaths<'a>, Error> {
        let meta_fs_stat =
            std::fs::metadata(&meta_path).map_err(|io| Error::MetaAccess(meta_path.into(), io))?;

        if !meta_fs_stat.is_file() {
            return Err(Error::NotAFile(meta_path.into()));
        }

        // Get the parent directory of the meta file.
        if let Some(meta_parent_dir_path) = meta_path.parent() {
            let ipi = match self.anchor {
                Anchor::External => {
                    // Return all children of the parent directory of this meta file.
                    let read_dir =
                        std::fs::read_dir(&meta_parent_dir_path).map_err(Error::IterDir)?;

                    ItemPathsInner::ReadDir(read_dir)
                }
                Anchor::Internal => {
                    // This is just the passed-in path, just push it on unchanged.
                    ItemPathsInner::Single(Some(meta_parent_dir_path))
                }
            };

            Ok(ItemPaths(ipi))
        } else {
            // This should never happen, since at this point we have a real meta
            // file and thus, a real parent directory for that file, but making
            // an error for it anyways.
            Err(Error::NoMetaParentDir(meta_path.into()))
        }
    }

    /// Similar to `item_paths`, but also performs selection filtering on the
    /// produced item paths.
    pub fn selected_item_paths<'a>(
        &self,
        meta_path: &'a Path,
        selection: &'a Selection,
    ) -> Result<SelectedItemPaths<'a>, Error> {
        Ok(SelectedItemPaths(self.item_paths(meta_path)?, selection))
    }
}

enum ItemPathsInner<'a> {
    ReadDir(std::fs::ReadDir),
    Single(Option<&'a Path>),
}

impl<'a> Iterator for ItemPathsInner<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::ReadDir(rd) => Some(rd.next()?.map(|e| Cow::Owned(e.path()))),
            Self::Single(o) => o.take().map(|p| Ok(Cow::Borrowed(p))),
        }
    }
}

pub struct ItemPaths<'a>(ItemPathsInner<'a>);

impl<'a> Iterator for ItemPaths<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct SelectedItemPaths<'a>(ItemPaths<'a>, &'a Selection);

impl<'a> Iterator for SelectedItemPaths<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(res) = self.0.next() {
            match res {
                Err(err) => {
                    return Some(Err(err));
                }
                Ok(path) => match self.1.is_selected(&path) {
                    Ok(true) => {
                        return Some(Ok(path));
                    }
                    Ok(false) => {
                        continue;
                    }
                    Err(err) => {
                        return Some(Err(err));
                    }
                },
            }
        }

        None
    }
}

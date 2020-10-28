use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::io::{Error as IoError, Result as IoResult};
use std::path::{Path, PathBuf};

use crate::metadata::schema::SchemaFormat;

#[derive(Debug)]
pub enum Error {
    NotADir(PathBuf),
    ItemAccess(PathBuf, IoError),
    NoParentDir(PathBuf),
    IterDir(IoError),
    IterDirEntry(IoError),
    NotAFile(PathBuf),
    MetaAccess(PathBuf, IoError),

    Bulk(IoError, Vec<IoError>),
    // InvalidItemDirPath(PathBuf),
    // CannotAccessItemPath(PathBuf, IoError),
    // NoItemPathParent(PathBuf),
    // CannotReadItemDir(IoError),
    // CannotReadItemDirEntry(IoError),

    // InvalidMetaFilePath(PathBuf),
    // CannotAccessMetaPath(PathBuf, IoError),
    // NoMetaPathParent(PathBuf), // THIS SHOULD NEVER OCCUR, JUST PANIC.

    // BulkSelectionError(IoError, Vec<IoError>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            _ => write!(f, "error!"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            _ => None,
        }
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

pub(crate) struct ItemPaths<'a>(ItemPathsInner<'a>);

impl<'a> Iterator for ItemPaths<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(crate) enum Source {
    /// The metadata file location is a sibling of the target item file path.
    External(String),

    /// The metadata file location is inside the target item file path.
    /// Implies that the the target item file path is a directory.
    Internal(String),
}

impl Source {
    pub(crate) fn fn_stub(&self) -> &str {
        match self {
            Self::External(fn_stub) => fn_stub,
            Self::Internal(fn_stub) => fn_stub,
        }
    }

    /// Given a concrete item file path, returns the meta file path that would
    /// provide metadata for that item path, according to the source rules.
    pub(crate) fn meta_path(
        &self,
        item_path: &Path,
        schema_fmt: &SchemaFormat,
    ) -> Result<PathBuf, Error> {
        // Get filesystem stat for item path.
        // This step is always done, even if the file/directory status does not
        // need to be checked, as it provides useful error information about
        // permissions and non-existence.
        let item_fs_stat =
            std::fs::metadata(&item_path).map_err(|io| Error::ItemAccess(item_path.into(), io))?;

        let meta_path_parent_dir = match self {
            Self::External(..) => item_path
                .parent()
                .ok_or_else(|| Error::NoParentDir(item_path.into()))?,
            Self::Internal(..) => {
                if !item_fs_stat.is_dir() {
                    return Err(Error::NotADir(item_path.into()));
                }

                item_path
            }
        };

        // Create the target meta file name.
        let target_fn = format!("{}.{}", self.fn_stub(), schema_fmt.file_extension());
        let meta_path = meta_path_parent_dir.join(target_fn);

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
            let ipi = match self {
                Self::External(..) => {
                    // Return all children of the parent directory of this meta file.
                    let read_dir =
                        std::fs::read_dir(&meta_parent_dir_path).map_err(Error::IterDir)?;

                    ItemPathsInner::ReadDir(read_dir)
                }
                Self::Internal(..) => {
                    // This is just the passed-in path, just push it on unchanged.
                    ItemPathsInner::Single(Some(meta_parent_dir_path))
                }
            };

            Ok(ItemPaths(ipi))
        } else {
            // This should never happen, since at this point we have a real meta
            // file and thus, a real parent directory for that file, but making
            // an error for it anyways.
            Err(Error::NoParentDir(meta_path.into()))
        }
    }
}

pub struct Compositor(Vec<Source>, SchemaFormat);

impl<'a> Compositor {
    pub(crate) fn new(fmt: SchemaFormat) -> Self {
        Self(Vec::new(), fmt)
    }

    fn _add_src<I, F>(&mut self, fn_stub: I, f: F) -> &mut Self
    where
        I: Into<String>,
        F: FnOnce(String) -> Source,
    {
        let mut src_fn = fn_stub.into();
        src_fn.push('.');
        src_fn.push_str(self.1.file_extension());

        let src = f(src_fn);

        self.0.push(src);
        self
    }

    pub(crate) fn ex_source<I>(&mut self, fn_stub: I) -> &mut Self
    where
        I: Into<String>,
    {
        self._add_src(fn_stub, Source::External)
    }

    pub(crate) fn in_source<I>(&mut self, fn_stub: I) -> &mut Self
    where
        I: Into<String>,
    {
        self._add_src(fn_stub, Source::Internal)
    }

    pub fn compose(&self, item_path: &Path) {}
}

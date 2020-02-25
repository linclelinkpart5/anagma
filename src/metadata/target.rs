use std::path::Path;
use std::path::PathBuf;
use std::borrow::Cow;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

use crate::config::selection::Selection;
use crate::config::meta_format::MetaFormat;

#[derive(Debug)]
pub enum Error {
    InvalidItemDirPath(PathBuf),
    CannotAccessItemPath(PathBuf, IoError),
    NoItemPathParent(PathBuf),
    CannotReadItemDir(IoError),
    CannotReadItemDirEntry(IoError),

    InvalidMetaFilePath(PathBuf),
    CannotAccessMetaPath(PathBuf, IoError),
    NoMetaPathParent(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidItemDirPath(ref p)
                => write!(f, "invalid item directory path: {}", p.display()),
            Self::CannotAccessItemPath(ref p, ref err)
                => write!(f, r#"cannot access item path "{}", error: {}"#, p.display(), err),
            Self::NoItemPathParent(ref p)
                => write!(f, "item path does not have a parent: {}", p.display()),
            Self::CannotReadItemDir(ref err)
                => write!(f, "unable to read entries in item directory: {}", err),
            Self::CannotReadItemDirEntry(ref err)
                => write!(f, "unable to read item directory entry: {}", err),

            Self::InvalidMetaFilePath(ref p)
                => write!(f, "invalid meta file path: {}", p.display()),
            Self::CannotAccessMetaPath(ref p, ref err)
                => write!(f, r#"cannot access meta path "{}", error: {}"#, p.display(), err),
            Self::NoMetaPathParent(ref p)
                => write!(f, "meta path does not have a parent: {}", p.display()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CannotAccessItemPath(_, ref err) => Some(err),
            Self::CannotAccessMetaPath(_, ref err) => Some(err),
            Self::CannotReadItemDir(ref err) => Some(err),
            Self::CannotReadItemDirEntry(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Error {
    pub(crate) fn is_fatal(&self) -> bool {
        match self {
            Self::CannotAccessMetaPath(_, io_error) => {
                match io_error.kind() {
                    IoErrorKind::NotFound => false,
                    _ => true,
                }
            },
            Self::InvalidItemDirPath(..) | Self::NoItemPathParent(..) => false,
            _ => true,
        }
    }
}

/// Represents the target location of the item files that a given metadata file
/// provides metadata for, relative to the location of the metadata file itself.
// WARNING: Do not modify the order of the variants in this enum!
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, EnumIter)]
pub enum Target {
    Siblings,
    Parent,
}

impl Target {
    /// Provides the meta file path that provides metadata for an item file for
    /// this target.
    // NOTE: This always returns a `PathBuf`, since joining paths is required.
    pub fn meta_path<P>(&self, item_path: &P, meta_format: MetaFormat) -> Result<PathBuf, Error>
    where
        P: AsRef<Path>,
    {
        let item_path = item_path.as_ref();

        // Get filesystem stat for item path.
        // This step is always done, even if the file/dir status does not need to be checked,
        // as it provides useful error information about permissions and non-existence.
        let item_fs_stat = match std::fs::metadata(&item_path) {
            Err(err) => return Err(Error::CannotAccessItemPath(item_path.into(), err)),
            Ok(item_fs_stat) => item_fs_stat,
        };

        let meta_path_parent_dir = match self {
            Self::Siblings => {
                match item_path.parent() {
                    Some(item_path_parent) => item_path_parent,
                    None => return Err(Error::NoItemPathParent(item_path.into())),
                }
            },
            Self::Parent => {
                if !item_fs_stat.is_dir() {
                    return Err(Error::InvalidItemDirPath(item_path.into()))
                }

                item_path
            },
        };

        // Create the target meta file name.
        let target_fn = format!("{}.{}", self.default_file_name(), meta_format.file_extension());
        let meta_path = meta_path_parent_dir.join(target_fn);

        // Get filesystem stat for meta path.
        // This step is always done, even if the file/dir status does not need to be checked,
        // as it provides useful error information about permissions and non-existence.
        let meta_fs_stat = match std::fs::metadata(&meta_path) {
            Err(err) => return Err(Error::CannotAccessMetaPath(meta_path, err)),
            Ok(meta_fs_stat) => meta_fs_stat,
        };

        // Ensure that the meta path is indeed a file.
        if !meta_fs_stat.is_file() {
            // Found a directory with the meta file name, this would be an unusual error case.
            Err(Error::InvalidMetaFilePath(meta_path))
        }
        else {
            Ok(meta_path)
        }
    }

    /// Provides a listing of the item file paths that this meta target
    /// could/should provide metadata for. Note that this does NOT parse meta
    /// files, it only uses file system locations and presence. In addition, no
    /// filtering or sorting of the returned item paths is performed.
    pub fn item_paths<'a, P>(&self, meta_path: &'a P) -> Result<Vec<Cow<'a, Path>>, Error>
    where
        P: AsRef<Path>,
    {
        let meta_path = meta_path.as_ref();

        let meta_fs_stat = match std::fs::metadata(&meta_path) {
            Err(err) => return Err(Error::CannotAccessMetaPath(meta_path.into(), err)),
            Ok(meta_fs_stat) => meta_fs_stat,
        };

        if !meta_fs_stat.is_file() {
            return Err(Error::InvalidMetaFilePath(meta_path.into()))
        }

        // Get the parent directory of the meta file.
        // NOTE: This is only outside the pattern match because all branches currently use it.
        if let Some(meta_parent_dir_path) = meta_path.parent() {
            let mut item_paths = vec![];

            match self {
                Self::Siblings => {
                    // Return all children of this directory.
                    let read_dir =
                        std::fs::read_dir(&meta_parent_dir_path)
                        .map_err(Error::CannotReadItemDir)?
                    ;

                    for entry in read_dir {
                        item_paths.push(Cow::Owned(entry.map_err(Error::CannotReadItemDirEntry)?.path()));
                    }
                },
                Self::Parent => {
                    // This is just the passed-in path, just push it on unchanged.
                    item_paths.push(Cow::Borrowed(meta_parent_dir_path));
                },
            }

            Ok(item_paths)
        }
        else {
            // This should never happen!
            Err(Error::NoMetaPathParent(meta_path.into()))?
        }
    }

    /// Similar to `item_paths`, but also performs selection filtering on the
    /// produced item paths.
    // NOTE: No sorting is performed, sorting only occurs if needed during plexing.
    pub fn selected_item_paths<'a, P>(&self, meta_path: &'a P, selection: &Selection) -> Result<Vec<Cow<'a, Path>>, Error>
    where
        P: AsRef<Path>,
    {
        let mut item_paths = self.item_paths(meta_path)?;

        item_paths.retain(|p| selection.is_selected(p));

        Ok(item_paths)
    }

    /// Returns the expected filename stub for a given target.
    pub fn default_file_name(&self) -> &'static str {
        match self {
            Self::Siblings => "item",
            Self::Parent => "self",
        }
    }
}

use std::path::Path;
use std::path::PathBuf;
use std::borrow::Cow;
use std::io::Error as IoError;

use crate::config::selection::Selection;
use crate::config::serialize_format::SerializeFormat;

#[derive(Debug)]
pub enum Error {
    InvalidItemDirPath(PathBuf),
    // InvalidItemFilePath(PathBuf),
    NonexistentItemPath(PathBuf),
    NoItemPathParent(PathBuf),
    CannotReadItemDir(IoError),
    CannotReadItemDirEntry(IoError),

    // InvalidMetaDirPath(PathBuf),
    InvalidMetaFilePath(PathBuf),
    NonexistentMetaPath(Vec<PathBuf>),
    NoMetaPathParent(PathBuf),
    // CannotReadMetaDir(IoError),
    // CannotReadMetaDirEntry(IoError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::InvalidItemDirPath(ref p) => write!(f, "invalid item directory path: {}", p.display()),
            // Self::InvalidItemFilePath(ref p) => write!(f, "invalid item file path: {}", p.display()),
            Self::NonexistentItemPath(ref p) => write!(f, "item path does not exist: {}", p.display()),
            Self::NoItemPathParent(ref p) => write!(f, "item path does not have a parent and/or is filesystem root: {}", p.display()),
            Self::CannotReadItemDir(ref err) => write!(f, "unable to read entries in item directory: {}", err),
            Self::CannotReadItemDirEntry(ref err) => write!(f, "unable to read item directory entry: {}", err),

            // Self::InvalidMetaDirPath(ref p) => write!(f, "invalid meta directory path: {}", p.display()),
            Self::InvalidMetaFilePath(ref p) => write!(f, "invalid meta file path: {}", p.display()),
            Self::NonexistentMetaPath(ref ps) => write!(f, "meta path does not exist, tried: {:?}", ps),
            Self::NoMetaPathParent(ref p) => write!(f, "meta path does not have a parent and/or is filesystem root: {}", p.display()),
            // Self::CannotReadMetaDir(ref err) => write!(f, "unable to read entries in meta directory: {}", err),
            // Self::CannotReadMetaDirEntry(ref err) => write!(f, "unable to read meta directory entry: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::CannotReadItemDir(ref err) => Some(err),
            Self::CannotReadItemDirEntry(ref err) => Some(err),
            // Self::CannotReadMetaDir(ref err) => Some(err),
            // Self::CannotReadMetaDirEntry(ref err) => Some(err),
            _ => None,
        }
    }
}

/// Represents the target location of the item files that a given metadata file
/// provides metadata for, relative to the location of the metadata file itself.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, EnumIter)]
pub enum Target {
    Siblings,
    Parent,
}

impl Target {
    /// Provides the meta file path that provides metadata for an item file for
    /// this target.
    // NOTE: This always returns a `PathBuf`, since joining paths is required.
    pub fn get_meta_path<'a, P>(
        &'a self,
        item_path: P,
        serialize_format: SerializeFormat,
    ) -> Result<PathBuf, Error>
    where
        P: Into<Cow<'a, Path>>,
    {
        let item_path = item_path.into();

        if !item_path.exists() {
            return Err(Error::NonexistentItemPath(item_path.into()))
        }

        let meta_path_parent_dir = match self {
            Self::Parent => {
                if !item_path.as_ref().is_dir() {
                    return Err(Error::InvalidItemDirPath(item_path.into()))
                }

                item_path.as_ref()
            },
            Self::Siblings => {
                match item_path.as_ref().parent() {
                    Some(item_path_parent) => item_path_parent,
                    None => Err(Error::NoItemPathParent(item_path.into()))?,
                }
            }
        };

        // Start with the default extension of the meta format.
        let exts =
            std::iter::once(serialize_format.default_file_extension())
            .chain(
                serialize_format
                .extra_file_extensions()
                .into_iter()
                .copied()
            )
        ;

        let mut attempted_paths = vec![];

        for ext in exts {
            // Create the target meta file name.
            let target_fn = format!("{}.{}", self.default_file_name(), ext);
            let meta_path = meta_path_parent_dir.join(target_fn);

            // LEARN: This is done to avoid calling `.clone()` unnecessarily.
            match (meta_path.exists(), meta_path.is_file()) {
                // Only an error if all extensions do not match.
                (false, _) => {
                    attempted_paths.push(meta_path);
                    continue
                },

                // Found a directory with the name of the meta file, that would be a very strange case.
                (_, false) => return Err(Error::InvalidMetaFilePath(meta_path)),

                (true, true) => return Ok(meta_path),
            };
        }

        // At this point, no valid meta paths were found.
        Err(Error::NonexistentMetaPath(attempted_paths))
    }

    /// Provides the possible owned item paths of this target.
    /// This is a listing of the file paths that this meta target could/should provide metadata for.
    /// Note that this does NOT parse meta files, it only uses file system locations and presence.
    /// Also, no filtering or sorting of the returned item paths is performed.
    pub fn get_item_paths<'a, P>(&'a self, meta_path: P) -> Result<Vec<PathBuf>, Error>
    where
        P: Into<Cow<'a, Path>>,
    {
        let meta_path = meta_path.into();

        if !meta_path.exists() {
            return Err(Error::NonexistentMetaPath(vec![meta_path.into()]))
        }

        if !meta_path.is_file() {
            return Err(Error::InvalidMetaFilePath(meta_path.into()))
        }

        // Get the parent directory of the meta file.
        // NOTE: This is only outside the pattern match because all branches currently use it.
        if let Some(meta_parent_dir_path) = meta_path.parent() {
            let mut po_item_paths = vec![];

            match self {
                Self::Parent => {
                    // This is just the passed-in path, just push it on unchanged.
                    po_item_paths.push(meta_parent_dir_path.into());
                },
                Self::Siblings => {
                    // Return all children of this directory.
                    for entry in std::fs::read_dir(&meta_parent_dir_path).map_err(Error::CannotReadItemDir)? {
                        po_item_paths.push(entry.map_err(Error::CannotReadItemDirEntry)?.path());
                    }
                },
            }

            Ok(po_item_paths)
        }
        else {
            // This should never happen!
            Err(Error::NoMetaPathParent(meta_path.into()))?
        }
    }

    // NOTE: No sorting is performed, sorting only occurs if needed during plexing.
    pub fn get_selected_item_paths<'a, P>(
        &'a self,
        meta_path: P,
        selection: &'a Selection,
        ) -> Result<Vec<PathBuf>, Error>
    where
        P: Into<Cow<'a, Path>>,
    {
        let mut item_paths = self.get_item_paths(meta_path)?;

        item_paths.retain(|p| selection.is_selected(p));

        Ok(item_paths)
    }

    pub fn default_file_name(&self) -> &'static str {
        match self {
            Self::Parent => "self",
            Self::Siblings => "item",
        }
    }
}

// enum ItemPaths<'a> {
//     Parent(Option<&'a Path>),
//     Siblings(ReadDir),
// }

// impl<'a> Iterator for ItemPaths<'a> {
//     type Item = Result<Cow<'a, Path>, IoError>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             Self::Parent(o) => o.take().map(Cow::Borrowed).map(Result::Ok),
//             Self::Siblings(rd) => rd.next().map(|dir_res| {
//                 dir_res.map(|entry| Cow::Owned(entry.path()))
//             }),
//         }
//     }
// }

// struct SelectedItemPaths<'a>(&'a Selection, ItemPaths<'a>);

// impl<'a> Iterator for SelectedItemPaths<'a> {
//     type Item = Result<Cow<'a, Path>, IoError>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let selection = &self.0;
//         self.1.find(|res| match res {
//             Ok(p) => selection.is_selected(p),
//             Err(_) => true,
//         })
//     }
// }

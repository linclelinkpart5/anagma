use std::path::Path;
use std::path::PathBuf;
use std::fs;

use crate::config::selection::Selection;
use crate::config::meta_format::MetaFormat;

#[derive(Debug)]
pub enum Error {
    InvalidItemDirPath(PathBuf),
    // InvalidItemFilePath(PathBuf),
    NonexistentItemPath(PathBuf),
    NoItemPathParent(PathBuf),
    CannotReadItemDir(std::io::Error),
    CannotReadItemDirEntry(std::io::Error),

    // InvalidMetaDirPath(PathBuf),
    InvalidMetaFilePath(PathBuf),
    NonexistentMetaPath(Vec<PathBuf>),
    NoMetaPathParent(PathBuf),
    // CannotReadMetaDir(std::io::Error),
    // CannotReadMetaDirEntry(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidItemDirPath(ref p) => write!(f, "invalid item directory path: {}", p.display()),
            // Error::InvalidItemFilePath(ref p) => write!(f, "invalid item file path: {}", p.display()),
            Error::NonexistentItemPath(ref p) => write!(f, "item path does not exist: {}", p.display()),
            Error::NoItemPathParent(ref p) => write!(f, "item path does not have a parent and/or is filesystem root: {}", p.display()),
            Error::CannotReadItemDir(ref err) => write!(f, "unable to read entries in item directory: {}", err),
            Error::CannotReadItemDirEntry(ref err) => write!(f, "unable to read item directory entry: {}", err),

            // Error::InvalidMetaDirPath(ref p) => write!(f, "invalid meta directory path: {}", p.display()),
            Error::InvalidMetaFilePath(ref p) => write!(f, "invalid meta file path: {}", p.display()),
            Error::NonexistentMetaPath(ref ps) => write!(f, "meta path does not exist, tried: {:?}", ps),
            Error::NoMetaPathParent(ref p) => write!(f, "meta path does not have a parent and/or is filesystem root: {}", p.display()),
            // Error::CannotReadMetaDir(ref err) => write!(f, "unable to read entries in meta directory: {}", err),
            // Error::CannotReadMetaDirEntry(ref err) => write!(f, "unable to read meta directory entry: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::CannotReadItemDir(ref err) => Some(err),
            Error::CannotReadItemDirEntry(ref err) => Some(err),
            // Error::CannotReadMetaDir(ref err) => Some(err),
            // Error::CannotReadMetaDirEntry(ref err) => Some(err),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum MetaLocation {
    Contains,
    Siblings,
}

impl MetaLocation {
    pub fn get_meta_path<P: AsRef<Path>>(&self, item_path: P, meta_format: MetaFormat) -> Result<PathBuf, Error> {
        let item_path = item_path.as_ref();

        if !item_path.exists() {
            Err(Error::NonexistentItemPath(item_path.to_path_buf()))?
        }

        let meta_path_parent_dir = match *self {
            MetaLocation::Contains => {
                if !item_path.is_dir() {
                    Err(Error::InvalidItemDirPath(item_path.to_path_buf()))?
                }

                item_path
            },
            MetaLocation::Siblings => {
                match item_path.parent() {
                    Some(item_path_parent) => item_path_parent,
                    None => Err(Error::NoItemPathParent(item_path.to_path_buf()))?,
                }
            }
        };

        // Start with the default extension of the meta format.
        let exts = std::iter::once(meta_format.default_file_extension()).chain(meta_format.extra_file_extensions().into_iter().cloned());

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

    /// Provides the possible owned item paths of this location.
    /// This is a listing of the file paths that this meta location *could* provide metadata for.
    /// Note that this does NOT parse meta files, it only uses file system locations and presence.
    /// Also, no filtering or sorting of the returned item paths is performed.
    pub fn get_item_paths<P: AsRef<Path>>(&self, meta_path: P) -> Result<Vec<PathBuf>, Error> {
        let meta_path = meta_path.as_ref();

        if !meta_path.exists() {
            Err(Error::NonexistentMetaPath(vec![meta_path.to_path_buf()]))?
        }

        if !meta_path.is_file() {
            Err(Error::InvalidMetaFilePath(meta_path.to_path_buf()))?
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
                    for entry in fs::read_dir(&meta_parent_dir_path).map_err(Error::CannotReadItemDir)? {
                        po_item_paths.push(entry.map_err(Error::CannotReadItemDirEntry)?.path());
                    }
                },
            }

            Ok(po_item_paths)
        }
        else {
            // This should never happen!
            Err(Error::NoMetaPathParent(meta_path.to_path_buf()))?
        }
    }

    // NOTE: No sorting is performed, sorting only occurs if needed during plexing.
    pub fn get_selected_item_paths<P: AsRef<Path>>(
        &self,
        meta_path: P,
        selection: &Selection,
        ) -> Result<Vec<PathBuf>, Error>
    {
        let item_paths = self.get_item_paths(meta_path)?;

        // Use the config object to select the item paths.
        Ok(selection.select(item_paths).collect())
    }

    pub fn default_file_name(&self) -> &'static str {
        match *self {
            MetaLocation::Contains => "self",
            MetaLocation::Siblings => "item",
        }
    }
}

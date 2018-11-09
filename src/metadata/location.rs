use std::path::Path;
use std::path::PathBuf;
use std::fs;

use config::selection::Selection;

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
    NonexistentMetaPath(PathBuf),
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
            Error::NonexistentMetaPath(ref p) => write!(f, "meta path does not exist: {}", p.display()),
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
    pub fn get_meta_path<P: AsRef<Path>>(&self, item_path: P) -> Result<PathBuf, Error> {
        let item_path = item_path.as_ref();

        if !item_path.exists() {
            Err(Error::NonexistentItemPath(item_path.to_path_buf()))?
        }

        let meta_path = match *self {
            MetaLocation::Contains => {
                if !item_path.is_dir() {
                    Err(Error::InvalidItemDirPath(item_path.to_path_buf()))?
                }

                item_path.join("self.yml")
            },
            MetaLocation::Siblings => {
                match item_path.parent() {
                    Some(item_path_parent) => item_path_parent.join("item.yml"),
                    None => Err(Error::NoItemPathParent(item_path.to_path_buf()))?,
                }
            }
        };

        // LEARN: This is done to avoid calling `.clone()` unnecessarily.
        match (meta_path.exists(), meta_path.is_file()) {
            (false, _) => Err(Error::NonexistentMetaPath(meta_path)),
            (_, false) => Err(Error::InvalidMetaFilePath(meta_path)),
            (true, true) => Ok(meta_path),
        }
    }

    /// Provides the possible owned item paths of this location.
    /// This is a listing of the file paths that this meta location *could* provide metadata for.
    /// Note that this does NOT parse meta files, it only uses file system locations and presence.
    /// Also, no filtering or sorting of the returned item paths is performed.
    pub fn get_item_paths<P: AsRef<Path>>(&self, meta_path: P) -> Result<Vec<PathBuf>, Error> {
        let meta_path = meta_path.as_ref();

        if !meta_path.exists() {
            Err(Error::NonexistentMetaPath(meta_path.to_path_buf()))?
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
}

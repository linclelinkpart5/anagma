use std::path::Path;
use std::path::PathBuf;
use std::fs;

use failure::Error;
use failure::ResultExt;

use error::ErrorKind;
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
            Err(ErrorKind::NonexistentPath(item_path.to_path_buf()))?
        }

        let meta_path = match *self {
            MetaLocation::Contains => {
                if !item_path.is_dir() {
                    Err(ErrorKind::InvalidDirPath(item_path.to_path_buf()))?
                }

                item_path.join("self.yml")
            },
            MetaLocation::Siblings => {
                match item_path.parent() {
                    Some(item_path_parent) => item_path_parent.join("item.yml"),
                    None => Err(ErrorKind::NoPathParent(item_path.to_path_buf()))?,
                }
            }
        };

        if !meta_path.exists() {
            Err(ErrorKind::NonexistentPath(meta_path.to_path_buf()))?
        }
        if !meta_path.is_file() {
            Err(ErrorKind::InvalidFilePath(meta_path.to_path_buf()))?
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
            Err(ErrorKind::NonexistentPath(meta_path.to_path_buf()))?
        }

        if !meta_path.is_file() {
            Err(ErrorKind::InvalidFilePath(meta_path.to_path_buf()))?
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
                    for entry in fs::read_dir(&meta_parent_dir_path).map_err(|_| ErrorKind::CannotReadDir(meta_parent_dir_path.to_path_buf()))? {
                        po_item_paths.push(entry.map_err(|_| ErrorKind::CannotReadDirEntry)?.path());
                    }
                },
            }

            Ok(po_item_paths)
        }
        else {
            // This should never happen!
            Err(ErrorKind::NoPathParent(meta_path.to_path_buf()))?
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
        Ok(config.select(item_paths))
    }
}

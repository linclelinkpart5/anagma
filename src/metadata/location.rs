use std::path::Path;
use std::path::PathBuf;
use std::fs;

use failure::Error;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum MetaLocation {
    Contains(String),
    Siblings(String),
}

impl MetaLocation {
    pub fn get_owning_meta_path<P: AsRef<Path>>(&self, item_path: P) -> Result<PathBuf, Error> {
        let item_path = item_path.as_ref();

        ensure!(item_path.exists(), format!("item path does not exist: {}", item_path.to_string_lossy()));

        let meta_path = match *self {
            MetaLocation::Contains(ref file_name) => {
                ensure!(item_path.is_dir(), format!("item path is not a directory: {}", item_path.to_string_lossy()));

                item_path.join(file_name)
            },
            MetaLocation::Siblings(ref file_name) => {
                match item_path.parent() {
                    Some(item_path_parent) => item_path_parent.join(file_name),
                    None => bail!(format!("item path does not have a parent or is system root: {}", item_path.to_string_lossy())),
                }
            }
        };

        ensure!(meta_path.exists(), format!("meta path does not exist: {}", meta_path.to_string_lossy()));
        ensure!(meta_path.is_file(), format!("meta path is not a file: {}", meta_path.to_string_lossy()));

        Ok(meta_path)
    }

    /// Provides the possible owned item paths of this location.
    /// This is a listing of the file paths that this meta location *could* provide metadata for.
    /// Note that this does NOT parse meta files, it only uses file system locations and presence.
    /// Also, no filtering or sorting of the returned item paths is performed.
    pub fn get_possible_owned_item_paths<P: AsRef<Path>>(&self, item_dir_path: P) -> Result<Vec<PathBuf>, Error> {
        let item_dir_path = item_dir_path.as_ref();

        ensure!(item_dir_path.exists(), format!("path does not exist: {}", item_dir_path.to_string_lossy()));
        ensure!(item_dir_path.is_dir(), format!("path is not a directory: {}", item_dir_path.to_string_lossy()));

        let mut po_item_paths = vec![];

        match *self {
            MetaLocation::Contains(_) => {
                // This is just the passed-in path, just push it on unchanged.
                po_item_paths.push(item_dir_path.to_path_buf());
            },
            MetaLocation::Siblings(_) => {
                // Return all children of this directory.
                for entry in fs::read_dir(&item_dir_path)? {
                    po_item_paths.push(entry?.path());
                }
            },
        }

        Ok(po_item_paths)
    }
}

use std::path::Path;
use std::path::PathBuf;
use std::fs;

use failure::Error;

use library::config::Config;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum MetaLocation {
    Contains,
    Siblings,
}

impl MetaLocation {
    pub fn get_meta_path<P: AsRef<Path>>(&self, item_path: P) -> Result<PathBuf, Error> {
        let item_path = item_path.as_ref();

        ensure!(item_path.exists(), format!("item path does not exist: {}", item_path.to_string_lossy()));

        let meta_path = match *self {
            MetaLocation::Contains => {
                ensure!(item_path.is_dir(), format!("item path is not a directory: {}", item_path.to_string_lossy()));

                item_path.join("self.yml")
            },
            MetaLocation::Siblings => {
                match item_path.parent() {
                    Some(item_path_parent) => item_path_parent.join("item.yml"),
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
    pub fn get_item_paths<P: AsRef<Path>>(&self, meta_path: P) -> Result<Vec<PathBuf>, Error> {
        let meta_path = meta_path.as_ref();

        ensure!(meta_path.exists(), format!("meta path does not exist: {}", meta_path.to_string_lossy()));
        ensure!(meta_path.is_file(), format!("meta path is not a file: {}", meta_path.to_string_lossy()));

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
                    for entry in fs::read_dir(&meta_parent_dir_path)? {
                        po_item_paths.push(entry?.path());
                    }
                },
            }

            Ok(po_item_paths)
        }
        else {
            // This should never happen!
            bail!(format!("meta path does not have a parent directory: {}", meta_path.to_string_lossy()));
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

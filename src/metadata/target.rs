use std::path::Path;
use std::path::PathBuf;

use failure::Error;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum MetaTarget {
    Contains(PathBuf),
    Siblings(PathBuf),
}

impl MetaTarget {
    pub fn get_target_meta_path<P: AsRef<Path>>(&self, item_path: P) -> Result<PathBuf, Error> {
        let item_path = item_path.as_ref();

        ensure!(item_path.exists(), format!("item path does not exist: {}", item_path.to_string_lossy()));

        let meta_path = match *self {
            MetaTarget::Contains(ref file_name) => {
                ensure!(item_path.is_dir(), format!("item path is not a directory: {}", item_path.to_string_lossy()));

                item_path.join(file_name)
            },
            MetaTarget::Siblings(ref file_name) => {
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

    // pub fn get_target_metadata<P: AsRef<Path>, MR: MetaReader>(&self, item_path: P) -> Result<Metadata> {
    //     let meta_path = self.get_target_meta_path(item_path)?;

    //     // Try to load metadata in the format associated with the MetaTarget.
    //     match *self {
    //         MetaTarget::Contains => {},
    //         MetaTarget::Siblings => {},
    //     }

    //     Ok(hashmap![])
    // }
}
